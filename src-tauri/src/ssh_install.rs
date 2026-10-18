//! What the desk does on a server over SSH: the probe first, which tells
//! whether the server already has an ahpd the desk uses, and, when it has
//! none, whether it can take ahpd as a user service and where its Node comes
//! from.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::hosts::HostStore;
use crate::local_ahpd::{version_of_text, OLDEST_SUPPORTED};
use crate::ssh::known_keys::KnownKeys;
use crate::ssh::{self, exec_capture, quote, ConnectError, Keys, Session, SignInError, SshTarget};

/// The oldest Node the desk's ahpd runs on.
const NODE_MAJOR: u64 = 22;

/// Where the desk installs ahpd on a server, under the user's home.
pub const INSTALL_DIR: &str = ".local/share/ahpd";

/// The first line of the probe script, which also names it to a test server.
const PROBE_MARK: &str = "# ahp-desk probe";
/// The first line of the script that asks a found ahpd its plugins and state.
const FOUND_MARK: &str = "# ahp-desk found";

/// One POSIX `sh` script that answers the probe in `key=value` lines, so a
/// server answers in one round trip; it runs after `PROBE_MARK`.
const PROBE_SCRIPT: &str = r#"say() { printf '%s=%s\n' "$1" "$2"; }
say os "$(uname -s 2>/dev/null)"
if [ -r /etc/os-release ]; then say pretty "$(. /etc/os-release 2>/dev/null; printf %s "${PRETTY_NAME:-}")"; fi
say arch "$(uname -m 2>/dev/null)"
if ldd --version 2>&1 | grep -qi musl; then say libc musl; else say libc glibc; fi
say home "$HOME"
say user "$(id -un 2>/dev/null)"
run="/run/user/$(id -u)"
if systemctl --user show-environment >/dev/null 2>&1; then say systemd yes
elif [ -d "$run" ] && XDG_RUNTIME_DIR="$run" systemctl --user show-environment >/dev/null 2>&1; then say systemd yes; say runtimeDir "$run"
else say systemd no; fi
say linger "$(loginctl show-user "$(id -un)" -p Linger --value 2>/dev/null)"
login() { "${SHELL:-sh}" -lc "$1" 2>/dev/null | tail -n 1; }
node=$(command -v node 2>/dev/null)
[ -n "$node" ] || node=$(login 'command -v node')
if [ -n "$node" ]; then
  say node "$node"
  say nodeVersion "$("$node" -v 2>/dev/null)"
  [ -x "$(dirname "$node")/npm" ] && say npm "$(dirname "$node")/npm"
fi
if command -v curl >/dev/null 2>&1; then say fetch curl; elif command -v wget >/dev/null 2>&1; then say fetch wget; fi
command -v tar >/dev/null 2>&1 && say tar yes
unit="$HOME/.config/systemd/user/ahpd.service"
[ -f "$unit" ] && grep -q '^Description=.*ahp-desk' "$unit" && say deskUnit yes
desk="$HOME/.local/share/ahpd"
if [ -f "$desk/node_modules/.bin/ahpd" ]; then
  dnode=$(sed -n 's/^ExecStart=\([^ ]*\).*/\1/p' "$unit" 2>/dev/null | head -n 1)
  [ -x "$dnode" ] || dnode="$desk/node/bin/node"
  [ -x "$dnode" ] || dnode="$node"
  v=$("$dnode" "$desk/node_modules/.bin/ahpd" --version 2>/dev/null)
  if [ -n "$v" ]; then say ahpd "$desk/node_modules/.bin/ahpd"; say ahpdNode "$dnode"; say ahpdVersion "$v"; say ahpdDesk yes; exit 0; fi
fi
a=$(command -v ahpd 2>/dev/null)
if [ -n "$a" ]; then v=$("$a" --version 2>/dev/null); if [ -n "$v" ]; then say ahpd "$a"; say ahpdVersion "$v"; exit 0; fi; fi
a=$(login 'command -v ahpd')
if [ -n "$a" ]; then v=$(login "$a --version"); if [ -n "$v" ]; then say ahpd "$a"; say ahpdVersion "$v"; say ahpdLogin yes; fi; fi
exit 0
"#;

/// A CPU nodejs.org builds Node for, named as its downloads are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Cpu {
    X64,
    Arm64,
    Armv7l,
}

