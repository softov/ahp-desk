//! An SSH session to a server: the connection, the sign-in, and the commands
//! run there. Built on `russh`, as doop's installer is, without its
//! accept-any-key handler: the server key goes to the `check` the caller gives,
//! which is `KnownKeys::checker` in the desk.

pub mod known_keys;
pub mod password;
#[cfg(test)]
pub(crate) mod test_server;
pub mod tunnel;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use russh::client::{self, Handle};
use russh::keys::agent::client::AgentClient;
use russh::keys::agent::AgentIdentity;
use russh::keys::{decode_secret_key, PrivateKey, PrivateKeyWithHashAlg};
use russh::ChannelMsg;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use known_keys::{Fingerprint, KeyCheck};

/// How often the session pings the server, so a tunnel notices a dead link.
const KEEPALIVE: Duration = Duration::from_secs(15);
/// Pings unanswered before the session counts as gone.
const KEEPALIVE_MAX: usize = 3;
/// How often a running command looks at its cancel flag.
const CANCEL_POLL: Duration = Duration::from_millis(200);

/// The server to reach and the user to sign in as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshTarget {
    pub address: String,
    pub port: u16,
    pub user: String,
    pub key_file: Option<PathBuf>,
}

impl SshTarget {
    /// `address` is `host` or `host:port`; `port` is used when it names none.
    pub fn new(address: &str, port: u16, user: &str, key_file: Option<PathBuf>) -> Self {
        let (address, port) = match address.rsplit_once(':') {
            // An IPv6 address has colons of its own and needs its port apart.
            Some((host, given)) if !host.contains(':') => match given.parse() {
                Ok(given) => (host.to_string(), given),
                Err(_) => (address.to_string(), port),
            },
            _ => (address.to_string(), port),
        };
        SshTarget {
            address,
            port,
            user: user.to_string(),
            key_file,
        }
    }
}

/// The way a sign-in went through, which the host keeps as `signIn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignIn {
    Agent,
    KeyFile,
    Password,
}

/// What a person types that the sign-in may need.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Need {
    Passphrase,
    Password,
}

/// Why a connection did not open. A refused key is refused before any
/// sign-in, so the person is asked nothing on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", rename_all = "camelCase")]
pub enum ConnectError {
    /// A key neither `known_hosts` nor the desk knows; Trust records it and
    /// the shell connects again.
    #[serde(rename_all = "camelCase")]
    NewKey {
        offered: Fingerprint,
    },
    /// Another key than the one known; Forget removes the desk's, and one from
    /// `known_hosts` is removed with `ssh-keygen -R <host>`.
    #[serde(rename_all = "camelCase")]
    ChangedKey {
        known: Fingerprint,
        offered: Fingerprint,
        in_known_hosts: bool,
    },
    Failed {
        why: String,
    },
}

impl std::fmt::Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectError::NewKey { offered } => write!(
                f,
                "the server's {} key {} is not trusted yet",
                offered.key_type, offered.sha256
            ),
            ConnectError::ChangedKey { known, offered, .. } => write!(
                f,
                "the server's key changed from {} to {}",
                known.sha256, offered.sha256
            ),
            ConnectError::Failed { why } => write!(f, "{why}"),
        }
    }
}

/// How a sign-in failed, as the shell reads it: `kind`, and for `connect`
/// the `reason` of `ConnectError`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SignInError {
    /// The server could not be reached, or its key was refused.
    Connect(ConnectError),
    /// The key file is encrypted and no passphrase that opens it is held;
    /// the form asks for it.
    Passphrase,
    /// The host signed in before with a passphrase or a password that is not
    /// held now, as after a restart; its row shows "Sign in".
    NeedsPerson { what: Need },
    /// Every way was tried and none went through, each named with why.
    Refused { tried: Vec<String> },
}

