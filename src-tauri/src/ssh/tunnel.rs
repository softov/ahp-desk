//! A tunnel per `ssh` host: a local port, chosen once and kept on the host,
//! forwarded over SSH to the ahpd on the server's 127.0.0.1, while the desk
//! runs. A session that drops opens again by itself; one that needs the
//! person, or a server key refused, waits for them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde::Serialize;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::net::TcpListener;
use tokio::sync::Notify;

use super::known_keys::KnownKeys;
use super::{open as open_session, Keys, Need, Secrets, Session, SignIn, SignInError, SshTarget};
use crate::hosts::{HostKind, HostStore, Reach};

/// The event every change of a tunnel sends, with every tunnel's state.
pub const TUNNELS_CHANGED: &str = "tunnels-changed";
/// The first wait before a dropped tunnel opens again; it doubles up to `LAST_WAIT`.
const FIRST_WAIT: Duration = Duration::from_secs(2);
const LAST_WAIT: Duration = Duration::from_secs(60);
/// How often an open tunnel looks whether its session is still there.
const WATCH: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelState {
    pub on: bool,
    /// Why it is down: a server key refused, the server unreachable, the
    /// local port taken, or the sign-in.
    pub error: Option<String>,
    /// What the person types to open it again, as after a restart.
    pub needs: Option<Need>,
    /// The server key it refused, new or changed, which Edit host shows
    /// with Trust or Forget.
    pub key: Option<super::ConnectError>,
}

/// What a tunnel needs of its host, read when it opens.
#[derive(Clone, Debug)]
pub struct TunnelSpec {
    pub id: String,
    pub target: SshTarget,
    pub local_port: u16,
    pub remote_port: u16,
    pub kept_password: Option<String>,
    pub signed_in_before: Option<SignIn>,
}

/// The tunnel of an `ssh` host, from what the host list keeps of it.
pub fn spec_of(store: &HostStore, id: &str) -> Result<TunnelSpec, String> {
    let ssh = store
        .ssh(id)?
        .ok_or_else(|| format!("{id} has no SSH details"))?;
    if ssh.reach != Reach::Tunnel {
        return Err(format!("{id} is reached on its open port, not a tunnel"));
    }
    let local_port = ssh
        .local_port
        .ok_or_else(|| format!("{id} has no local port"))?;
    Ok(TunnelSpec {
        id: id.to_string(),
        target: SshTarget::new(
            &ssh.address,
            ssh.port,
            &ssh.user,
            ssh.key_file.map(Into::into),
        ),
        local_port,
        remote_port: ssh.remote_port,
        kept_password: store.ssh_password(id)?,
        signed_in_before: ssh.sign_in,
    })
}

/// Called with every tunnel's state after each change.
type OnChange = Arc<dyn Fn(&HashMap<String, TunnelState>) + Send + Sync>;

struct Running {
    task: JoinHandle<()>,
    retry: Arc<Notify>,
}