impl Cpu {
    fn of(arch: &str) -> Option<Cpu> {
        match arch {
            "x86_64" | "amd64" => Some(Cpu::X64),
            "aarch64" | "arm64" => Some(Cpu::Arm64),
            "armv7l" => Some(Cpu::Armv7l),
            _ => None,
        }
    }
}

/// Where the Node of an ahpd the desk installs comes from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NodeOnServer {
    /// A Node 22 or later already on the server.
    #[serde(rename_all = "camelCase")]
    Own {
        path: String,
        version: String,
        npm: Option<String>,
    },
    /// None, or an older one: the newest 22 is downloaded for this CPU.
    Download { cpu: Cpu },
    /// None, and the download cannot work here.
    Refused { reason: String },
}

/// An ahpd already on the server, which the desk uses and never installs over.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundOnServer {
    /// Its `ahpd` program, or the desk's install's script.
    pub path: String,
    /// The Node the desk's install runs with; none for an `ahpd` that runs itself.
    pub node: Option<String>,
    /// Found through the login shell, so it runs there too.
    pub login_shell: bool,
    pub version: String,
    /// Not older than the oldest ahpd the desk drives.
    pub supported: bool,
    /// Its plugins that are ready.
    pub plugins: Vec<String>,
    pub running: bool,
    pub url: Option<String>,
    /// The desk's own install, with the desk's unit.
    pub by_desk: bool,
}

/// What the probe found, and whether the desk can go on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    pub os: String,
    pub pretty: Option<String>,
    pub arch: String,
    pub home: String,
    pub user: String,
    /// `systemctl --user` answers, maybe only with `runtime_dir`.
    pub systemd: bool,
    /// The `XDG_RUNTIME_DIR` every `systemctl --user` and `journalctl --user`
    /// runs with, when the session has no user bus of its own.
    pub runtime_dir: Option<String>,
    pub linger: bool,
    pub node: NodeOnServer,
    pub found: Option<FoundOnServer>,
    /// Why the desk cannot use or install ahpd here; none when it can.
    pub refusal: Option<String>,
    /// What the person should know, without stopping.
    pub warnings: Vec<String>,
}

/// The probe of a server from what the probe script printed. A found ahpd
/// has no plugins or state yet; `parse_found` gives them.
pub fn parse_probe(text: &str) -> Probe {
    let said: HashMap<&str, &str> = text
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim(), value.trim()))
        .collect();
    let get = |key: &str| said.get(key).copied().filter(|value| !value.is_empty());
    let yes = |key: &str| get(key) == Some("yes");
    let os = get("os").unwrap_or("unknown").to_string();
    let arch = get("arch").unwrap_or("unknown").to_string();
    let user = get("user").unwrap_or_default().to_string();
    let systemd = yes("systemd");
    let linger = get("linger") == Some("yes");

    let node = node_on_server(
        get("node"),
        get("nodeVersion"),
        get("npm"),
        &arch,
        get("libc") == Some("musl"),
        get("fetch").is_some(),
        yes("tar"),
    );

    let found = match (get("ahpd"), get("ahpdVersion")) {
        (Some(path), Some(version)) if version_of_text(version).is_some() => {
            let desk = yes("ahpdDesk");
            Some(FoundOnServer {
                path: path.to_string(),
                node: desk.then(|| get("ahpdNode").unwrap_or_default().to_string()),
                login_shell: yes("ahpdLogin"),
                version: version.to_string(),
                supported: version_of_text(version).is_some_and(|v| v >= OLDEST_SUPPORTED),
                plugins: Vec::new(),
                running: false,
                url: None,
                by_desk: desk && yes("deskUnit"),
            })
        }
        // One that does not give a version, as an install cancelled half-way, is not found.
        _ => None,
    };

    let mut refusal = None;
    if os != "Linux" {
        refusal = Some(format!(
            "The desk installs ahpd on Linux servers only, and this one runs {os}."
        ));
    } else if let Some(found) = &found {
        if !found.supported {
            let (major, minor, patch) = OLDEST_SUPPORTED;
            refusal = Some(format!(
                "The ahpd on this server is {}; the desk needs {major}.{minor}.{patch} or later. \
                 Update it on the server and try again.",
                found.version
            ));
        }
    } else if !systemd {
        refusal = Some(format!(
            "systemctl --user does not answer for {user} here, and ahpd runs as a user service."
        ));
    } else if let NodeOnServer::Refused { reason } = &node {
        refusal = Some(reason.clone());
    }

    let mut warnings = Vec::new();
    match &found {
        None if !linger => warnings.push(format!(
            "Lingering is off for {user}, so ahpd stops when {user} logs out. \
             An administrator turns it on with: sudo loginctl enable-linger {user}"
        )),
        Some(found) if !found.by_desk => warnings.push(
            "This ahpd has no service, so it will not come back after the server restarts."
                .to_string(),
        ),
        _ => {}
    }

    Probe {
        os,
        pretty: get("pretty").map(str::to_string),
        arch,
        home: get("home").unwrap_or_default().to_string(),
        user,
        systemd,
        runtime_dir: get("runtimeDir").map(str::to_string),
        linger,
        node,
        found,
        refusal,
        warnings,
    }
}