impl std::fmt::Display for SignInError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignInError::Connect(why) => write!(f, "cannot connect: {why}"),
            SignInError::Passphrase => write!(f, "the key file needs its passphrase"),
            SignInError::NeedsPerson {
                what: Need::Passphrase,
            } => {
                write!(f, "sign in again: the key file's passphrase is needed")
            }
            SignInError::NeedsPerson {
                what: Need::Password,
            } => {
                write!(f, "sign in again: the password is needed")
            }
            SignInError::Refused { tried } => {
                write!(f, "the server refused every sign-in: {}", tried.join("; "))
            }
        }
    }
}

/// What a person typed, per host id, for this process only: never on disk,
/// in an event or in a log line.
#[derive(Default)]
pub struct Secrets(Mutex<HashMap<String, Held>>);

#[derive(Default)]
struct Held {
    passphrase: Option<String>,
    password: Option<String>,
}

impl Secrets {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Held>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_passphrase(&self, id: &str, passphrase: &str) {
        self.lock().entry(id.into()).or_default().passphrase = Some(passphrase.into());
    }

    pub fn set_password(&self, id: &str, password: &str) {
        self.lock().entry(id.into()).or_default().password = Some(password.into());
    }

    fn passphrase(&self, id: &str) -> Option<String> {
        self.lock().get(id).and_then(|held| held.passphrase.clone())
    }

    fn password(&self, id: &str) -> Option<String> {
        self.lock().get(id).and_then(|held| held.password.clone())
    }

    fn drop_passphrase(&self, id: &str) {
        if let Some(held) = self.lock().get_mut(id) {
            held.passphrase = None;
        }
    }

    fn drop_password(&self, id: &str) {
        if let Some(held) = self.lock().get_mut(id) {
            held.password = None;
        }
    }

    /// Drops everything held for `id`, as when its host is removed.
    pub fn forget(&self, id: &str) {
        self.lock().remove(id);
    }

    #[cfg(test)]
    pub fn holds(&self, id: &str) -> bool {
        self.lock().contains_key(id)
    }
}

/// What is known of a server key, answered at once: the desk's is
/// `KnownKeys::checker`, and a test gives its own.
pub type ServerCheck = Box<dyn Fn(&russh::keys::ssh_key::PublicKey) -> KeyCheck + Send + Sync>;

pub struct Client {
    check: ServerCheck,
    /// The answer on a refused key, which `connect` turns into its error.
    refused: Arc<Mutex<Option<KeyCheck>>>,
}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let answer = (self.check)(key);
        if answer == KeyCheck::Known {
            return Ok(true);
        }
        *self
            .refused
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(answer);
        Ok(false)
    }
}

pub type Session = Handle<Client>;

/// Opens a connection to `target`, handing the server key to `check`; a key
/// it does not know is refused as `NewKey` or `ChangedKey`.
pub async fn connect(target: &SshTarget, check: ServerCheck) -> Result<Session, ConnectError> {
    let config = Arc::new(client::Config {
        keepalive_interval: Some(KEEPALIVE),
        keepalive_max: KEEPALIVE_MAX,
        ..Default::default()
    });
    let refused = Arc::new(Mutex::new(None));
    let client = Client {
        check,
        refused: refused.clone(),
    };
    client::connect(config, (target.address.as_str(), target.port), client)
        .await
        .map_err(|err| {
            let refused = refused
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            match refused {
                Some(KeyCheck::New { offered }) => ConnectError::NewKey { offered },
                Some(KeyCheck::Changed {
                    known,
                    offered,
                    in_known_hosts,
                }) => ConnectError::ChangedKey {
                    known,
                    offered,
                    in_known_hosts,
                },
                Some(KeyCheck::Known) | None => ConnectError::Failed {
                    why: err.to_string(),
                },
            }
        })
}

/// A transport failure in the middle of a sign-in.
fn failed(err: impl std::fmt::Display) -> SignInError {
    SignInError::Connect(ConnectError::Failed {
        why: err.to_string(),
    })
}