pub struct Tunnels {
    known: KnownKeys,
    secrets: Arc<Secrets>,
    on_change: OnChange,
    running: Mutex<HashMap<String, Running>>,
    states: Arc<Mutex<HashMap<String, TunnelState>>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Tunnels {
    pub fn new(known: KnownKeys, secrets: Arc<Secrets>, on_change: OnChange) -> Self {
        Tunnels {
            known,
            secrets,
            on_change,
            running: Mutex::new(HashMap::new()),
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Opens the host's tunnel, in place of one already open.
    pub fn open(&self, spec: TunnelSpec) {
        self.close(&spec.id);
        let retry = Arc::new(Notify::new());
        let set = {
            let (states, on_change, id) =
                (self.states.clone(), self.on_change.clone(), spec.id.clone());
            move |state: TunnelState| {
                let all = {
                    let mut states = lock(&states);
                    states.insert(id.clone(), state);
                    states.clone()
                };
                on_change(&all);
            }
        };
        let task = tauri::async_runtime::spawn(supervise(
            spec.clone(),
            self.known.clone(),
            self.secrets.clone(),
            retry.clone(),
            set,
        ));
        lock(&self.running).insert(spec.id, Running { task, retry });
    }

    /// Opens a down tunnel now, as Retry does; false when the host has none.
    pub fn retry(&self, id: &str) -> bool {
        match lock(&self.running).get(id) {
            Some(running) => {
                running.retry.notify_one();
                true
            }
            None => false,
        }
    }

    /// Holds what the person typed, as the tunnel asked for it, and opens it.
    pub fn sign_in(&self, id: &str, secret: &str) -> Result<(), String> {
        let needs = lock(&self.states).get(id).and_then(|state| state.needs);
        match needs {
            Some(Need::Passphrase) => self.secrets.set_passphrase(id, secret),
            Some(Need::Password) => self.secrets.set_password(id, secret),
            None => return Err(format!("the tunnel of {id} asks for nothing")),
        }
        if self.retry(id) {
            Ok(())
        } else {
            Err(format!("{id} has no tunnel"))
        }
    }

    /// Closes the host's tunnel: its listener, its session and its forwards.
    pub fn close(&self, id: &str) {
        if let Some(running) = lock(&self.running).remove(id) {
            running.task.abort();
        }
        let all = {
            let mut states = lock(&self.states);
            states.remove(id);
            states.clone()
        };
        (self.on_change)(&all);
    }

    pub fn close_all(&self) {
        for (_, running) in lock(&self.running).drain() {
            running.task.abort();
        }
        lock(&self.states).clear();
    }

    pub fn states(&self) -> HashMap<String, TunnelState> {
        lock(&self.states).clone()
    }
}

/// Keeps one tunnel open: connects, binds the local port, forwards, and
/// opens again when the session drops, waiting longer each time.
async fn supervise(
    spec: TunnelSpec,
    known: KnownKeys,
    secrets: Arc<Secrets>,
    retry: Arc<Notify>,
    set: impl Fn(TunnelState),
) {
    let down = |error: String, needs: Option<Need>| TunnelState {
        on: false,
        error: Some(error),
        needs,
        key: None,
    };
    let mut wait = FIRST_WAIT;
    loop {
        let keys = Keys {
            id: &spec.id,
            secrets: &secrets,
            kept_password: spec.kept_password.clone(),
            signed_in_before: spec.signed_in_before,
        };
        let check = known.checker(&spec.target.address, spec.target.port);
        match open_session(&spec.target, check, keys).await {
            Ok((session, _)) => {
                let listener = match TcpListener::bind(("127.0.0.1", spec.local_port)).await {
                    Ok(listener) => listener,
                    Err(err) => {
                        // The tunnel keeps its port: the host's address is the same at every launch.
                        set(down(
                            format!(
                                "local port {} is taken by another program ({err})",
                                spec.local_port
                            ),
                            None,
                        ));
                        retry.notified().await;
                        continue;
                    }
                };
                set(TunnelState {
                    on: true,
                    ..TunnelState::default()
                });
                wait = FIRST_WAIT;
                serve(listener, Arc::new(session), &spec).await;
                set(down("the SSH connection dropped".into(), None));
            }
            // Asking again would ask the server again with nothing new: the person acts first.
            Err(SignInError::NeedsPerson { what }) => {
                set(down(
                    SignInError::NeedsPerson { what }.to_string(),
                    Some(what),
                ));
                retry.notified().await;
                continue;
            }
            Err(SignInError::Passphrase) => {
                set(down(
                    SignInError::Passphrase.to_string(),
                    Some(Need::Passphrase),
                ));
                retry.notified().await;
                continue;
            }
            Err(SignInError::Connect(
                key @ (super::ConnectError::NewKey { .. } | super::ConnectError::ChangedKey { .. }),
            )) => {
                set(TunnelState {
                    key: Some(key.clone()),
                    ..down(key.to_string(), None)
                });
                retry.notified().await;
                continue;
            }
            Err(err @ SignInError::Refused { .. }) => {
                set(down(err.to_string(), None));
                retry.notified().await;
                continue;
            }
            Err(err) => set(down(err.to_string(), None)),
        }
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = retry.notified() => {}
        }
        wait = (wait * 2).min(LAST_WAIT);
    }
}

/// Forwards each connection the listener takes to ahpd on the server, until
/// the session is gone.
async fn serve(listener: TcpListener, session: Arc<Session>, spec: &TunnelSpec) {
    let (remote_port, local_port) = (u32::from(spec.remote_port), u32::from(spec.local_port));
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let Ok((mut local, _)) = accepted else { continue };
                let session = session.clone();
                tauri::async_runtime::spawn(async move {
                    let Ok(channel) = session
                        .channel_open_direct_tcpip("127.0.0.1", remote_port, "127.0.0.1", local_port)
                        .await
                    else {
                        return;
                    };
                    let mut remote = channel.into_stream();
                    // Either side closing ends the copy; nothing waits on its result.
                    let _ = tokio::io::copy_bidirectional(&mut local, &mut remote).await;
                });
            }
            _ = tokio::time::sleep(WATCH) => {
                if session.is_closed() {
                    return;
                }
            }
        }
    }
}