fn node_on_server(
    path: Option<&str>,
    version: Option<&str>,
    npm: Option<&str>,
    arch: &str,
    musl: bool,
    fetch: bool,
    tar: bool,
) -> NodeOnServer {
    if let (Some(path), Some(version)) = (path, version) {
        if node_version(version).is_some_and(|major| major >= NODE_MAJOR) {
            return NodeOnServer::Own {
                path: path.to_string(),
                version: version.to_string(),
                npm: npm.map(str::to_string),
            };
        }
    }
    let have = match version {
        Some(version) => format!("Node {version}, older than {NODE_MAJOR},"),
        None => "no Node".to_string(),
    };
    let reason = |why: &str| {
        NodeOnServer::Refused {
        reason: format!("This server has {have} and {why}. Install Node {NODE_MAJOR} or later on it and try again."),
    }
    };
    let Some(cpu) = Cpu::of(arch) else {
        return reason(&format!("nodejs.org has no Node build for its {arch} CPU"));
    };
    if musl {
        return reason("uses musl, for which nodejs.org has no official Node build");
    }
    if !fetch {
        return reason("has neither curl nor wget to download one");
    }
    if !tar {
        return reason("has no tar to unpack one");
    }
    NodeOnServer::Download { cpu }
}

/// The major version of a `node -v` line, as `v22.11.0`.
pub fn node_version(text: &str) -> Option<u64> {
    text.trim()
        .trim_start_matches('v')
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// What a found ahpd's `plugin list --json` and `status --json` said.
#[derive(Debug, PartialEq, Eq)]
pub struct FoundState {
    pub plugins: Vec<String>,
    pub running: bool,
    pub url: Option<String>,
}

/// The two answers of the found script, each after its marker line.
pub fn parse_found(text: &str) -> FoundState {
    let section = |name: &str| {
        let marker = format!("--{name}--");
        text.split(marker.as_str())
            .nth(1)
            .map(|rest| rest.split("\n--").next().unwrap_or(rest).trim().to_string())
            .unwrap_or_default()
    };
    let plugins = serde_json::from_str::<Vec<serde_json::Value>>(&section("plugins"))
        .unwrap_or_default()
        .iter()
        .filter(|plugin| plugin["state"] == "ready")
        .filter_map(|plugin| plugin["name"].as_str().map(str::to_string))
        .collect();
    let status = serde_json::from_str::<serde_json::Value>(&section("status")).unwrap_or_default();
    let url = status["url"].as_str().map(str::to_string);
    FoundState {
        plugins,
        running: status["running"] != serde_json::Value::Bool(false) && url.is_some(),
        url,
    }
}

/// How to run the found ahpd in a remote command, as one quoted line.
pub fn ahpd_command(found: &FoundOnServer, home: &str, args: &str) -> String {
    match (&found.node, found.login_shell) {
        (Some(node), _) => format!(
            "AHPD_PLUGIN_ROOT={} {} {} {args}",
            quote(&format!("{home}/{INSTALL_DIR}")),
            quote(node),
            quote(&found.path)
        ),
        (None, true) => format!(
            "\"${{SHELL:-sh}}\" -lc {}",
            quote(&format!("{} {args}", quote(&found.path)))
        ),
        (None, false) => format!("{} {args}", quote(&found.path)),
    }
}

/// Probes the server: one command, and one more when an ahpd is found.
pub async fn probe(session: &Session) -> Result<Probe, String> {
    let (_, text) = exec_capture(
        session,
        &format!(
            "{PROBE_MARK}
{PROBE_SCRIPT}"
        ),
    )
    .await?;
    let mut probe = parse_probe(&text);
    if let Some(found) = probe.found.as_mut() {
        let script = format!(
            "{FOUND_MARK}\nprintf '%s\\n' --plugins--\n{} 2>/dev/null\nprintf '\\n%s\\n' --status--\n{} 2>/dev/null\nexit 0\n",
            ahpd_command(found, &probe.home, "plugin list --json"),
            ahpd_command(found, &probe.home, "status --json"),
        );
        let (_, text) = exec_capture(session, &script).await?;
        let state = parse_found(&text);
        found.plugins = state.plugins;
        found.running = state.running;
        found.url = state.url;
    }
    Ok(probe)
}

/// The server as the form names it, with what the person typed for it.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeTarget {
    /// The host's id, under which what was typed is held.
    pub id: String,
    pub address: String,
    #[serde(default = "ssh_port")]
    pub port: u16,
    pub user: String,
    #[serde(default)]
    pub key_file: Option<String>,
}