/// The secrets a sign-in may use besides the agent and the key file.
pub struct Keys<'a> {
    /// The host's id, under which `secrets` holds what was typed.
    pub id: &'a str,
    pub secrets: &'a Secrets,
    /// The password kept for the host, in the keyring or the plain file.
    pub kept_password: Option<String>,
    /// How the host signed in before; `None` on a first sign-in from the form.
    pub signed_in_before: Option<SignIn>,
}

/// Connects and signs in, as `connect` and `sign_in` do.
pub async fn open(
    target: &SshTarget,
    check: ServerCheck,
    keys: Keys<'_>,
) -> Result<(Session, SignIn), SignInError> {
    let mut session = connect(target, check).await.map_err(SignInError::Connect)?;
    let way = sign_in(&mut session, target, keys).await?;
    Ok((session, way))
}

/// Signs in with the agent, then the key file, then a password, and answers
/// the way that went through.
pub async fn sign_in(
    session: &mut Session,
    target: &SshTarget,
    keys: Keys<'_>,
) -> Result<SignIn, SignInError> {
    let mut tried = Vec::new();
    match try_agent(session, &target.user).await {
        Ok(true) => return Ok(SignIn::Agent),
        Ok(false) => tried.push("agent: no key it holds was taken".to_string()),
        Err(why) => tried.push(format!("agent: {why}")),
    }

    if let Some(path) = &target.key_file {
        match key_file(path, &keys) {
            Ok(key) => {
                if publickey(session, &target.user, key).await? {
                    return Ok(SignIn::KeyFile);
                }
                tried.push(format!("key file {}: not taken", path.display()));
            }
            Err(KeyFileError::Encrypted) => {
                return Err(match keys.signed_in_before {
                    Some(SignIn::KeyFile) => SignInError::NeedsPerson {
                        what: Need::Passphrase,
                    },
                    _ => SignInError::Passphrase,
                });
            }
            Err(KeyFileError::Unreadable(why)) => {
                tried.push(format!("key file {}: {why}", path.display()))
            }
        }
    }

    let typed = keys.secrets.password(keys.id);
    match typed.clone().or(keys.kept_password.clone()) {
        Some(password) => {
            let auth = session
                .authenticate_password(&target.user, password)
                .await
                .map_err(failed)?;
            if auth.success() {
                return Ok(SignIn::Password);
            }
            // A refused password is not tried again without the person.
            if typed.is_some() {
                keys.secrets.drop_password(keys.id);
            }
            tried.push("password: refused".to_string());
        }
        None if keys.signed_in_before == Some(SignIn::Password) => {
            return Err(SignInError::NeedsPerson {
                what: Need::Password,
            });
        }
        None => tried.push("password: none given".to_string()),
    }
    Err(SignInError::Refused { tried })
}

enum KeyFileError {
    /// Encrypted, and no passphrase held opens it.
    Encrypted,
    Unreadable(String),
}

/// The key file, opened with the passphrase held for the host when it needs
/// one; a held passphrase that does not open it is dropped.
fn key_file(path: &std::path::Path, keys: &Keys<'_>) -> Result<PrivateKey, KeyFileError> {
    let text =
        std::fs::read_to_string(path).map_err(|err| KeyFileError::Unreadable(err.to_string()))?;
    match decode_secret_key(&text, None) {
        Ok(key) => Ok(key),
        Err(russh::keys::Error::KeyIsEncrypted) => {
            let Some(passphrase) = keys.secrets.passphrase(keys.id) else {
                return Err(KeyFileError::Encrypted);
            };
            decode_secret_key(&text, Some(&passphrase)).map_err(|_| {
                keys.secrets.drop_passphrase(keys.id);
                KeyFileError::Encrypted
            })
        }
        Err(err) => Err(KeyFileError::Unreadable(err.to_string())),
    }
}

