//! What the desk does on a server over SSH: the probe first, which tells
//! whether the server already has an ahpd the desk uses, and, when it has
//! none, whether it can take ahpd as a user service and where its Node comes
//! from.

// `node()` waits for its caller, the install (task 05). Drop this then.
#![cfg_attr(not(test), allow(dead_code))]

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::hosts::HostStore;
use crate::local_ahpd::{version_of_text, OLDEST_SUPPORTED};
use crate::ssh::known_keys::KnownKeys;
use crate::ssh::{
    self, exec_capture, exec_lines, quote, ConnectError, Keys, Session, SignInError, SshTarget,
};

/// The oldest Node the desk's ahpd runs on.
const NODE_MAJOR: u64 = 22;

/// Where a server downloads its Node: the newest 22 release, not the version
/// the desk pins for itself (decision `the-remote-node-download-is-the-newest-22`).
pub const NODE_DIST: &str = "https://nodejs.org/dist/latest-v22.x/";

/// How many of a failed script's last lines its error carries.
const LAST_LINES: usize = 5;

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

/// The tool a server downloads with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Fetch {
    Curl,
    Wget,
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
    /// `curl`, else `wget`, whichever the server has.
    pub fetch: Option<Fetch>,
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
    let fetch = match get("fetch") {
        Some("curl") => Some(Fetch::Curl),
        Some("wget") => Some(Fetch::Wget),
        _ => None,
    };

    let node = node_on_server(
        get("node"),
        get("nodeVersion"),
        get("npm"),
        &arch,
        get("libc") == Some("musl"),
        fetch.is_some(),
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
        fetch,
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

/// The Node an ahpd the desk installs runs on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteNode {
    pub node: String,
    pub npm: String,
    /// The folder of both, which the unit puts first on `PATH`.
    pub bin_dir: String,
    /// `node -v`, which the install's steps show, since two installs can get
    /// different 22 releases.
    pub version: String,
}

/// The `sh` script that downloads the newest Node 22 for `cpu` from `dist`
/// into `<install_dir>/node`, checked against the release's `SHASUMS256.txt`;
/// an older Node there is replaced only once the new one is whole.
pub fn download_script(install_dir: &str, cpu: Cpu, fetch: Fetch, dist: &str) -> String {
    let cpu = match cpu {
        Cpu::X64 => "x64",
        Cpu::Arm64 => "arm64",
        Cpu::Armv7l => "armv7l",
    };
    let get = match fetch {
        Fetch::Curl => r#"curl -fsSL "$1" -o "$2""#,
        Fetch::Wget => r#"wget -q -O "$2" "$1""#,
    };
    format!(
        r#"set -eu
dir={dir}
dist={dist}
mkdir -p "$dir"
work=$(mktemp -d "$dir/.node-XXXXXX")
trap 'rm -rf "$work"' EXIT
get() {{ {get}; }}
echo "Reading the newest Node {major} from $dist"
get "${{dist}}SHASUMS256.txt" "$work/SHASUMS256.txt" || {{ echo "Cannot download ${{dist}}SHASUMS256.txt; does this server reach the internet?" >&2; exit 10; }}
grep -E '  node-v{major}\.[0-9]+\.[0-9]+-linux-{cpu}\.tar\.gz$' "$work/SHASUMS256.txt" > "$work/line" || true
count=$(wc -l < "$work/line" | tr -d ' ')
if [ "$count" != 1 ]; then echo "SHASUMS256.txt names $count Node {major} archives for linux-{cpu}; it holds:" >&2; cat "$work/SHASUMS256.txt" >&2; exit 11; fi
name=$(awk '{{print $2}}' "$work/line")
echo "Downloading $name"
get "$dist$name" "$work/$name" || {{ echo "Cannot download $dist$name" >&2; exit 12; }}
if ! (cd "$work" && sha256sum -c line >/dev/null 2>&1); then rm -f "$work/$name"; echo "$name does not match its SHA-256 in SHASUMS256.txt, and was deleted" >&2; exit 13; fi
echo "Unpacking $name"
mkdir "$work/node"
tar -xzf "$work/$name" -C "$work/node" --strip-components=1
rm -rf "$dir/node.old"
if [ -d "$dir/node" ]; then mv "$dir/node" "$dir/node.old"; fi
if ! mv "$work/node" "$dir/node"; then [ -d "$dir/node.old" ] && mv "$dir/node.old" "$dir/node"; exit 14; fi
rm -rf "$dir/node.old"
"#,
        dir = quote(install_dir),
        dist = quote(dist),
        major = NODE_MAJOR,
    )
}