fn ssh_port() -> u16 {
    22
}

/// Signs in to the server the form names and probes it. A typed passphrase or
/// password is held until the desk quits, as task 01 holds them.
#[tauri::command]
pub async fn probe_ssh_server(
    store: State<'_, HostStore>,
    known: State<'_, KnownKeys>,
    target: ProbeTarget,
    passphrase: Option<String>,
    password: Option<String>,
) -> Result<Probe, SignInError> {
    if let Some(passphrase) = passphrase.filter(|typed| !typed.is_empty()) {
        store.secrets().set_passphrase(&target.id, &passphrase);
    }
    if let Some(password) = password.filter(|typed| !typed.is_empty()) {
        store.secrets().set_password(&target.id, &password);
    }
    let saved = store.ssh(&target.id).ok().flatten();
    let ssh_target = SshTarget::new(
        &target.address,
        target.port,
        &target.user,
        target.key_file.map(Into::into),
    );
    let keys = Keys {
        id: &target.id,
        secrets: store.secrets(),
        kept_password: store.ssh_password(&target.id).ok().flatten(),
        signed_in_before: saved.and_then(|ssh| ssh.sign_in),
    };
    let (session, _) = ssh::open(
        &ssh_target,
        known.checker(&ssh_target.address, ssh_target.port),
        keys,
    )
    .await?;
    probe(&session)
        .await
        .map_err(|why| SignInError::Connect(ConnectError::Failed { why }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::known_keys::KeyCheck;
    use crate::ssh::test_server::{TestServer, PASSWORD};
    use crate::ssh::{Secrets, SignIn};

    const DEBIAN: &str = "os=Linux\npretty=Debian GNU/Linux 12 (bookworm)\narch=x86_64\nlibc=glibc\nhome=/home/ana\nuser=ana\nsystemd=yes\nlinger=yes\nnode=/usr/bin/node\nnodeVersion=v22.11.0\nnpm=/usr/bin/npm\nfetch=curl\ntar=yes\n";

    #[test]
    fn probe_debian_with_node_22_and_lingering_uses_its_own_node() {
        let probe = parse_probe(DEBIAN);
        assert_eq!(probe.refusal, None);
        assert!(probe.warnings.is_empty());
        assert_eq!(
            probe.pretty.as_deref(),
            Some("Debian GNU/Linux 12 (bookworm)")
        );
        assert_eq!(
            probe.node,
            NodeOnServer::Own {
                path: "/usr/bin/node".into(),
                version: "v22.11.0".into(),
                npm: Some("/usr/bin/npm".into())
            }
        );
        assert_eq!(probe.found, None);
    }

    #[test]
    fn probe_ubuntu_with_no_node_and_lingering_off_downloads_and_warns() {
        let probe = parse_probe("os=Linux\narch=x86_64\nlibc=glibc\nhome=/home/bo\nuser=bo\nsystemd=yes\nlinger=no\nfetch=wget\ntar=yes\n");
        assert_eq!(probe.refusal, None);
        assert_eq!(probe.node, NodeOnServer::Download { cpu: Cpu::X64 });
        assert!(probe.warnings[0].contains("sudo loginctl enable-linger bo"));
    }

    #[test]
    fn probe_a_node_20_is_replaced_by_a_download() {
        let probe = parse_probe(&DEBIAN.replace("v22.11.0", "v20.18.1"));
        assert_eq!(probe.node, NodeOnServer::Download { cpu: Cpu::X64 });
    }

    #[test]
    fn probe_alpine_is_refused_without_node_and_fine_with_one() {
        let alpine = "os=Linux\narch=x86_64\nlibc=musl\nhome=/home/a\nuser=a\nsystemd=yes\nlinger=yes\nfetch=wget\ntar=yes\n";
        let probe = parse_probe(alpine);
        assert!(probe.refusal.unwrap().contains("musl"));
        let with_node = format!("{alpine}node=/usr/bin/node\nnodeVersion=v22.1.0\n");
        assert_eq!(parse_probe(&with_node).refusal, None);
    }

    #[test]
    fn probe_arm64_downloads_for_arm64_and_an_unknown_cpu_is_refused() {
        let arm = DEBIAN
            .replace("x86_64", "aarch64")
            .replace("nodeVersion=v22.11.0\n", "");
        assert_eq!(
            parse_probe(&arm).node,
            NodeOnServer::Download { cpu: Cpu::Arm64 }
        );
        let riscv = arm.replace("aarch64", "riscv64");
        assert!(parse_probe(&riscv).refusal.unwrap().contains("riscv64"));
    }

    #[test]
    fn probe_macos_is_refused() {
        let probe = parse_probe(&DEBIAN.replace("os=Linux", "os=Darwin"));
        assert!(probe.refusal.unwrap().contains("Darwin"));
    }

    #[test]
    fn probe_no_systemctl_user_is_refused_and_a_runtime_dir_is_kept() {
        let probe = parse_probe(&DEBIAN.replace("systemd=yes", "systemd=no"));
        assert!(probe.refusal.unwrap().contains("systemctl --user"));
        let probe = parse_probe(&format!("{DEBIAN}runtimeDir=/run/user/1000\n"));
        assert_eq!(probe.refusal, None);
        assert_eq!(probe.runtime_dir.as_deref(), Some("/run/user/1000"));
    }

    #[test]
    fn probe_an_ahpd_on_path_is_used_and_lifts_the_install_refusals() {
        let text = DEBIAN
            .replace("systemd=yes", "systemd=no")
            .replace("x86_64", "riscv64")
            + "ahpd=/usr/local/bin/ahpd\nahpdVersion=0.12.0\n";
        let probe = parse_probe(&text);
        assert_eq!(probe.refusal, None);
        let found = probe.found.unwrap();
        assert_eq!(
            (found.path.as_str(), found.version.as_str()),
            ("/usr/local/bin/ahpd", "0.12.0")
        );
        assert!(found.supported && !found.by_desk && found.node.is_none());
        assert!(probe.warnings[0].contains("will not come back"));
    }

    #[test]
    fn probe_the_desks_own_install_with_its_unit_is_by_desk() {
        let text = format!("{DEBIAN}deskUnit=yes\nahpd=/home/ana/.local/share/ahpd/node_modules/.bin/ahpd\nahpdNode=/home/ana/.local/share/ahpd/node/bin/node\nahpdVersion=0.12.0\nahpdDesk=yes\n");
        let probe = parse_probe(&text);
        let found = probe.found.unwrap();
        assert!(found.by_desk);
        assert_eq!(
            found.node.as_deref(),
            Some("/home/ana/.local/share/ahpd/node/bin/node")
        );
        assert!(probe.warnings.is_empty());
        assert!(ahpd_command(&found, "/home/ana", "status --json")
            .starts_with("AHPD_PLUGIN_ROOT=/home/ana/.local/share/ahpd /home/ana/.local/share/ahpd/node/bin/node "));
    }

    #[test]
    fn probe_an_install_that_gives_no_version_is_not_found() {
        let probe = parse_probe(&format!(
            "{DEBIAN}ahpd=/home/ana/.local/share/ahpd/node_modules/.bin/ahpd\nahpdVersion=\n"
        ));
        assert_eq!(probe.found, None);
        let garbled = parse_probe(&format!(
            "{DEBIAN}ahpd=/x/ahpd\nahpdVersion=Error: Cannot find module\n"
        ));
        assert_eq!(garbled.found, None);
    }

    #[test]
    fn probe_an_old_ahpd_is_shown_not_replaced() {
        let probe = parse_probe(&format!("{DEBIAN}ahpd=/usr/bin/ahpd\nahpdVersion=0.7.0\n"));
        assert!(!probe.found.unwrap().supported);
        assert!(probe.refusal.unwrap().contains("0.7.0"));
    }

    #[test]
    fn probe_a_found_ahpd_through_the_login_shell_runs_there() {
        let probe = parse_probe(&format!(
            "{DEBIAN}ahpd=/home/ana/.nvm/bin/ahpd\nahpdVersion=0.12.0\nahpdLogin=yes\n"
        ));
        let found = probe.found.unwrap();
        assert_eq!(
            ahpd_command(&found, "/home/ana", "--version"),
            "\"${SHELL:-sh}\" -lc '/home/ana/.nvm/bin/ahpd --version'"
        );
    }

    #[test]
    fn probe_found_reads_a_running_ahpd_with_its_ready_plugins() {
        let state = parse_found("--plugins--\n[{\"name\":\"@ahpd/agent-claude\",\"state\":\"ready\"},{\"name\":\"@ahpd/x\",\"state\":\"failed\"}]\n\n--status--\n{\"pid\":7,\"url\":\"ws://127.0.0.1:9187\"}\n");
        assert_eq!(
            state,
            FoundState {
                plugins: vec!["@ahpd/agent-claude".into()],
                running: true,
                url: Some("ws://127.0.0.1:9187".into())
            }
        );
    }

    #[test]
    fn probe_found_reads_a_stopped_ahpd_with_no_plugin() {
        let state = parse_found("--plugins--\n[]\n\n--status--\n{\"running\":false}\n");
        assert_eq!(
            state,
            FoundState {
                plugins: vec![],
                running: false,
                url: None
            }
        );
    }

    #[test]
    fn probe_node_version_reads_the_major() {
        assert_eq!(node_version("v22.11.0"), Some(22));
        assert_eq!(node_version("v8.0.0\n"), Some(8));
        assert_eq!(node_version("command not found"), None);
    }

    async fn signed_in(server: &TestServer) -> Session {
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let keys = Keys {
            id: "dev",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let (session, way) = ssh::open(&server.target(None), Box::new(|_| KeyCheck::Known), keys)
            .await
            .unwrap();
        assert_eq!(way, SignIn::Password);
        session
    }

    #[tokio::test]
    async fn probe_sends_one_command_and_a_second_only_for_a_found_ahpd() {
        let server = TestServer::start().await;
        server.answer(PROBE_MARK, DEBIAN, 0);
        let session = signed_in(&server).await;
        let first = probe(&session).await.unwrap();
        assert_eq!(first.found, None);
        assert_eq!(server.commands().len(), 1);

        let found = TestServer::start().await;
        found.answer(
            PROBE_MARK,
            &format!("{DEBIAN}ahpd=/usr/bin/ahpd\nahpdVersion=0.12.0\n"),
            0,
        );
        found.answer(
            FOUND_MARK,
            "--plugins--\n[{\"name\":\"@ahpd/agent-claude\",\"state\":\"ready\"}]\n\n--status--\n{\"running\":false}\n",
            0,
        );
        let session = signed_in(&found).await;
        let second = probe(&session).await.unwrap();
        let ahpd = second.found.unwrap();
        assert_eq!(ahpd.plugins, ["@ahpd/agent-claude"]);
        assert!(!ahpd.running);
        let commands = found.commands();
        assert_eq!(commands.len(), 2);
        assert!(
            commands[1].contains("'/usr/bin/ahpd' plugin list --json")
                || commands[1].contains("/usr/bin/ahpd plugin list --json")
        );
    }

    /// By hand, on the test server Softov names, as `ssh_a_real_sign_in`; prints its probe.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_probe() {
        let (target, check) = crate::ssh::tests::real_target();
        let secrets = Secrets::default();
        let keys = Keys {
            id: "real",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let (session, _) = ssh::open(&target, check, keys).await.unwrap();
        let probe = probe(&session).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&probe).unwrap());
    }
}