async fn publickey(
    session: &mut Session,
    user: &str,
    key: PrivateKey,
) -> Result<bool, SignInError> {
    let hash = session
        .best_supported_rsa_hash()
        .await
        .map_err(failed)?
        .flatten();
    let auth = session
        .authenticate_publickey(user, PrivateKeyWithHashAlg::new(Arc::new(key), hash))
        .await
        .map_err(failed)?;
    Ok(auth.success())
}

/// Signs in with a key the SSH agent holds: `$SSH_AUTH_SOCK` on Unix; the
/// OpenSSH agent's pipe, then Pageant, on Windows. `Err` when there is none.
async fn try_agent(session: &mut Session, user: &str) -> Result<bool, String> {
    #[cfg(unix)]
    {
        let agent = AgentClient::connect_env()
            .await
            .map_err(|err| format!("none reachable ({err})"))?;
        agent_sign_in(session, user, agent).await
    }
    #[cfg(windows)]
    {
        let mut reached = false;
        if let Ok(agent) = AgentClient::connect_named_pipe(r"\\.\pipe\openssh-ssh-agent").await {
            reached = true;
            if agent_sign_in(session, user, agent).await? {
                return Ok(true);
            }
        }
        if let Ok(agent) = AgentClient::connect_pageant().await {
            reached = true;
            if agent_sign_in(session, user, agent).await? {
                return Ok(true);
            }
        }
        if reached {
            Ok(false)
        } else {
            Err("none reachable".into())
        }
    }
}

async fn agent_sign_in<S>(
    session: &mut Session,
    user: &str,
    mut agent: AgentClient<S>,
) -> Result<bool, String>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let identities = agent
        .request_identities()
        .await
        .map_err(|err| err.to_string())?;
    let hash = session
        .best_supported_rsa_hash()
        .await
        .map_err(|err| err.to_string())?
        .flatten();
    for identity in identities {
        // A certificate is not offered; a plain key is.
        let AgentIdentity::PublicKey { key, .. } = identity else {
            continue;
        };
        if let Ok(auth) = session
            .authenticate_publickey_with(user, key, hash, &mut agent)
            .await
        {
            if auth.success() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Runs `command` on the server and answers its exit code with its stdout
/// and stderr together; -1 when the server sent no exit code.
pub async fn exec_capture(session: &Session, command: &str) -> Result<(i32, String), String> {
    let mut channel = session
        .channel_open_session()
        .await
        .map_err(|err| err.to_string())?;
    channel
        .exec(true, command)
        .await
        .map_err(|err| err.to_string())?;
    let mut output = Vec::new();
    let mut code = -1;
    while let Some(message) = channel.wait().await {
        match message {
            ChannelMsg::Data { ref data } | ChannelMsg::ExtendedData { ref data, .. } => {
                output.extend_from_slice(data)
            }
            ChannelMsg::ExitStatus { exit_status } => code = exit_status as i32,
            _ => {}
        }
    }
    Ok((code, String::from_utf8_lossy(&output).into_owned()))
}

/// Runs `command` on the server, sending each line of its stdout and stderr
/// as it comes, and fails with the exit code when it is not 0. A raised
/// `cancel` closes the command and fails as cancelled.
pub async fn exec_lines(
    session: &Session,
    command: &str,
    line: impl Fn(String),
    cancel: &AtomicBool,
) -> Result<(), String> {
    let mut channel = session
        .channel_open_session()
        .await
        .map_err(|err| err.to_string())?;
    channel
        .exec(true, command)
        .await
        .map_err(|err| err.to_string())?;
    // Each stream keeps its unfinished line apart, so the two never mix in one.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut code = None;
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = channel.close().await;
            return Err("cancelled".into());
        }
        let message = match tokio::time::timeout(CANCEL_POLL, channel.wait()).await {
            Ok(Some(message)) => message,
            Ok(None) => break,
            Err(_) => continue,
        };
        match message {
            ChannelMsg::Data { ref data } => send_lines(&mut out, data, &line),
            ChannelMsg::ExtendedData { ref data, .. } => send_lines(&mut err, data, &line),
            ChannelMsg::ExitStatus { exit_status } => code = Some(exit_status),
            _ => {}
        }
    }
    for rest in [out, err] {
        if !rest.is_empty() {
            line(String::from_utf8_lossy(&rest).into_owned());
        }
    }
    match code {
        Some(0) => Ok(()),
        Some(code) => Err(format!("exited with {code}")),
        None => Err("the server sent no exit code".into()),
    }
}