/// The Node the remote ahpd runs on: the server's own, or the newest 22
/// downloaded into `install_dir`, each line of the download sent to `line`.
pub async fn node(
    session: &Session,
    probe: &Probe,
    install_dir: &str,
    line: impl Fn(String),
    cancel: &AtomicBool,
) -> Result<RemoteNode, String> {
    let cpu = match &probe.node {
        NodeOnServer::Own { path, version, npm } => {
            let bin_dir = path
                .rsplit_once('/')
                .map_or(".", |(dir, _)| dir)
                .to_string();
            return Ok(RemoteNode {
                npm: npm.clone().unwrap_or_else(|| format!("{bin_dir}/npm")),
                node: path.clone(),
                bin_dir,
                version: version.clone(),
            });
        }
        NodeOnServer::Refused { reason } => return Err(reason.clone()),
        NodeOnServer::Download { cpu } => *cpu,
    };
    let fetch = probe
        .fetch
        .ok_or("this server has neither curl nor wget to download Node")?;
    // The error carries the last lines, where the script says why it stopped.
    let last = Mutex::new(VecDeque::new());
    let script = download_script(install_dir, cpu, fetch, NODE_DIST);
    let ran = exec_lines(
        session,
        &script,
        |text| {
            let mut kept = last.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if kept.len() == LAST_LINES {
                kept.pop_front();
            }
            kept.push_back(text.clone());
            line(text);
        },
        cancel,
    )
    .await;
    if let Err(err) = ran {
        let kept = last
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let said = kept.into_iter().collect::<Vec<_>>().join("\n");
        return Err(format!("the Node download stopped ({err}): {said}"));
    }
    let bin_dir = format!("{install_dir}/node/bin");
    let node = format!("{bin_dir}/node");
    let (code, version) = exec_capture(session, &format!("{} -v", quote(&node))).await?;
    let version = version.trim().to_string();
    if code != 0 || !node_version(&version).is_some_and(|major| major >= NODE_MAJOR) {
        return Err(format!(
            "the downloaded Node answered {version:?}, not a Node {NODE_MAJOR}"
        ));
    }
    Ok(RemoteNode {
        npm: format!("{bin_dir}/npm"),
        node,
        bin_dir,
        version,
    })
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

    #[test]
    fn node_the_download_script_fetches_only_from_latest_22_and_checks_before_unpacking() {
        for (cpu, name) in [
            (Cpu::X64, "x64"),
            (Cpu::Arm64, "arm64"),
            (Cpu::Armv7l, "armv7l"),
        ] {
            for (fetch, tool) in [(Fetch::Curl, "curl -fsSL"), (Fetch::Wget, "wget -q")] {
                let script = download_script("/home/a b/.local/share/ahpd", cpu, fetch, NODE_DIST);
                assert!(
                    script.contains("dir='/home/a b/.local/share/ahpd'"),
                    "{script}"
                );
                assert!(script.contains(&format!("dist='{NODE_DIST}'")));
                assert!(script.contains(&format!(
                    "node-v22\\.[0-9]+\\.[0-9]+-linux-{name}\\.tar\\.gz$"
                )));
                assert!(script.contains(tool));
                assert_eq!(
                    script.matches("get \"").count(),
                    2,
                    "only SHASUMS256.txt and its archive"
                );
                let check = script.find("sha256sum -c").unwrap();
                assert!(check < script.find("tar -xzf").unwrap());
            }
        }
    }

    /// `sh` on `PATH`, or Git for Windows' own.
    fn sh() -> std::path::PathBuf {
        let on_path = std::process::Command::new("sh")
            .arg("-c")
            .arg("exit 0")
            .status();
        if on_path.is_ok_and(|status| status.success()) {
            return "sh".into();
        }
        let git = std::path::PathBuf::from(r"C:\Program Files\Git\bin\sh.exe");
        assert!(git.is_file(), "this test runs the download script in sh: install Git for Windows, or run it where sh is on PATH");
        git
    }

    /// A scratch dist folder with a fake Node archive and its SHASUMS256.txt,
    /// served to the script as a `file://` URL, and an install folder.
    struct Dist(std::path::PathBuf);
    impl Dist {
        fn new() -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "ahp-desk-node-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(dir.join("dist")).unwrap();
            Dist(dir)
        }
        /// Runs a shell line in the scratch folder.
        fn sh(&self, line: &str) -> std::process::Output {
            std::process::Command::new(sh())
                .arg("-c")
                .arg(line)
                .current_dir(&self.0)
                .output()
                .unwrap()
        }
        fn archive(&self, version: &str, cpu: &str) -> String {
            let name = format!("node-{version}-linux-{cpu}.tar.gz");
            let made = self.sh(&format!(
                "mkdir -p pkg/node-{version}/bin && printf '#!/bin/sh\\necho {version}\\n' > pkg/node-{version}/bin/node && chmod +x pkg/node-{version}/bin/node && tar -czf dist/{name} -C pkg node-{version}"
            ));
            assert!(made.status.success(), "{made:?}");
            name
        }
        fn sums(&self, lines: &str) {
            std::fs::write(self.0.join("dist/SHASUMS256.txt"), lines).unwrap();
        }
        fn sum_of(&self, name: &str) -> String {
            let out = self.sh(&format!("sha256sum dist/{name}"));
            String::from_utf8_lossy(&out.stdout)
                .split(' ')
                .next()
                .unwrap()
                .to_string()
        }
        /// Runs the download script for x64 with curl from this dist.
        fn run(&self) -> std::process::Output {
            // Git Bash's `pwd -W` gives C:/..., which curl opens; elsewhere `pwd` does.
            let url = self
                .sh("cd dist && p=$(pwd -W 2>/dev/null || pwd) && printf 'file:///%s/' \"${p#/}\"");
            let url = String::from_utf8_lossy(&url.stdout).to_string();
            let install = self.sh("printf '%s/install' \"$(pwd)\"");
            let install = String::from_utf8_lossy(&install.stdout).to_string();
            self.sh(&download_script(&install, Cpu::X64, Fetch::Curl, &url))
        }
    }
    impl Drop for Dist {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn node_the_download_script_in_sh_picks_the_cpus_line_and_replaces_the_old_node_whole() {
        let dist = Dist::new();
        let x64 = dist.archive("v22.11.0", "x64");
        let arm = dist.archive("v22.11.0", "arm64");
        dist.sums(&format!(
            "{}  {x64}\n{}  {arm}\n{}  node-v22.11.0-linux-x64.tar.xz\n",
            dist.sum_of(&x64),
            dist.sum_of(&arm),
            "0".repeat(64)
        ));
        let ran = dist.run();
        assert!(ran.status.success(), "{ran:?}");
        let said = String::from_utf8_lossy(&ran.stdout);
        assert!(said.contains(&format!("Downloading {x64}")), "{said}");
        let node = dist.sh("install/node/bin/node");
        assert_eq!(String::from_utf8_lossy(&node.stdout).trim(), "v22.11.0");

        // A second run replaces it whole, and leaves no work folder behind.
        let newer = dist.archive("v22.12.0", "x64");
        dist.sums(&format!("{}  {newer}\n", dist.sum_of(&newer)));
        assert!(dist.run().status.success());
        let node = dist.sh("install/node/bin/node");
        assert_eq!(String::from_utf8_lossy(&node.stdout).trim(), "v22.12.0");
        let left = dist.sh("ls -A install");
        assert_eq!(String::from_utf8_lossy(&left.stdout).trim(), "node");
    }

    #[test]
    fn node_the_download_script_in_sh_refuses_a_missing_line_and_deletes_a_wrong_sum() {
        let dist = Dist::new();
        let arm = dist.archive("v22.11.0", "arm64");
        dist.sums(&format!("{}  {arm}\n", dist.sum_of(&arm)));
        let ran = dist.run();
        assert_eq!(ran.status.code(), Some(11));
        assert!(
            String::from_utf8_lossy(&ran.stderr).contains("names 0 Node 22 archives for linux-x64")
        );

        let x64 = dist.archive("v22.11.0", "x64");
        dist.sums(&format!("{}  {x64}\n", "a".repeat(64)));
        let ran = dist.run();
        assert_eq!(ran.status.code(), Some(13), "{ran:?}");
        assert!(String::from_utf8_lossy(&ran.stderr).contains("was deleted"));
        let left = dist.sh("ls -A install");
        assert_eq!(
            String::from_utf8_lossy(&left.stdout).trim(),
            "",
            "no archive and no node left"
        );
    }

    #[tokio::test]
    async fn node_the_servers_own_sends_nothing_and_a_failed_download_says_its_last_lines() {
        let server = TestServer::start().await;
        let session = signed_in(&server).await;
        let cancel = AtomicBool::new(false);
        let own = parse_probe(DEBIAN);
        let found = node(
            &session,
            &own,
            "/home/ana/.local/share/ahpd",
            |_| {},
            &cancel,
        )
        .await
        .unwrap();
        assert_eq!(found.node, "/usr/bin/node");
        assert_eq!(found.bin_dir, "/usr/bin");
        assert_eq!(found.version, "v22.11.0");
        assert!(
            server.commands().is_empty(),
            "the server's own Node downloads nothing"
        );

        server.answer(
            "set -eu",
            "Reading the newest Node 22\nCannot download SHASUMS256.txt\n",
            10,
        );
        let none = parse_probe(&DEBIAN.replace("nodeVersion=v22.11.0\n", ""));
        let failed = node(
            &session,
            &none,
            "/home/ana/.local/share/ahpd",
            |_| {},
            &cancel,
        )
        .await
        .unwrap_err();
        assert!(failed.contains("exited with 10"), "{failed}");
        assert!(
            failed.contains("Cannot download SHASUMS256.txt"),
            "{failed}"
        );
    }

    /// By hand, on the test server Softov names: downloads the newest Node 22
    /// into `~/.cache/ahp-desk-test-node`, twice, and removes it after.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_node_download() {
        let (target, check) = crate::ssh::tests::real_target();
        let secrets = Secrets::default();
        let keys = Keys {
            id: "real",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let (session, _) = ssh::open(&target, check, keys).await.unwrap();
        let mut probed = probe(&session).await.unwrap();
        let cpu = Cpu::of(&probed.arch).unwrap();
        probed.node = NodeOnServer::Download { cpu };
        let scratch = format!("{}/.cache/ahp-desk-test-node", probed.home);
        let cancel = AtomicBool::new(false);
        for _ in 0..2 {
            let got = node(
                &session,
                &probed,
                &scratch,
                |text| println!("{text}"),
                &cancel,
            )
            .await
            .unwrap();
            println!("{}", got.version);
            assert_eq!(node_version(&got.version), Some(22));
        }
        let (code, _) = exec_capture(&session, &format!("rm -rf {}", quote(&scratch)))
            .await
            .unwrap();
        assert_eq!(code, 0);
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