/// A local port the OS has free now, which a new host keeps from then on.
pub fn pick_local_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|err| err.to_string())?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|err| err.to_string())
}

/// Opens every `ssh` host's tunnel, off the main thread, as the desk starts.
pub fn open_on_launch(app: &AppHandle) {
    let store = app.state::<HostStore>();
    let tunnels = app.state::<Tunnels>();
    for host in store.list() {
        if host.kind != HostKind::Ssh {
            continue;
        }
        match spec_of(&store, &host.id) {
            Ok(spec) => tunnels.open(spec),
            Err(err) => eprintln!("ahp-desk: the tunnel of {} cannot open: {err}", host.id),
        }
    }
}

/// The desk's tunnels, which send `tunnels-changed` to every window.
pub fn manage(app: &AppHandle) {
    let emitter = app.clone();
    let on_change: OnChange = Arc::new(move |all| {
        // A window that is gone misses a state it would redraw anyway.
        let _ = emitter.emit(TUNNELS_CHANGED, all);
    });
    let tunnels = Tunnels::new(
        app.state::<KnownKeys>().inner().clone(),
        app.state::<HostStore>().secrets().clone(),
        on_change,
    );
    app.manage(tunnels);
}

#[tauri::command]
pub fn ssh_tunnel_states(tunnels: State<'_, Tunnels>) -> HashMap<String, TunnelState> {
    tunnels.states()
}

/// Opens a down tunnel now, or opens one the host had none of.
#[tauri::command]
pub fn open_ssh_tunnel(
    store: State<'_, HostStore>,
    tunnels: State<'_, Tunnels>,
    id: String,
) -> Result<(), String> {
    if !tunnels.retry(&id) {
        tunnels.open(spec_of(&store, &id)?);
    }
    Ok(())
}

/// What the person typed on a row's "Sign in", held until the desk quits.
#[tauri::command]
pub fn ssh_sign_in(tunnels: State<'_, Tunnels>, id: String, secret: String) -> Result<(), String> {
    tunnels.sign_in(&id, &secret)
}