/// Appends `data` to `pending` and sends each line it completes.
fn send_lines(pending: &mut Vec<u8>, data: &[u8], line: &impl Fn(String)) {
    pending.extend_from_slice(data);
    while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
        let text: Vec<u8> = pending.drain(..=end).collect();
        let text = String::from_utf8_lossy(&text[..end]);
        line(text.trim_end_matches('\r').to_string());
    }
}

/// `s` as one word for the server's shell: left as it is when plain,
/// single-quoted otherwise. Every path and name in a remote command goes
/// through it.
pub fn quote(s: &str) -> String {
    let plain = !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '@' | ':' | '+')
        });
    if plain {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::known_keys::tests::Scratch;
    use super::test_server::{TestServer, PASSWORD, USER};
    use super::*;
    use std::sync::Mutex as StdMutex;

    fn trust_any() -> ServerCheck {
        Box::new(|_| KeyCheck::Known)
    }

    fn keys<'a>(secrets: &'a Secrets, before: Option<SignIn>) -> Keys<'a> {
        Keys {
            id: "dev",
            secrets,
            kept_password: None,
            signed_in_before: before,
        }
    }

    #[test]
    fn ssh_the_address_may_name_its_port() {
        assert_eq!(SshTarget::new("dev02", 22, "u", None).port, 22);
        let target = SshTarget::new("dev02:2222", 22, "u", None);
        assert_eq!((target.address.as_str(), target.port), ("dev02", 2222));
        let v6 = SshTarget::new("fe80::1", 22, "u", None);
        assert_eq!((v6.address.as_str(), v6.port), ("fe80::1", 22));
    }

    #[test]
    fn ssh_quote_leaves_a_plain_word_and_quotes_the_rest() {
        assert_eq!(quote("/home/me/.local"), "/home/me/.local");
        assert_eq!(quote("@ahpd/agent-claude"), "@ahpd/agent-claude");
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote("it's"), "'it'\\''s'");
        assert_eq!(quote(""), "''");
    }

    #[tokio::test]
    async fn ssh_a_password_signs_in() {
        let server = TestServer::start().await;
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let (_session, way) = open(&server.target(None), trust_any(), keys(&secrets, None))
            .await
            .unwrap();
        assert_eq!(way, SignIn::Password);
    }

    #[tokio::test]
    async fn ssh_a_kept_password_signs_in() {
        let server = TestServer::start().await;
        let secrets = Secrets::default();
        let mut with_kept = keys(&secrets, Some(SignIn::Password));
        with_kept.kept_password = Some(PASSWORD.into());
        let (_session, way) = open(&server.target(None), trust_any(), with_kept)
            .await
            .unwrap();
        assert_eq!(way, SignIn::Password);
    }

    #[tokio::test]
    async fn ssh_a_wrong_password_fails_naming_each_way_and_is_dropped() {
        let server = TestServer::start().await;
        let secrets = Secrets::default();
        secrets.set_password("dev", "wrong");
        let Err(SignInError::Refused { tried }) =
            open(&server.target(None), trust_any(), keys(&secrets, None)).await
        else {
            panic!("a wrong password signed in");
        };
        assert!(tried[0].starts_with("agent"), "{tried:?}");
        assert_eq!(tried.last().unwrap(), "password: refused");
        assert_eq!(secrets.password("dev"), None);
    }

    #[tokio::test]
    async fn ssh_a_key_file_signs_in() {
        let server = TestServer::start().await;
        let file = server.key_file(None);
        let secrets = Secrets::default();
        let (_session, way) = open(
            &server.target(Some(file.path())),
            trust_any(),
            keys(&secrets, None),
        )
        .await
        .unwrap();
        assert_eq!(way, SignIn::KeyFile);
    }

    #[tokio::test]
    async fn ssh_an_encrypted_key_file_asks_its_passphrase_then_signs_in_and_again() {
        let server = TestServer::start().await;
        let file = server.key_file(Some("open sesame"));
        let target = server.target(Some(file.path()));
        let secrets = Secrets::default();
        assert_eq!(
            open(&target, trust_any(), keys(&secrets, None)).await.err(),
            Some(SignInError::Passphrase)
        );
        secrets.set_passphrase("dev", "wrong");
        assert_eq!(
            open(&target, trust_any(), keys(&secrets, None)).await.err(),
            Some(SignInError::Passphrase)
        );
        assert_eq!(
            secrets.passphrase("dev"),
            None,
            "a wrong passphrase is dropped"
        );
        secrets.set_passphrase("dev", "open sesame");
        let (_, way) = open(&target, trust_any(), keys(&secrets, None))
            .await
            .unwrap();
        assert_eq!(way, SignIn::KeyFile);
        // Held until the desk quits: a second connection asks nothing.
        let (_, way) = open(&target, trust_any(), keys(&secrets, Some(SignIn::KeyFile)))
            .await
            .unwrap();
        assert_eq!(way, SignIn::KeyFile);
    }

    #[tokio::test]
    async fn ssh_after_a_restart_a_host_needs_its_person_again() {
        let server = TestServer::start().await;
        let file = server.key_file(Some("open sesame"));
        let empty = Secrets::default();
        assert_eq!(
            open(
                &server.target(Some(file.path())),
                trust_any(),
                keys(&empty, Some(SignIn::KeyFile))
            )
            .await
            .err(),
            Some(SignInError::NeedsPerson {
                what: Need::Passphrase
            })
        );
        assert_eq!(
            open(
                &server.target(None),
                trust_any(),
                keys(&empty, Some(SignIn::Password))
            )
            .await
            .err(),
            Some(SignInError::NeedsPerson {
                what: Need::Password
            })
        );
    }

    #[tokio::test]
    async fn ssh_a_new_server_key_is_refused_before_any_sign_in_until_trusted() {
        let server = TestServer::start().await;
        let scratch = Scratch::new();
        let known = scratch.known();
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let target = server.target(None);
        let Err(SignInError::Connect(ConnectError::NewKey { offered })) = open(
            &target,
            known.checker(&target.address, target.port),
            keys(&secrets, None),
        )
        .await
        else {
            panic!("a new key connected");
        };
        assert_eq!(
            server.sign_in_attempts(),
            0,
            "nothing is asked on a refused key"
        );
        known
            .confirm(&target.address, target.port, &offered.sha256)
            .unwrap();
        let (_, way) = open(
            &target,
            known.checker(&target.address, target.port),
            keys(&secrets, None),
        )
        .await
        .unwrap();
        assert_eq!(way, SignIn::Password);
    }

    #[tokio::test]
    async fn ssh_a_changed_server_key_is_refused_with_both_and_forget_asks_again() {
        let server = TestServer::start().await;
        let scratch = Scratch::new();
        let known = scratch.known();
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let target = server.target(None);
        let connect = || {
            open(
                &target,
                known.checker(&target.address, target.port),
                keys(&secrets, None),
            )
        };
        let Err(SignInError::Connect(ConnectError::NewKey { offered: first })) = connect().await
        else {
            panic!("a new key connected");
        };
        known
            .confirm(&target.address, target.port, &first.sha256)
            .unwrap();
        server.change_host_key();
        let Err(SignInError::Connect(ConnectError::ChangedKey {
            known: old,
            offered,
            in_known_hosts,
        })) = connect().await
        else {
            panic!("a changed key connected");
        };
        assert_eq!(old, first);
        assert_ne!(offered, first);
        assert!(!in_known_hosts);
        known.forget(&target.address, target.port).unwrap();
        let Err(SignInError::Connect(ConnectError::NewKey { offered: again })) = connect().await
        else {
            panic!("a forgotten key's server connected");
        };
        assert_eq!(again, offered);
        known
            .confirm(&target.address, target.port, &again.sha256)
            .unwrap();
        assert_eq!(connect().await.unwrap().1, SignIn::Password);
    }

    async fn signed_in(server: &TestServer) -> Session {
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        open(&server.target(None), trust_any(), keys(&secrets, None))
            .await
            .unwrap()
            .0
    }

    #[tokio::test]
    async fn ssh_exec_capture_answers_the_code_and_both_streams() {
        let server = TestServer::start().await;
        let session = signed_in(&server).await;
        let (code, output) = exec_capture(&session, "fail").await.unwrap();
        assert_eq!(code, 3);
        assert_eq!(output, "partial\nbad\n");
        assert_eq!(
            exec_capture(&session, "printf %s \"$HOME\"").await.unwrap(),
            (0, format!("/home/{USER}"))
        );
    }

    #[tokio::test]
    async fn ssh_exec_lines_sends_lines_in_order_and_fails_on_a_non_zero_exit() {
        let server = TestServer::start().await;
        let session = signed_in(&server).await;
        let seen = StdMutex::new(Vec::new());
        let cancel = AtomicBool::new(false);
        exec_lines(
            &session,
            "lines",
            |text| seen.lock().unwrap().push(text),
            &cancel,
        )
        .await
        .unwrap();
        assert_eq!(*seen.lock().unwrap(), ["one", "two", "warn", "three"]);
        let failed = exec_lines(&session, "fail", |_| {}, &cancel).await;
        assert_eq!(failed, Err("exited with 3".into()));
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            exec_lines(&session, "lines", |_| {}, &cancel).await,
            Err("cancelled".into())
        );
    }

    /// From this computer to the test server Softov names, by hand:
    /// `AHP_DESK_SSH_TEST=<user>@<host> AHP_DESK_SSH_TEST_KEY=SHA256:... cargo test -- --ignored ssh_a_real_sign_in`,
    /// with `AHP_DESK_SSH_TEST_KEY_FILE=<path>` to sign in with a key file when no agent runs.
    /// The key is the server's host key fingerprint from `ssh-keygen -lf`; no other key is trusted.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_sign_in() {
        let (target, check) = real_target();
        let secrets = Secrets::default();
        let (session, way) = open(&target, check, keys(&secrets, None)).await.unwrap();
        let expected = match target.key_file {
            Some(_) => [SignIn::Agent, SignIn::KeyFile],
            None => [SignIn::Agent, SignIn::Agent],
        };
        assert!(expected.contains(&way), "signed in by {way:?}");
        let (_, home) = exec_capture(&session, "printf %s \"$HOME\"").await.unwrap();
        assert!(home.starts_with('/'));
    }

    /// The server and the one key trusted for the ignored tests on the test server.
    pub fn real_target() -> (SshTarget, ServerCheck) {
        let at = std::env::var("AHP_DESK_SSH_TEST").expect("AHP_DESK_SSH_TEST=<user>@<host>");
        let trusted =
            std::env::var("AHP_DESK_SSH_TEST_KEY").expect("AHP_DESK_SSH_TEST_KEY=SHA256:...");
        let key_file = std::env::var_os("AHP_DESK_SSH_TEST_KEY_FILE").map(PathBuf::from);
        let (user, host) = at.split_once('@').expect("AHP_DESK_SSH_TEST=<user>@<host>");
        let check: ServerCheck = Box::new(move |key| {
            let offered = known_keys::fingerprint(key);
            if offered.sha256 == trusted {
                KeyCheck::Known
            } else {
                KeyCheck::New { offered }
            }
        });
        (SshTarget::new(host, 22, user, key_file), check)
    }
}