#[tauri::command]
pub fn pick_ssh_local_port() -> Result<u16, String> {
    pick_local_port()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::known_keys::tests::Scratch;
    use crate::ssh::test_server::{TestServer, PASSWORD};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    /// Tunnels that trust the test server's key, and the states they sent.
    struct Fixture {
        tunnels: Tunnels,
        secrets: Arc<Secrets>,
        known: KnownKeys,
        _scratch: Scratch,
    }

    impl Fixture {
        async fn new(server: &TestServer) -> Self {
            let scratch = Scratch::new();
            let known = scratch.known();
            let secrets = Arc::new(Secrets::default());
            let tunnels = Tunnels::new(known.clone(), secrets.clone(), Arc::new(|_| {}));
            // The key is trusted as a person would, once, before the tunnel opens.
            let target = server.target(None);
            let Err(crate::ssh::ConnectError::NewKey { offered }) =
                crate::ssh::connect(&target, known.checker(&target.address, target.port)).await
            else {
                panic!("the test server's key was already known");
            };
            known
                .confirm(&target.address, target.port, &offered.sha256)
                .unwrap();
            Fixture {
                tunnels,
                secrets,
                known,
                _scratch: scratch,
            }
        }

        async fn until(&self, id: &str, want: impl Fn(&TunnelState) -> bool) -> TunnelState {
            for _ in 0..200 {
                if let Some(state) = self.tunnels.states().get(id) {
                    if want(state) {
                        return state.clone();
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            panic!("{id} never reached the state: {:?}", self.tunnels.states());
        }
    }

    fn spec(
        server: &TestServer,
        key_file: Option<&std::path::Path>,
        local_port: u16,
    ) -> TunnelSpec {
        TunnelSpec {
            id: "dev".into(),
            target: server.target(key_file),
            local_port,
            remote_port: 9187,
            kept_password: None,
            signed_in_before: Some(if key_file.is_some() {
                SignIn::KeyFile
            } else {
                SignIn::Password
            }),
        }
    }

    async fn echo(port: u16, text: &[u8]) -> Vec<u8> {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        stream.write_all(text).await.unwrap();
        let mut back = vec![0; text.len()];
        stream.read_exact(&mut back).await.unwrap();
        back
    }

    #[tokio::test]
    async fn tunnel_forwards_the_local_port_to_the_server_two_at_once() {
        let server = TestServer::start().await;
        let fixture = Fixture::new(&server).await;
        fixture.secrets.set_password("dev", PASSWORD);
        let port = pick_local_port().unwrap();
        fixture.tunnels.open(spec(&server, None, port));
        fixture.until("dev", |state| state.on).await;
        let (one, two) = tokio::join!(echo(port, b"hello"), echo(port, b"world"));
        assert_eq!(
            (one.as_slice(), two.as_slice()),
            (&b"hello"[..], &b"world"[..])
        );
        fixture.tunnels.close("dev");
        // The abort lands on the tunnel's task a moment later, which frees the port.
        let mut freed = false;
        for _ in 0..40 {
            if TcpStream::connect(("127.0.0.1", port)).await.is_err() {
                freed = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(freed, "the local port closes with its tunnel");
    }

    #[tokio::test]
    async fn tunnel_a_dropped_session_opens_again_on_the_same_port_asking_nothing() {
        let server = TestServer::start().await;
        let fixture = Fixture::new(&server).await;
        let key = server.key_file(Some("open sesame"));
        fixture.secrets.set_passphrase("dev", "open sesame");
        let port = pick_local_port().unwrap();
        fixture.tunnels.open(spec(&server, Some(key.path()), port));
        fixture.until("dev", |state| state.on).await;
        server.drop_sessions();
        fixture.until("dev", |state| !state.on).await;
        fixture.until("dev", |state| state.on).await;
        assert_eq!(echo(port, b"again").await, b"again");
    }

    #[tokio::test]
    async fn tunnel_a_local_port_taken_is_its_error_and_no_other_port_is_used() {
        let server = TestServer::start().await;
        let fixture = Fixture::new(&server).await;
        fixture.secrets.set_password("dev", PASSWORD);
        let taken = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = taken.local_addr().unwrap().port();
        fixture.tunnels.open(spec(&server, None, port));
        let state = fixture.until("dev", |state| state.error.is_some()).await;
        assert!(state
            .error
            .unwrap()
            .contains(&format!("local port {port} is taken")));
        assert!(!state.on);
    }

    /// By hand, on the test server Softov names: a tunnel to its own SSH port
    /// reads the server's banner, which needs nothing installed there.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_tunnel() {
        let (target, _) = crate::ssh::tests::real_target();
        let trusted = std::env::var("AHP_DESK_SSH_TEST_KEY").unwrap();
        let scratch = Scratch::new();
        let known = scratch.known();
        let Err(crate::ssh::ConnectError::NewKey { offered }) =
            crate::ssh::connect(&target, known.checker(&target.address, target.port)).await
        else {
            panic!("the server's key was already known");
        };
        assert_eq!(
            offered.sha256, trusted,
            "only the key Softov matched is trusted"
        );
        known
            .confirm(&target.address, target.port, &offered.sha256)
            .unwrap();
        let tunnels = Tunnels::new(known, Arc::new(Secrets::default()), Arc::new(|_| {}));
        let port = pick_local_port().unwrap();
        tunnels.open(TunnelSpec {
            id: "real".into(),
            target,
            local_port: port,
            remote_port: 22,
            kept_password: None,
            signed_in_before: None,
        });
        for _ in 0..200 {
            if tunnels
                .states()
                .get("real")
                .is_some_and(|state| state.on || state.error.is_some())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(tunnels.states()["real"].on, "{:?}", tunnels.states());
        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let mut banner = [0u8; 8];
        stream.read_exact(&mut banner).await.unwrap();
        println!("{}", String::from_utf8_lossy(&banner));
        assert_eq!(&banner, b"SSH-2.0-");
        tunnels.close_all();
    }

    #[tokio::test]
    async fn tunnel_a_changed_key_is_in_its_state_and_forget_then_trust_opens_it() {
        let server = TestServer::start().await;
        let fixture = Fixture::new(&server).await;
        fixture.secrets.set_password("dev", PASSWORD);
        server.change_host_key();
        let port = pick_local_port().unwrap();
        fixture.tunnels.open(spec(&server, None, port));
        let state = fixture.until("dev", |state| state.key.is_some()).await;
        let Some(crate::ssh::ConnectError::ChangedKey { in_known_hosts, .. }) = state.key else {
            panic!("not a changed key: {state:?}");
        };
        assert!(!in_known_hosts && !state.on && state.needs.is_none());
        let target = server.target(None);
        fixture.known.forget(&target.address, target.port).unwrap();
        fixture.tunnels.retry("dev");
        let state = fixture
            .until("dev", |state| {
                matches!(state.key, Some(crate::ssh::ConnectError::NewKey { .. }))
            })
            .await;
        let Some(crate::ssh::ConnectError::NewKey { offered }) = state.key else {
            unreachable!()
        };
        fixture
            .known
            .confirm(&target.address, target.port, &offered.sha256)
            .unwrap();
        fixture.tunnels.retry("dev");
        let state = fixture.until("dev", |state| state.on).await;
        assert!(state.key.is_none() && state.error.is_none());
        assert_eq!(echo(port, b"trusted").await, b"trusted");
    }

    #[tokio::test]
    async fn tunnel_after_a_restart_needs_the_passphrase_and_sign_in_opens_it() {
        let server = TestServer::start().await;
        let fixture = Fixture::new(&server).await;
        let key = server.key_file(Some("open sesame"));
        let port = pick_local_port().unwrap();
        fixture.tunnels.open(spec(&server, Some(key.path()), port));
        let state = fixture.until("dev", |state| state.needs.is_some()).await;
        assert_eq!(state.needs, Some(Need::Passphrase));
        assert!(fixture.tunnels.sign_in("dev", "open sesame").is_ok());
        fixture.until("dev", |state| state.on).await;
        assert_eq!(echo(port, b"signed").await, b"signed");
    }
}
