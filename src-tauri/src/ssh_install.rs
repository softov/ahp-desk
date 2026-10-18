//! What the desk does on a server over SSH: the probe first, which tells
//! whether the server already has an ahpd the desk uses, and, when it has
//! none, whether it can take ahpd as a user service and where its Node comes
//! from.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::State;

use crate::hosts::{HostStore, Reach};
use crate::local_ahpd::{
    check_backends, ready_specs, spec_of, token_in_connect_url, version_of_text, InstallEvent,
    StepStatus, OLDEST_SUPPORTED, QUIET_NPM,
};
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
/// The user unit the desk writes for the ahpd it installs, under the user's home.
pub const UNIT_FILE: &str = ".config/systemd/user/ahpd.service";
/// Where a long step on an ahpd the desk did not install keeps its pid file.
const STEP_DIR: &str = ".cache/ahp-desk";
/// The port an ahpd listens on unless the form names another.
pub const AHPD_PORT: u16 = 9187;
/// How many lines of the journal a `ready` that timed out shows.
const JOURNAL_LINES: usize = 40;

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
[ -d "${XDG_CONFIG_HOME:-$HOME/.config}/ahpd" ] && say ahpdConfig "${XDG_CONFIG_HOME:-$HOME/.config}/ahpd"
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
    /// How the sign-in went through, which the host keeps, so a tunnel after a
    /// restart knows what to ask; set by `probe_ssh_server`.
    pub sign_in: Option<crate::ssh::SignIn>,
    /// An ahpd config already on the server with no ahpd found, from one set
    /// up before: the desk's ahpd uses it, and a plugin there that the newest
    /// ahpd cannot load stops the install.
    pub ahpd_config: Option<String>,
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

    let ahpd_config = match &found {
        None => get("ahpdConfig").map(str::to_string),
        Some(_) => None,
    };

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
        sign_in: None,
        ahpd_config,
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
    let plugins = ready_specs(
        &serde_json::from_str::<Vec<serde_json::Value>>(&section("plugins")).unwrap_or_default(),
    );
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
        // `plugin install` runs npm, which the Node's own folder holds.
        (Some(node), _) => format!(
            "AHPD_PLUGIN_ROOT={} PATH={}:\"$PATH\" {} {} {args}",
            quote(&format!("{home}/{INSTALL_DIR}")),
            quote(node.rsplit_once('/').map_or(".", |(dir, _)| dir)),
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
    let script = download_script(install_dir, cpu, fetch, NODE_DIST);
    run_step(session, "node", install_dir, &script, &line, cancel)
        .await
        .map_err(|err| format!("the Node download stopped: {err}"))?;
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

/// `body` as the remote command of step `id`, named by its first line.
fn script(id: &str, body: &str) -> String {
    format!("# ahp-desk {id}\n{body}")
}

/// Runs a long step in a session of its own, its pid in `<dir>/.step.pid`,
/// so a cancel stops it and every process it started. Its lines go to `line`;
/// a failure carries its last lines.
async fn run_step(
    session: &Session,
    id: &str,
    dir: &str,
    body: &str,
    line: &impl Fn(String),
    cancel: &AtomicBool,
) -> Result<(), String> {
    let pid = format!("{dir}/.step.pid");
    let command = script(
        id,
        &format!(
            "mkdir -p {}\nexec setsid -w sh -c {}",
            quote(dir),
            quote(&format!("echo $$ > {}\n{body}", quote(&pid)))
        ),
    );
    let last = Mutex::new(VecDeque::new());
    let ran = exec_lines(
        session,
        &command,
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
    match ran {
        Ok(()) => Ok(()),
        Err(err) if err == "cancelled" => {
            // The channel is closed; the step's own session is killed over a new one.
            let kill = format!(
                "pid=$(cat {pid} 2>/dev/null) && kill -TERM -- \"-$pid\" 2>/dev/null; rm -f {pid}",
                pid = quote(&pid)
            );
            let _ = exec_capture(session, &script("cancel", &kill)).await;
            Err(err)
        }
        Err(err) => {
            let kept = last
                .into_inner()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let said = kept.into_iter().collect::<Vec<_>>().join("\n");
            Err(if said.is_empty() {
                err
            } else {
                format!("{err}: {said}")
            })
        }
    }
}

/// Runs a short step's command and fails with what it said when it exits non-zero.
async fn run_short(session: &Session, id: &str, body: &str) -> Result<String, String> {
    let (code, said) = exec_capture(session, &script(id, body)).await?;
    if code == 0 {
        Ok(said)
    } else {
        Err(format!("{id} exited with {code}: {}", said.trim()))
    }
}

/// The backends asked for that a `plugin list --json` does not have ready,
/// each with its state and ahpd's reason.
pub fn not_ready(list: &[serde_json::Value], backends: &[String]) -> Vec<String> {
    backends
        .iter()
        .filter_map(|backend| {
            let row = list
                .iter()
                .find(|row| spec_of(row).as_deref() == Some(backend));
            match row {
                Some(row) if row["state"] == "ready" => None,
                Some(row) => Some(format!(
                    "{backend} ({}: {})",
                    row["state"].as_str().unwrap_or("not ready"),
                    row["problem"].as_str().unwrap_or("no reason given")
                )),
                None => Some(format!("{backend} (not listed)")),
            }
        })
        .collect()
}

/// After `plugin install`, every backend asked for must be ready in the
/// ahpd that will run; a plugin of an older ahpd in its config folder, which
/// it loads first, is the usual reason one is not. The desk never edits that
/// folder, so it stops here and says what to do.
async fn check_ready(
    session: &Session,
    ahpd: &FoundOnServer,
    home: &str,
    backends: &[String],
) -> Result<(), String> {
    let said = run_short(
        session,
        "plugins",
        &ahpd_command(ahpd, home, "plugin list --json"),
    )
    .await?;
    let list: Vec<serde_json::Value> = serde_json::from_str(said.trim()).map_err(|_| {
        format!(
            "ahpd plugin list --json did not answer a list: {}",
            said.trim()
        )
    })?;
    let missing = not_ready(&list, backends);
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "ahpd cannot load {}. Its config folder on the server (~/.config/ahpd) holds plugins it loads first; \
         update or remove them there, then try again.",
        missing.join(", ")
    ))
}

/// Where the desk's ahpd listens: only on the server, reached through the
/// tunnel, or on every address, reached on its open port with its token.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bind {
    #[default]
    Loopback,
    All,
}

/// `s` as one word of a systemd unit line: double-quoted when it needs to be,
/// and `%` written `%%`, which systemd would read as a specifier.
fn unit_word(s: &str) -> String {
    let s = s.replace('%', "%%");
    if s.chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '\\' || c == '\'')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s
    }
}

/// The user unit of the desk's ahpd: `ahpd run` on the ahpd port, with its
/// token file and its backends under the install folder, in the user's home.
pub fn render_unit(node: &str, script: &str, prefix: &str, bind: Bind, port: u16) -> String {
    let host = match bind {
        Bind::Loopback => "127.0.0.1",
        Bind::All => "0.0.0.0",
    };
    let node_bin = node.rsplit_once('/').map_or(".", |(dir, _)| dir);
    format!(
        "[Unit]
Description=ahpd, installed by ahp-desk
After=network-online.target

[Service]
Type=simple
WorkingDirectory=%h
ExecStart={node} {script} run --host {host} --port {port} --connection-token-file {token}
Environment={root}
Environment={path}
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
",
        node = unit_word(node),
        script = unit_word(script),
        token = unit_word(&format!("{prefix}/connection-token")),
        root = unit_word(&format!("AHPD_PLUGIN_ROOT={prefix}")),
        path = unit_word(&format!("PATH={node_bin}:/usr/local/bin:/usr/bin:/bin")),
    )
}

/// `systemctl --user <args>`, with the probe's `XDG_RUNTIME_DIR` when the
/// session has no user bus of its own.
pub fn systemctl_user(runtime_dir: Option<&str>, args: &str) -> String {
    match runtime_dir {
        Some(dir) => format!("XDG_RUNTIME_DIR={} systemctl --user {args}", quote(dir)),
        None => format!("systemctl --user {args}"),
    }
}

/// `journalctl --user` for the desk's unit, with the same runtime folder.
fn journal_user(runtime_dir: Option<&str>) -> String {
    let args = format!("-u ahpd.service -n {JOURNAL_LINES} --no-pager");
    match runtime_dir {
        Some(dir) => format!("XDG_RUNTIME_DIR={} journalctl --user {args}", quote(dir)),
        None => format!("journalctl --user {args}"),
    }
}

/// Asks `http://127.0.0.1:<port>/` with the server's Node every 500 ms for up
/// to 30 s; ahpd answers a plain GET with 426, which counts.
fn ready_script(node: &str, port: u16) -> String {
    let ask = "const http=require('http');const port=+process.argv[1];const end=Date.now()+30000;\
               (function ask(){http.get({host:'127.0.0.1',port,path:'/'},r=>{r.resume();console.log('answered '+r.statusCode);process.exit(0)})\
               .on('error',()=>{if(Date.now()>end){console.log('no answer on '+port);process.exit(1)}setTimeout(ask,500)})})()";
    format!("{} -e {} {port}", quote(node), quote(ask))
}

/// What Add host asks for an install over SSH.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshInstall {
    pub target: ProbeTarget,
    /// The backends ticked, as `@ahpd/` names.
    pub backends: Vec<String>,
    /// The folder ahpd serves; the user's home on the server when none.
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default = "ahpd_port")]
    pub ahpd_port: u16,
    /// The tunnel keeps ahpd on the server's 127.0.0.1; an open port puts it on every address.
    #[serde(default)]
    pub reach: Reach,
}

impl SshInstall {
    pub fn bind(&self) -> Bind {
        match self.reach {
            Reach::Tunnel => Bind::Loopback,
            Reach::Port => Bind::All,
        }
    }
}

fn ahpd_port() -> u16 {
    AHPD_PORT
}

/// The ahpd the desk connects to after an install or with a found one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshInstalled {
    /// Its connection token, when it takes one; never in a log line.
    pub token: Option<String>,
    pub version: String,
    /// The desk's install folder, on an ahpd the desk installed.
    pub prefix: Option<String>,
    pub node: Option<String>,
    pub ahpd_port: u16,
    pub by_desk: bool,
}

/// One install at a time, and its cancel.
#[derive(Default)]
pub struct InstallControl {
    running: AtomicBool,
    cancel: AtomicBool,
}

/// Holds the one install while it runs.
pub struct Installing<'a>(&'a InstallControl);

impl InstallControl {
    pub fn begin(&self) -> Result<Installing<'_>, String> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err("an install over SSH is already running".into());
        }
        self.cancel.store(false, Ordering::SeqCst);
        Ok(Installing(self))
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    /// The flag a running install or update watches for its cancel.
    pub fn cancelled(&self) -> &AtomicBool {
        &self.cancel
    }
}

impl Drop for Installing<'_> {
    fn drop(&mut self) {
        self.0.running.store(false, Ordering::SeqCst);
    }
}

/// Sends each step as it starts and ends, and stops before a step once cancelled.
struct Steps<'a> {
    send: &'a (dyn Fn(InstallEvent) + Sync),
    cancel: &'a AtomicBool,
}

impl Steps<'_> {
    fn line(&self, text: String) {
        (self.send)(InstallEvent::Line { text });
    }

    fn status(&self, id: &str, status: StepStatus) {
        (self.send)(InstallEvent::Step {
            id: id.into(),
            status,
        });
    }

    async fn run<T>(
        &self,
        id: &str,
        work: impl std::future::Future<Output = Result<T, String>>,
    ) -> Result<T, String> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        self.status(id, StepStatus::Running);
        let done = work.await;
        self.status(
            id,
            if done.is_ok() {
                StepStatus::Done
            } else {
                StepStatus::Failed
            },
        );
        done
    }

    fn skip(&self, id: &str) -> Result<(), String> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        self.status(id, StepStatus::Skipped);
        Ok(())
    }
}

/// Connects, probes, and installs ahpd as a user service, or uses the one
/// found; each step sent to `send` as it starts and ends.
pub async fn install(
    target: &SshTarget,
    check: crate::ssh::ServerCheck,
    keys: Keys<'_>,
    request: &SshInstall,
    send: &(dyn Fn(InstallEvent) + Sync),
    cancel: &AtomicBool,
) -> Result<SshInstalled, String> {
    let steps = Steps { send, cancel };
    let session = steps
        .run("connect", async {
            ssh::open(target, check, keys)
                .await
                .map(|(session, _)| session)
                .map_err(|err| err.to_string())
        })
        .await?;
    let probed = steps
        .run("probe", async {
            let probed = probe(&session).await?;
            match &probed.refusal {
                Some(refusal) => Err(refusal.clone()),
                None => Ok(probed),
            }
        })
        .await?;
    match probed.found.clone() {
        Some(found) => use_found(&session, &probed, found, request, &steps).await,
        None => install_new(&session, &probed, request, &steps).await,
    }
}

async fn install_new(
    session: &Session,
    probed: &Probe,
    request: &SshInstall,
    steps: &Steps<'_>,
) -> Result<SshInstalled, String> {
    check_backends(&request.backends)?;
    let prefix = format!("{}/{INSTALL_DIR}", probed.home);
    let line = |text| steps.line(text);
    let node = steps
        .run("node", node(session, probed, &prefix, line, steps.cancel))
        .await?;
    steps.line(format!("Node {}", node.version));
    let quiet = QUIET_NPM
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ");
    steps
        .run(
            "server",
            server_step(session, &prefix, &node.bin_dir, &node.npm, "latest", steps),
        )
        .await?;
    let ahpd = FoundOnServer {
        path: format!("{prefix}/node_modules/.bin/ahpd"),
        node: Some(node.node.clone()),
        login_shell: false,
        version: String::new(),
        supported: true,
        plugins: Vec::new(),
        running: false,
        url: None,
        by_desk: true,
    };
    let names = request
        .backends
        .iter()
        .map(|name| quote(name))
        .collect::<Vec<_>>()
        .join(" ");
    steps
        .run("backends", async {
            run_step(
                session,
                "backends",
                &prefix,
                &format!(
                    "{quiet} {}",
                    ahpd_command(&ahpd, &probed.home, &format!("plugin install {names}"))
                ),
                &line,
                steps.cancel,
            )
            .await?;
            check_ready(session, &ahpd, &probed.home, &request.backends).await
        })
        .await?;
    let folder = request
        .folder
        .clone()
        .unwrap_or_else(|| probed.home.clone());
    steps
        .run(
            "folder",
            run_short(
                session,
                "folder",
                &ahpd_command(
                    &ahpd,
                    &probed.home,
                    &format!("paths add {}", quote(&folder)),
                ),
            ),
        )
        .await?;
    let unit = render_unit(
        &node.node,
        &ahpd.path,
        &prefix,
        request.bind(),
        request.ahpd_port,
    );
    let runtime = probed.runtime_dir.as_deref();
    let unit_file = format!("{}/{UNIT_FILE}", probed.home);
    steps
        .run(
            "unit",
            run_short(
                session,
                "unit",
                &format!(
                    "mkdir -p {dir}\ncat > {file} <<'AHP_DESK_UNIT'\n{unit}AHP_DESK_UNIT\n{reload}",
                    dir = quote(unit_file.rsplit_once('/').map_or(".", |(dir, _)| dir)),
                    file = quote(&unit_file),
                    reload = systemctl_user(runtime, "daemon-reload"),
                ),
            ),
        )
        .await?;
    steps
        .run(
            "start",
            run_short(
                session,
                "start",
                &format!(
                    "{} && {}",
                    systemctl_user(runtime, "enable ahpd.service"),
                    systemctl_user(runtime, "restart ahpd.service")
                ),
            ),
        )
        .await?;
    let (token, version) = steps
        .run(
            "ready",
            ready_for_desk(session, probed, &node.node, &ahpd, request.ahpd_port, steps),
        )
        .await?;
    Ok(SshInstalled {
        token,
        version,
        prefix: Some(prefix),
        node: Some(node.node),
        ahpd_port: request.ahpd_port,
        by_desk: true,
    })
}

/// Waits for the desk's ahpd to answer on `port`, then reads its token from
/// its file and its version; a timeout shows the journal and names the port.
async fn ready_for_desk(
    session: &Session,
    probed: &Probe,
    node: &str,
    ahpd: &FoundOnServer,
    port: u16,
    steps: &Steps<'_>,
) -> Result<(Option<String>, String), String> {
    let (code, said) = exec_capture(session, &script("ready", &ready_script(node, port))).await?;
    if code != 0 {
        let (_, journal) = exec_capture(
            session,
            &script("journal", &journal_user(probed.runtime_dir.as_deref())),
        )
        .await?;
        for text in journal.lines() {
            steps.line(text.to_string());
        }
        return Err(format!(
            "ahpd did not answer on port {port} within 30 s; another program may hold that port on the server ({})",
            said.trim()
        ));
    }
    let prefix = format!("{}/{INSTALL_DIR}", probed.home);
    let token = run_short(
        session,
        "token",
        &format!("cat {}", quote(&format!("{prefix}/connection-token"))),
    )
    .await?
    .trim()
    .to_string();
    let version = run_short(
        session,
        "version",
        &ahpd_command(ahpd, &probed.home, "--version"),
    )
    .await?
    .trim()
    .to_string();
    Ok(((!token.is_empty()).then_some(token), version))
}

/// Uses an ahpd already on the server: adds the backends it lacks, starts it
/// when it is off, and reads how to reach it. Nothing is installed.
async fn use_found(
    session: &Session,
    probed: &Probe,
    found: FoundOnServer,
    request: &SshInstall,
    steps: &Steps<'_>,
) -> Result<SshInstalled, String> {
    let missing: Vec<String> = request
        .backends
        .iter()
        .filter(|name| !found.plugins.contains(name))
        .cloned()
        .collect();
    let line = |text| steps.line(text);
    if missing.is_empty() {
        steps.skip("backends")?;
    } else {
        check_backends(&missing)?;
        let names = missing
            .iter()
            .map(|name| quote(name))
            .collect::<Vec<_>>()
            .join(" ");
        let quiet = QUIET_NPM
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(" ");
        // The step's pid file: in the desk's install for its own ahpd, else in a cache
        // folder, so an ahpd the desk did not install gets nothing beside it.
        let dir = if found.by_desk {
            format!("{}/{INSTALL_DIR}", probed.home)
        } else {
            format!("{}/{STEP_DIR}", probed.home)
        };
        steps
            .run("backends", async {
                run_step(
                    session,
                    "backends",
                    &dir,
                    &format!(
                        "{quiet} {}",
                        ahpd_command(&found, &probed.home, &format!("plugin install {names}"))
                    ),
                    &line,
                    steps.cancel,
                )
                .await?;
                check_ready(session, &found, &probed.home, &missing).await
            })
            .await?;
    }
    let runtime = probed.runtime_dir.as_deref();
    if found.by_desk {
        // The desk's own install: its unit, restarted so new backends load.
        steps
            .run(
                "start",
                run_short(
                    session,
                    "start",
                    &format!(
                        "{} && {}",
                        systemctl_user(runtime, "enable ahpd.service"),
                        systemctl_user(runtime, "restart ahpd.service")
                    ),
                ),
            )
            .await?;
        let unit_file = format!("{}/{UNIT_FILE}", probed.home);
        let port = run_short(
            session,
            "port",
            &format!(
                "sed -n 's/.*--port \\([0-9]*\\).*/\\1/p' {}",
                quote(&unit_file)
            ),
        )
        .await?
        .trim()
        .parse()
        .unwrap_or(AHPD_PORT);
        let node = found.node.clone().unwrap_or_default();
        let (token, version) = steps
            .run(
                "ready",
                ready_for_desk(session, probed, &node, &found, port, steps),
            )
            .await?;
        return Ok(SshInstalled {
            token,
            version,
            prefix: Some(format!("{}/{INSTALL_DIR}", probed.home)),
            node: found.node,
            ahpd_port: port,
            by_desk: true,
        });
    }
    // An ahpd the desk did not install: its own start or restart, and no unit.
    if !found.running {
        steps
            .run(
                "start",
                run_short(
                    session,
                    "start",
                    &ahpd_command(&found, &probed.home, "start"),
                ),
            )
            .await?;
    } else if !missing.is_empty() {
        steps
            .run(
                "start",
                run_short(
                    session,
                    "start",
                    &ahpd_command(&found, &probed.home, "restart"),
                ),
            )
            .await?;
    } else {
        steps.skip("start")?;
    }
    let (port, token) = steps
        .run("ready", async {
            let status = ahpd_command(&found, &probed.home, "status --json");
            let mut url = None;
            for _ in 0..60 {
                let said = run_short(session, "ready", &status)
                    .await
                    .unwrap_or_default();
                let answer: serde_json::Value =
                    serde_json::from_str(said.trim()).unwrap_or_default();
                if let Some(found_url) = answer["url"].as_str() {
                    url = Some(found_url.to_string());
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            let url = url.ok_or("the ahpd on the server did not say it is running within 30 s")?;
            let port = tauri::Url::parse(&url)
                .ok()
                .and_then(|parsed| parsed.port())
                .ok_or_else(|| format!("the ahpd on the server gave no port in {url}"))?;
            let record = run_short(
                session,
                "token",
                "cat \"${XDG_CONFIG_HOME:-$HOME/.config}/ahpd/daemon.json\" 2>/dev/null || true",
            )
            .await?;
            let token = serde_json::from_str::<serde_json::Value>(record.trim())
                .ok()
                .and_then(|record| token_in_connect_url(&record));
            Ok((port, token))
        })
        .await?;
    Ok(SshInstalled {
        token,
        version: found.version,
        prefix: None,
        node: found.node,
        ahpd_port: port,
        by_desk: false,
    })
}

/// Installs `@ahpd/server@<version>` into the desk's install folder with the
/// npm next to the Node in `bin_dir`.
async fn server_step(
    session: &Session,
    prefix: &str,
    bin_dir: &str,
    npm: &str,
    version: &str,
    steps: &Steps<'_>,
) -> Result<(), String> {
    let quiet = QUIET_NPM
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ");
    run_step(
        session,
        "server",
        prefix,
        &format!(
            "{quiet} PATH={}:\"$PATH\" {} install --prefix {} {}",
            quote(bin_dir),
            quote(npm),
            quote(prefix),
            quote(&format!("@ahpd/server@{version}"))
        ),
        &|text| steps.line(text),
        steps.cancel,
    )
    .await
}

/// Updates the ahpd the desk installed over SSH to `version` (`latest`, or
/// an npm version): connects, finds the desk's install and the Node its unit
/// runs, installs the version beside the same backends, restarts the unit and
/// waits for it on `port`. The unit, the token, the folder and the backends
/// stay, so the host's address and token still hold. Answers the version now
/// running.
pub async fn update(
    target: &SshTarget,
    check: crate::ssh::ServerCheck,
    keys: Keys<'_>,
    version: &str,
    port: u16,
    send: &(dyn Fn(InstallEvent) + Sync),
    cancel: &AtomicBool,
) -> Result<String, String> {
    // A version is a word npm reads; anything else never reaches a shell.
    if version.is_empty()
        || !version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-+".contains(c))
    {
        return Err(format!("{version:?} is not an npm version"));
    }
    let steps = Steps { send, cancel };
    let session = steps
        .run("connect", async {
            ssh::open(target, check, keys)
                .await
                .map(|(session, _)| session)
                .map_err(|err| err.to_string())
        })
        .await?;
    let (probed, found, node) = steps
        .run("probe", async {
            let probed = probe(&session).await?;
            match probed.found.clone() {
                Some(found) if found.by_desk => {
                    let node = found.node.clone().filter(|node| !node.is_empty()).ok_or(
                        "the desk's ahpd unit names no Node in its ExecStart".to_string(),
                    )?;
                    Ok((probed, found, node))
                }
                Some(found) => Err(format!(
                    "the ahpd on the server is {} at {}, not the desk's install, so it updates through ahpd",
                    found.version, found.path
                )),
                None => Err(format!(
                    "the desk's install is not on the server: no ahpd under ~/{INSTALL_DIR} with its unit"
                )),
            }
        })
        .await?;
    let prefix = format!("{}/{INSTALL_DIR}", probed.home);
    let bin_dir = node
        .rsplit_once('/')
        .map_or(".", |(dir, _)| dir)
        .to_string();
    let npm = format!("{bin_dir}/npm");
    steps
        .run(
            "server",
            server_step(&session, &prefix, &bin_dir, &npm, version, &steps),
        )
        .await?;
    steps
        .run(
            "start",
            run_short(
                &session,
                "start",
                &systemctl_user(probed.runtime_dir.as_deref(), "restart ahpd.service"),
            ),
        )
        .await?;
    let (_, now) = steps
        .run(
            "ready",
            ready_for_desk(&session, &probed, &node, &found, port, &steps),
        )
        .await?;
    Ok(now)
}

/// `update` for host `id`, signed in as the host signs in; refused before any
/// connection for a host not reached over SSH or whose ahpd the desk did not
/// install.
pub async fn update_host(
    store: &HostStore,
    known: &KnownKeys,
    id: &str,
    version: &str,
    send: &(dyn Fn(InstallEvent) + Sync),
    cancel: &AtomicBool,
) -> Result<String, String> {
    let ssh = store
        .ssh(id)?
        .ok_or_else(|| format!("{id} is not reached over SSH"))?;
    if !ssh.by_desk {
        return Err(format!(
            "the desk did not install the ahpd of {id}, so it updates through ahpd"
        ));
    }
    let target = SshTarget::new(
        &ssh.address,
        ssh.port,
        &ssh.user,
        ssh.key_file.clone().map(Into::into),
    );
    let keys = Keys {
        id,
        secrets: store.secrets(),
        kept_password: store.ssh_password(id).ok().flatten(),
        signed_in_before: ssh.sign_in,
    };
    update(
        &target,
        known.checker(&target.address, target.port),
        keys,
        version,
        ssh.remote_port,
        send,
        cancel,
    )
    .await
}

/// The exit code of an uninstall step that found nothing to do.
const NOTHING_THERE: i32 = 100;

/// The uninstall's steps after `connect`, each with its command: `stop`
/// disables the unit, `unit` removes it, `files` removes the desk's install.
/// A step that finds nothing exits `NOTHING_THERE`. `~/.config/ahpd` is never
/// named. A home that is not an absolute folder is refused, so a wrong home
/// never removes another folder.
pub fn uninstall_script(
    home: &str,
    runtime_dir: Option<&str>,
) -> Result<Vec<(&'static str, String)>, String> {
    if !home.starts_with('/') || home.trim_end_matches('/').is_empty() || home.contains('\n') {
        return Err(format!("{home:?} is not a home folder to uninstall from"));
    }
    let home = home.trim_end_matches('/');
    let unit = quote(&format!("{home}/{UNIT_FILE}"));
    let prefix = quote(&format!("{home}/{INSTALL_DIR}"));
    Ok(vec![
        (
            "stop",
            format!(
                "test -f {unit} || exit {NOTHING_THERE}\n{}",
                systemctl_user(runtime_dir, "disable --now ahpd.service")
            ),
        ),
        (
            "unit",
            format!(
                "test -f {unit} || exit {NOTHING_THERE}\nrm -f {unit} && {}",
                systemctl_user(runtime_dir, "daemon-reload")
            ),
        ),
        (
            "files",
            format!(
                "test -e {prefix} || exit {NOTHING_THERE}\n\
                 if [ ! -d {prefix}/node_modules/@ahpd/server ] && [ ! -f {prefix}/connection-token ]; then\n\
                 echo {prefix} has no @ahpd/server and no connection-token, so the desk did not install it; exit 1\n\
                 fi\n\
                 rm -rf -- {prefix}"
            ),
        ),
    ])
}

/// Stops and removes the ahpd the desk installed over SSH: its user unit and
/// its install folder, with the token file in it; `~/.config/ahpd` stays.
pub async fn uninstall(
    target: &SshTarget,
    check: crate::ssh::ServerCheck,
    keys: Keys<'_>,
    send: &(dyn Fn(InstallEvent) + Sync),
) -> Result<(), String> {
    // The uninstall has no Cancel: half a removal helps no one.
    let never = AtomicBool::new(false);
    let steps = Steps {
        send,
        cancel: &never,
    };
    let (session, probed) = steps
        .run("connect", async {
            let (session, _) = ssh::open(target, check, keys)
                .await
                .map_err(|err| err.to_string())?;
            let probed = probe(&session).await?;
            Ok((session, probed))
        })
        .await?;
    for (id, body) in uninstall_script(&probed.home, probed.runtime_dir.as_deref())? {
        steps.status(id, StepStatus::Running);
        let ran = exec_capture(&session, &script(id, &body)).await;
        match ran {
            Ok((0, _)) => steps.status(id, StepStatus::Done),
            Ok((NOTHING_THERE, _)) => steps.status(id, StepStatus::Skipped),
            Ok((code, said)) => {
                steps.status(id, StepStatus::Failed);
                return Err(format!("{id} exited with {code}: {}", said.trim()));
            }
            Err(err) => {
                steps.status(id, StepStatus::Failed);
                return Err(err);
            }
        }
    }
    Ok(())
}

/// `uninstall` for host `id`, signed in as the host signs in; refused before
/// any connection for a host not reached over SSH or whose ahpd the desk did
/// not install.
async fn uninstall_host(
    store: &HostStore,
    known: &KnownKeys,
    id: &str,
    send: &(dyn Fn(InstallEvent) + Sync),
) -> Result<(), String> {
    let ssh = store
        .ssh(id)?
        .ok_or_else(|| format!("{id} is not reached over SSH"))?;
    if !ssh.by_desk {
        return Err(format!(
            "the desk did not install the ahpd of {id}, so it leaves it on the server"
        ));
    }
    let target = SshTarget::new(
        &ssh.address,
        ssh.port,
        &ssh.user,
        ssh.key_file.clone().map(Into::into),
    );
    let keys = Keys {
        id,
        secrets: store.secrets(),
        kept_password: store.ssh_password(id).ok().flatten(),
        signed_in_before: ssh.sign_in,
    };
    uninstall(
        &target,
        known.checker(&target.address, target.port),
        keys,
        send,
    )
    .await
}

/// Stops and uninstalls the ahpd the desk installed on host `id`'s server;
/// the shell removes the host once it is done.
#[tauri::command]
pub async fn uninstall_ssh_ahpd(
    store: State<'_, HostStore>,
    known: State<'_, KnownKeys>,
    control: State<'_, InstallControl>,
    id: String,
    events: Channel<InstallEvent>,
) -> Result<(), String> {
    let _held = control.begin()?;
    let send = move |event: InstallEvent| {
        let _ = events.send(event);
    };
    uninstall_host(&store, &known, &id, &send).await
}

/// Installs ahpd on the server the form names, or uses the one found there.
#[tauri::command]
pub async fn install_ssh_ahpd(
    store: State<'_, HostStore>,
    known: State<'_, KnownKeys>,
    control: State<'_, InstallControl>,
    request: SshInstall,
    events: Channel<InstallEvent>,
) -> Result<SshInstalled, String> {
    let _held = control.begin()?;
    let target = &request.target;
    let saved = store.ssh(&target.id).ok().flatten();
    let ssh_target = SshTarget::new(
        &target.address,
        target.port,
        &target.user,
        target.key_file.clone().map(Into::into),
    );
    let keys = Keys {
        id: &target.id,
        secrets: store.secrets(),
        kept_password: store.ssh_password(&target.id).ok().flatten(),
        signed_in_before: saved.and_then(|ssh| ssh.sign_in),
    };
    let send = move |event: InstallEvent| {
        // A form that went away mid-install does not stop it.
        let _ = events.send(event);
    };
    install(
        &ssh_target,
        known.checker(&ssh_target.address, ssh_target.port),
        keys,
        &request,
        &send,
        &control.cancel,
    )
    .await
}

/// Stops the install over SSH between steps, or in the middle of a long one.
#[tauri::command]
pub fn cancel_ssh_install(control: State<'_, InstallControl>) {
    control.cancel();
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
    let (session, way) = ssh::open(
        &ssh_target,
        known.checker(&ssh_target.address, ssh_target.port),
        keys,
    )
    .await?;
    let mut probed = probe(&session)
        .await
        .map_err(|why| SignInError::Connect(ConnectError::Failed { why }))?;
    probed.sign_in = Some(way);
    Ok(probed)
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
        assert!(probe.linger && probe.ahpd_config.is_none());
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
        assert!(!probe.linger);
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
        assert!(ahpd_command(&found, "/home/ana", "status --json").starts_with(
            "AHPD_PLUGIN_ROOT=/home/ana/.local/share/ahpd PATH=/home/ana/.local/share/ahpd/node/bin:\"$PATH\" /home/ana/.local/share/ahpd/node/bin/node "
        ));
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
                assert!(script.contains(&format!(
                    "dist={NODE_DIST}
"
                )));
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
            "# ahp-desk node",
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

    /// By hand, on the test server Softov names, on port 9387: installs with
    /// `@ahpd/agent-claude`, then uses that install adding `@ahpd/bot`; from a
    /// clean server, cancels during `server`, then installs over it. The
    /// server's `~/.config/ahpd` is copied aside first and put back at the end,
    /// with the unit and the install folder removed.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_install() {
        let (target, check) = crate::ssh::tests::real_target();
        let fingerprint = std::env::var("AHP_DESK_SSH_TEST_KEY").unwrap();
        let trusted = move || -> crate::ssh::ServerCheck {
            let fingerprint = fingerprint.clone();
            Box::new(move |key| {
                let offered = crate::ssh::known_keys::fingerprint(key);
                if offered.sha256 == fingerprint {
                    KeyCheck::Known
                } else {
                    KeyCheck::New { offered }
                }
            })
        };
        drop(check);
        let secrets = Secrets::default();
        let keys = || Keys {
            id: "real",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let (session, _) = ssh::open(&target, trusted(), keys()).await.unwrap();
        let shell = |command: String| {
            let session = &session;
            async move {
                let (code, said) = exec_capture(session, &command).await.unwrap();
                println!("$ {command}\n{said}");
                (code, said)
            }
        };
        let (_, home) = shell("printf %s \"$HOME\"".into()).await;
        let backup = format!("{home}/.config/ahpd.desk-test-backup");
        let config = format!("{home}/.config/ahpd");
        let (code, _) = shell(format!(
            "test ! -e {b} && if [ -e {c} ]; then cp -a {c} {b}; fi",
            b = quote(&backup),
            c = quote(&config)
        ))
        .await;
        assert_eq!(
            code, 0,
            "a backup from an earlier run is still there: {backup}"
        );

        let clean = format!(
            "systemctl --user disable --now ahpd.service 2>/dev/null; rm -f {unit}; systemctl --user daemon-reload; rm -rf {prefix}",
            unit = quote(&format!("{home}/{UNIT_FILE}")),
            prefix = quote(&format!("{home}/{INSTALL_DIR}"))
        );
        let restore = format!(
            "rm -rf {c}; if [ -e {b} ]; then mv {b} {c}; fi",
            b = quote(&backup),
            c = quote(&config)
        );
        let mut request = request(&["@ahpd/agent-claude"]);
        request.target = ProbeTarget {
            id: "real".into(),
            address: target.address.clone(),
            port: target.port,
            user: target.user.clone(),
            key_file: target
                .key_file
                .as_ref()
                .map(|path| path.display().to_string()),
        };
        request.ahpd_port = 9387;

        let run = |request: SshInstall, cancel_in_server: bool| {
            let (target, trusted, secrets) = (&target, &trusted, &secrets);
            async move {
                let events = Mutex::new(Vec::new());
                let cancel = AtomicBool::new(false);
                let send = |event: InstallEvent| {
                    match &event {
                        InstallEvent::Line { text } => println!("  {text}"),
                        InstallEvent::Step { id, status } => println!("[{id}] {status:?}"),
                    }
                    events.lock().unwrap().push(event);
                };
                let keys = Keys {
                    id: "real",
                    secrets,
                    kept_password: None,
                    signed_in_before: None,
                };
                let canceller = async {
                    if !cancel_in_server {
                        return;
                    }
                    loop {
                        let in_server = events.lock().unwrap().contains(&InstallEvent::Step {
                            id: "server".into(),
                            status: StepStatus::Running,
                        });
                        if in_server {
                            tokio::time::sleep(Duration::from_secs(3)).await;
                            cancel.store(true, Ordering::SeqCst);
                            return;
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                };
                let (done, ()) = tokio::join!(
                    install(target, trusted(), keys, &request, &send, &cancel),
                    canceller
                );
                (done, events.into_inner().unwrap())
            }
        };

        let first = run(request.clone(), false).await.0;
        let checked = async {
            let installed = first?;
            assert!(installed.by_desk);
            let (_, active) = shell("systemctl --user is-active ahpd.service".into()).await;
            assert_eq!(active.trim(), "active");
            let (_, status) = shell(format!(
                "\"${{SHELL:-sh}}\" -lc {}",
                quote(&format!(
                    "AHPD_PLUGIN_ROOT={home}/{INSTALL_DIR} {home}/{INSTALL_DIR}/node/bin/node {home}/{INSTALL_DIR}/node_modules/.bin/ahpd status --json"
                ))
            ))
            .await;
            println!("status from a login shell: {status}");
            // Read without `shell`, which prints what it ran and what it said.
            let (_, file) = exec_capture(
                &session,
                &format!("cat {home}/{INSTALL_DIR}/connection-token"),
            )
            .await
            .unwrap();
            assert_eq!(installed.token.as_deref(), Some(file.trim()));

            let mut second = request.clone();
            second.backends = vec!["@ahpd/agent-claude".into(), "@ahpd/bot".into()];
            let (again, events) = run(second, false).await;
            assert!(again?.by_desk);
            let ran: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    InstallEvent::Step { id, status: StepStatus::Done } => Some(id.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(ran, ["connect", "probe", "backends", "start", "ready"]);

            shell(clean.clone()).await;
            let (cancelled, _) = run(request.clone(), true).await;
            assert_eq!(cancelled.unwrap_err(), "cancelled");
            let fourth = run(request.clone(), false).await.0?;
            assert!(fourth.by_desk);
            Ok::<(), String>(())
        }
        .await;
        shell(clean).await;
        shell(restore).await;
        checked.unwrap();
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

    #[test]
    fn install_the_unit_runs_ahpd_on_its_port_with_its_token_file_and_plugin_root() {
        let unit = render_unit(
            "/home/ana/.local/share/ahpd/node/bin/node",
            "/home/ana/.local/share/ahpd/node_modules/.bin/ahpd",
            "/home/ana/.local/share/ahpd",
            Bind::Loopback,
            9187,
        );
        assert!(unit.contains("Description=ahpd, installed by ahp-desk\n"));
        assert!(unit.contains("WorkingDirectory=%h\n"));
        assert!(unit.contains("ExecStart=/home/ana/.local/share/ahpd/node/bin/node /home/ana/.local/share/ahpd/node_modules/.bin/ahpd run --host 127.0.0.1 --port 9187 --connection-token-file /home/ana/.local/share/ahpd/connection-token\n"));
        assert!(unit.contains("Environment=AHPD_PLUGIN_ROOT=/home/ana/.local/share/ahpd\n"));
        assert!(unit.contains(
            "Environment=PATH=/home/ana/.local/share/ahpd/node/bin:/usr/local/bin:/usr/bin:/bin\n"
        ));
        assert!(
            unit.contains("Restart=on-failure\n") && unit.contains("WantedBy=default.target\n")
        );
        let open = render_unit("/usr/bin/node", "/s/ahpd", "/p", Bind::All, 9300);
        assert!(open.contains("run --host 0.0.0.0 --port 9300 "));
    }

    #[test]
    fn install_the_unit_quotes_a_home_with_a_space_for_systemd() {
        let unit = render_unit(
            "/home/a b/.local/share/ahpd/node/bin/node",
            "/home/a b/.local/share/ahpd/node_modules/.bin/ahpd",
            "/home/a b/.local/share/ahpd",
            Bind::Loopback,
            9187,
        );
        assert!(unit.contains("ExecStart=\"/home/a b/.local/share/ahpd/node/bin/node\" \"/home/a b/.local/share/ahpd/node_modules/.bin/ahpd\" run"));
        assert!(unit.contains("Environment=\"AHPD_PLUGIN_ROOT=/home/a b/.local/share/ahpd\"\n"));
        assert_eq!(unit_word("50%"), "50%%");
    }

    #[test]
    fn install_systemctl_takes_the_runtime_dir_when_the_probe_needed_one() {
        assert_eq!(
            systemctl_user(None, "daemon-reload"),
            "systemctl --user daemon-reload"
        );
        assert_eq!(
            systemctl_user(Some("/run/user/1000"), "daemon-reload"),
            "XDG_RUNTIME_DIR=/run/user/1000 systemctl --user daemon-reload"
        );
    }

    /// The test server's canned answers for a whole install.
    fn answer_install(server: &TestServer, probe: &str) {
        server.answer(PROBE_MARK, probe, 0);
        for id in ["server", "backends", "folder", "unit", "start", "cancel"] {
            server.answer(&format!("# ahp-desk {id}\n"), "", 0);
        }
        server.answer("# ahp-desk ready\n", "answered 426\n", 0);
        server.answer("# ahp-desk token\n", "the-token\n", 0);
        server.answer("# ahp-desk version\n", "0.12.0\n", 0);
        server.answer("# ahp-desk port\n", "9300\n", 0);
        server.answer(
            "# ahp-desk plugins\n",
            r#"[{"spec":"@ahpd/agent-claude","name":"claude","state":"ready"},{"spec":"@ahpd/bot","name":"bot","state":"ready"}]"#,
            0,
        );
    }

    fn request(backends: &[&str]) -> SshInstall {
        SshInstall {
            target: ProbeTarget {
                id: "dev".into(),
                address: "127.0.0.1".into(),
                port: 22,
                user: "tester".into(),
                key_file: None,
            },
            backends: backends.iter().map(|name| name.to_string()).collect(),
            folder: None,
            ahpd_port: 9300,
            reach: Reach::Tunnel,
        }
    }

    /// Runs an install against the test server and answers its result, the
    /// steps in the order they ran, and the commands the server saw.
    async fn run_install(
        server: &TestServer,
        request: &SshInstall,
        cancel: &AtomicBool,
    ) -> (
        Result<SshInstalled, String>,
        Vec<(String, StepStatus)>,
        Vec<String>,
    ) {
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let keys = Keys {
            id: "dev",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let events = Mutex::new(Vec::new());
        let send = |event: InstallEvent| events.lock().unwrap().push(event);
        let done = install(
            &server.target(None),
            Box::new(|_| KeyCheck::Known),
            keys,
            request,
            &send,
            cancel,
        )
        .await;
        let steps = events
            .into_inner()
            .unwrap()
            .into_iter()
            .filter_map(|event| match event {
                InstallEvent::Step { id, status } if status != StepStatus::Running => {
                    Some((id, status))
                }
                _ => None,
            })
            .collect();
        (done, steps, server.commands())
    }

    fn ids(steps: &[(String, StepStatus)]) -> Vec<&str> {
        steps.iter().map(|(id, _)| id.as_str()).collect()
    }

    #[tokio::test]
    async fn install_on_a_server_with_its_own_node_runs_every_step_with_the_plugin_root() {
        let server = TestServer::start().await;
        answer_install(&server, DEBIAN);
        let cancel = AtomicBool::new(false);
        let (done, steps, commands) =
            run_install(&server, &request(&["@ahpd/agent-claude"]), &cancel).await;
        let installed = done.unwrap();
        assert_eq!(
            ids(&steps),
            [
                "connect", "probe", "node", "server", "backends", "folder", "unit", "start",
                "ready"
            ]
        );
        assert!(steps.iter().all(|(_, status)| *status == StepStatus::Done));
        assert_eq!(
            installed,
            SshInstalled {
                token: Some("the-token".into()),
                version: "0.12.0".into(),
                prefix: Some("/home/ana/.local/share/ahpd".into()),
                node: Some("/usr/bin/node".into()),
                ahpd_port: 9300,
                by_desk: true
            }
        );
        let of = |id: &str| {
            commands
                .iter()
                .find(|command| command.starts_with(&format!("# ahp-desk {id}\n")))
                .unwrap()
                .clone()
        };
        assert!(of("server").contains(
            "/usr/bin/npm install --prefix /home/ana/.local/share/ahpd @ahpd/server@latest"
        ));
        assert!(of("server").contains("setsid -w sh -c"));
        for id in ["backends", "folder", "version"] {
            assert!(
                of(id).contains("AHPD_PLUGIN_ROOT=/home/ana/.local/share/ahpd"),
                "{id}: {}",
                of(id)
            );
        }
        assert!(of("backends").contains("plugin install @ahpd/agent-claude"));
        assert!(of("folder").contains("paths add /home/ana"));
        assert!(of("unit")
            .contains("cat > /home/ana/.config/systemd/user/ahpd.service <<'AHP_DESK_UNIT'"));
        assert!(of("unit").contains("--port 9300"));
        assert!(of("unit").ends_with("systemctl --user daemon-reload"));
        assert!(of("start").contains(
            "systemctl --user enable ahpd.service && systemctl --user restart ahpd.service"
        ));
        assert!(of("ready").contains(" 9300"));
        assert!(
            !commands.iter().any(|command| command.contains("the-token")),
            "the token is only read"
        );
    }

    #[tokio::test]
    async fn install_with_a_found_ahpd_adds_only_the_backends_it_lacks_and_installs_nothing() {
        let server = TestServer::start().await;
        // The first answer that matches wins, so these go in before the install's.
        server.answer(
            FOUND_MARK,
            "--plugins--\n[{\"name\":\"@ahpd/agent-claude\",\"state\":\"ready\"}]\n\n--status--\n{\"pid\":4,\"url\":\"ws://127.0.0.1:9187\"}\n",
            0,
        );
        server.answer(
            "# ahp-desk ready\n/usr/local/bin/ahpd status --json",
            "{\"pid\":4,\"url\":\"ws://127.0.0.1:9187\"}\n",
            0,
        );
        server.answer(
            "# ahp-desk token\ncat",
            "{\"pid\":4,\"connectUrl\":\"ws://127.0.0.1:9187/?tkn=found%2Btoken\"}\n",
            0,
        );
        answer_install(
            &server,
            &format!("{DEBIAN}ahpd=/usr/local/bin/ahpd\nahpdVersion=0.12.0\n"),
        );
        let cancel = AtomicBool::new(false);
        let (done, steps, commands) = run_install(
            &server,
            &request(&["@ahpd/agent-claude", "@ahpd/bot"]),
            &cancel,
        )
        .await;
        let installed = done.unwrap();
        assert_eq!(
            ids(&steps),
            ["connect", "probe", "backends", "start", "ready"]
        );
        assert_eq!(installed.ahpd_port, 9187);
        assert_eq!(installed.token.as_deref(), Some("found+token"));
        assert!(!installed.by_desk);
        let backends = commands
            .iter()
            .find(|command| command.starts_with("# ahp-desk backends\n"))
            .unwrap();
        assert!(backends.contains("plugin install @ahpd/bot"), "{backends}");
        assert!(!backends.contains("agent-claude"));
        assert!(
            backends.contains("/home/ana/.cache/ahp-desk/.step.pid")
                && !backends.contains(INSTALL_DIR),
            "nothing goes in the desk's install folder: {backends}"
        );
        let start = commands
            .iter()
            .find(|command| command.starts_with("# ahp-desk start\n"))
            .unwrap();
        assert!(
            start.ends_with("/usr/local/bin/ahpd restart"),
            "a running ahpd restarts so the new backend loads"
        );
        assert!(
            !commands
                .iter()
                .any(|command| command.contains("@ahpd/server")),
            "nothing installed"
        );
        assert!(
            !commands
                .iter()
                .filter(|command| !command.starts_with(PROBE_MARK))
                .any(|command| command.contains("systemctl")),
            "no unit"
        );
    }

    #[tokio::test]
    async fn install_with_a_found_ahpd_too_old_is_refused_with_no_install_command() {
        let server = TestServer::start().await;
        answer_install(
            &server,
            &format!("{DEBIAN}ahpd=/usr/bin/ahpd\nahpdVersion=0.7.0\n"),
        );
        server.answer(
            FOUND_MARK,
            "--plugins--\n[]\n--status--\n{\"running\":false}\n",
            0,
        );
        let cancel = AtomicBool::new(false);
        let (done, steps, commands) =
            run_install(&server, &request(&["@ahpd/agent-claude"]), &cancel).await;
        assert!(done.unwrap_err().contains("0.7.0"));
        assert_eq!(
            steps.last().unwrap(),
            &("probe".to_string(), StepStatus::Failed)
        );
        assert!(commands
            .iter()
            .all(|command| command.starts_with(PROBE_MARK) || command.starts_with(FOUND_MARK)));
    }

    #[tokio::test]
    async fn install_a_ready_that_never_answers_shows_the_journal_and_names_the_port() {
        let server = TestServer::start().await;
        // The first answer that matches wins, so these go in before the install's.
        server.answer("# ahp-desk ready\n", "no answer on 9300\n", 1);
        server.answer("# ahp-desk journal\n", "ahpd[1]: port 9300 is in use\n", 0);
        answer_install(&server, DEBIAN);
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let keys = Keys {
            id: "dev",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let events = Mutex::new(Vec::new());
        let send = |event: InstallEvent| events.lock().unwrap().push(event);
        let cancel = AtomicBool::new(false);
        let failed = install(
            &server.target(None),
            Box::new(|_| KeyCheck::Known),
            keys,
            &request(&["@ahpd/agent-claude"]),
            &send,
            &cancel,
        )
        .await
        .unwrap_err();
        assert!(failed.contains("port 9300"), "{failed}");
        let events = events.into_inner().unwrap();
        assert!(events.contains(&InstallEvent::Line {
            text: "ahpd[1]: port 9300 is in use".into()
        }));
        assert_eq!(
            events.last().unwrap(),
            &InstallEvent::Step {
                id: "ready".into(),
                status: StepStatus::Failed
            }
        );
    }

    #[test]
    fn install_not_ready_names_each_backend_with_its_state_and_reason() {
        let list: Vec<serde_json::Value> = serde_json::from_str(
            r#"[{"spec":"@ahpd/agent-claude","name":"@ahpd/agent-claude","state":"error","problem":"its name is not a slug"},{"spec":{"name":"@ahpd/bot"},"name":"bot","state":"ready"}]"#,
        )
        .unwrap();
        let asked = [
            "@ahpd/agent-claude".to_string(),
            "@ahpd/bot".to_string(),
            "@ahpd/x".to_string(),
        ];
        assert_eq!(
            not_ready(&list, &asked),
            [
                "@ahpd/agent-claude (error: its name is not a slug)",
                "@ahpd/x (not listed)"
            ]
        );
    }

    #[test]
    fn probe_an_ahpd_config_already_there_is_named() {
        let probe = parse_probe(&format!("{DEBIAN}ahpdConfig=/home/ana/.config/ahpd\n"));
        assert_eq!(probe.refusal, None);
        assert_eq!(probe.ahpd_config.as_deref(), Some("/home/ana/.config/ahpd"));
    }

    #[tokio::test]
    async fn install_a_backend_the_new_ahpd_cannot_load_stops_before_the_unit() {
        let server = TestServer::start().await;
        server.answer(
            "# ahp-desk plugins\n",
            r#"[{"spec":"@ahpd/agent-claude","name":"@ahpd/agent-claude","state":"error","problem":"its name is not a slug"}]"#,
            0,
        );
        answer_install(&server, DEBIAN);
        let cancel = AtomicBool::new(false);
        let (done, steps, commands) =
            run_install(&server, &request(&["@ahpd/agent-claude"]), &cancel).await;
        let failed = done.unwrap_err();
        assert!(
            failed.contains("@ahpd/agent-claude (error: its name is not a slug)"),
            "{failed}"
        );
        assert!(failed.contains("~/.config/ahpd"));
        assert_eq!(
            steps.last().unwrap(),
            &("backends".to_string(), StepStatus::Failed)
        );
        assert!(!commands
            .iter()
            .any(|command| command.starts_with("# ahp-desk unit\n")));
    }

    #[tokio::test]
    async fn install_a_cancel_before_a_step_sends_no_further_step() {
        let server = TestServer::start().await;
        answer_install(&server, DEBIAN);
        let cancel = AtomicBool::new(true);
        let (done, steps, commands) =
            run_install(&server, &request(&["@ahpd/agent-claude"]), &cancel).await;
        assert_eq!(done.unwrap_err(), "cancelled");
        assert!(steps.is_empty());
        assert!(commands.is_empty());
    }

    #[test]
    fn install_control_allows_one_install_at_a_time() {
        let control = InstallControl::default();
        let held = control.begin().unwrap();
        assert!(control.begin().is_err());
        control.cancel();
        drop(held);
        let _again = control.begin().unwrap();
        assert!(
            !control.cancel.load(Ordering::SeqCst),
            "a new install starts uncancelled"
        );
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

    /// By hand, on the test server Softov names, as `ssh_a_real_install`:
    /// installs the newest ahpd, updates it to an older one and back, checking
    /// each time that the service runs that version and keeps its token; then
    /// removes what it installed and puts `~/.config/ahpd` back.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_update() {
        const OLDER: &str = "0.11.0";
        let (target, check) = crate::ssh::tests::real_target();
        drop(check);
        let fingerprint = std::env::var("AHP_DESK_SSH_TEST_KEY").unwrap();
        let trusted = move || -> crate::ssh::ServerCheck {
            let fingerprint = fingerprint.clone();
            Box::new(move |key| {
                let offered = crate::ssh::known_keys::fingerprint(key);
                if offered.sha256 == fingerprint {
                    KeyCheck::Known
                } else {
                    KeyCheck::New { offered }
                }
            })
        };
        let secrets = Secrets::default();
        let keys = || Keys {
            id: "real",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let (session, _) = ssh::open(&target, trusted(), keys()).await.unwrap();
        let shell = |command: String| {
            let session = &session;
            async move {
                let (code, said) = exec_capture(session, &command).await.unwrap();
                println!("$ {command}\n{said}");
                (code, said)
            }
        };
        let (_, home) = shell("printf %s \"$HOME\"".into()).await;
        let backup = format!("{home}/.config/ahpd.desk-test-backup");
        let config = format!("{home}/.config/ahpd");
        let (code, _) = shell(format!(
            "test ! -e {b} && if [ -e {c} ]; then cp -a {c} {b}; fi",
            b = quote(&backup),
            c = quote(&config)
        ))
        .await;
        assert_eq!(
            code, 0,
            "a backup from an earlier run is still there: {backup}"
        );
        let clean = format!(
            "systemctl --user disable --now ahpd.service 2>/dev/null; rm -f {unit}; systemctl --user daemon-reload; rm -rf {prefix}",
            unit = quote(&format!("{home}/{UNIT_FILE}")),
            prefix = quote(&format!("{home}/{INSTALL_DIR}"))
        );
        let restore = format!(
            "rm -rf {c}; if [ -e {b} ]; then mv {b} {c}; fi",
            b = quote(&backup),
            c = quote(&config)
        );
        let send = |event: InstallEvent| match event {
            InstallEvent::Line { text } => println!("  {text}"),
            InstallEvent::Step { id, status } => println!("[{id}] {status:?}"),
        };
        let cancel = AtomicBool::new(false);
        let mut request = request(&["@ahpd/agent-claude"]);
        request.target = ProbeTarget {
            id: "real".into(),
            address: target.address.clone(),
            port: target.port,
            user: target.user.clone(),
            key_file: target
                .key_file
                .as_ref()
                .map(|path| path.display().to_string()),
        };
        request.ahpd_port = 9387;

        let checked = async {
            let installed = install(&target, trusted(), keys(), &request, &send, &cancel).await?;
            let token = format!("{home}/{INSTALL_DIR}/connection-token");
            let (_, before) = shell(format!("sha256sum {}", quote(&token))).await;
            for version in [OLDER, "latest"] {
                let now = update(&target, trusted(), keys(), version, 9387, &send, &cancel).await?;
                let wanted = if version == "latest" {
                    installed.version.as_str()
                } else {
                    version
                };
                assert_eq!(now, wanted);
                let (_, active) = shell("systemctl --user is-active ahpd.service".into()).await;
                assert_eq!(active.trim(), "active");
                let (_, after) = shell(format!("sha256sum {}", quote(&token))).await;
                assert_eq!(after, before, "the token file is left as it was");
            }
            Ok::<(), String>(())
        }
        .await;
        shell(clean).await;
        shell(restore).await;
        checked.unwrap();
    }

    /// By hand, on the test server Softov names, as `ssh_a_real_install`:
    /// installs, uninstalls, and checks that no unit, no install and nothing on
    /// the ahpd port is left, and that `~/.config/ahpd` is as the install left it.
    #[tokio::test]
    #[ignore]
    async fn ssh_a_real_uninstall() {
        let (target, check) = crate::ssh::tests::real_target();
        drop(check);
        let fingerprint = std::env::var("AHP_DESK_SSH_TEST_KEY").unwrap();
        let trusted = move || -> crate::ssh::ServerCheck {
            let fingerprint = fingerprint.clone();
            Box::new(move |key| {
                let offered = crate::ssh::known_keys::fingerprint(key);
                if offered.sha256 == fingerprint {
                    KeyCheck::Known
                } else {
                    KeyCheck::New { offered }
                }
            })
        };
        let secrets = Secrets::default();
        let keys = || Keys {
            id: "real",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let (session, _) = ssh::open(&target, trusted(), keys()).await.unwrap();
        let shell = |command: String| {
            let session = &session;
            async move {
                let (code, said) = exec_capture(session, &command).await.unwrap();
                println!("$ {command}\n{said}");
                (code, said)
            }
        };
        let (_, home) = shell("printf %s \"$HOME\"".into()).await;
        let backup = format!("{home}/.config/ahpd.desk-test-backup");
        let config = format!("{home}/.config/ahpd");
        let (code, _) = shell(format!(
            "test ! -e {b} && if [ -e {c} ]; then cp -a {c} {b}; fi",
            b = quote(&backup),
            c = quote(&config)
        ))
        .await;
        assert_eq!(
            code, 0,
            "a backup from an earlier run is still there: {backup}"
        );
        let clean = format!(
            "systemctl --user disable --now ahpd.service 2>/dev/null; rm -f {unit}; systemctl --user daemon-reload; rm -rf {prefix}",
            unit = quote(&format!("{home}/{UNIT_FILE}")),
            prefix = quote(&format!("{home}/{INSTALL_DIR}"))
        );
        let restore = format!(
            "rm -rf {c}; if [ -e {b} ]; then mv {b} {c}; fi",
            b = quote(&backup),
            c = quote(&config)
        );
        let send = |event: InstallEvent| match event {
            InstallEvent::Line { text } => println!("  {text}"),
            InstallEvent::Step { id, status } => println!("[{id}] {status:?}"),
        };
        let mut request = request(&["@ahpd/agent-claude"]);
        request.target = ProbeTarget {
            id: "real".into(),
            address: target.address.clone(),
            port: target.port,
            user: target.user.clone(),
            key_file: target
                .key_file
                .as_ref()
                .map(|path| path.display().to_string()),
        };
        request.ahpd_port = 9387;

        let checked = async {
            let cancel = AtomicBool::new(false);
            install(&target, trusted(), keys(), &request, &send, &cancel).await?;
            let (_, config_before) = shell(format!("ls -A {}", quote(&config))).await;
            uninstall(&target, trusted(), keys(), &send).await?;
            let (unit_code, _) = shell("systemctl --user cat ahpd.service".into()).await;
            assert_ne!(unit_code, 0, "systemd still knows the unit");
            let (left, _) = shell(format!(
                "test -e {} || test -e {}",
                quote(&format!("{home}/{UNIT_FILE}")),
                quote(&format!("{home}/{INSTALL_DIR}"))
            ))
            .await;
            assert_ne!(left, 0, "the unit file or the install is still there");
            let (listening, _) = shell("ss -ltn | grep -q ':9387\\b'".into()).await;
            assert_ne!(listening, 0, "something still listens on 9387");
            let (_, config_after) = shell(format!("ls -A {}", quote(&config))).await;
            assert_eq!(
                config_after, config_before,
                "~/.config/ahpd is left as it was"
            );
            Ok::<(), String>(())
        }
        .await;
        shell(clean).await;
        shell(restore).await;
        checked.unwrap();
    }

    #[test]
    fn uninstall_script_quotes_a_home_with_a_space_and_never_names_the_config() {
        let steps = uninstall_script("/home/ana b", Some("/run/user/1000")).unwrap();
        let ids: Vec<&str> = steps.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, ["stop", "unit", "files"]);
        assert!(steps[0]
            .1
            .contains("test -f '/home/ana b/.config/systemd/user/ahpd.service'"));
        assert!(steps[0].1.contains(
            "XDG_RUNTIME_DIR=/run/user/1000 systemctl --user disable --now ahpd.service"
        ));
        assert!(steps[1]
            .1
            .contains("rm -f '/home/ana b/.config/systemd/user/ahpd.service'"));
        assert!(steps[2]
            .1
            .contains("rm -rf -- '/home/ana b/.local/share/ahpd'"));
        assert!(steps[2]
            .1
            .contains("'/home/ana b/.local/share/ahpd'/node_modules/@ahpd/server"));
        assert!(steps.iter().all(|(_, body)| !body.contains(".config/ahpd")));
    }

    #[test]
    fn uninstall_script_refuses_a_home_that_is_not_an_absolute_folder() {
        for home in ["", "/", "//", "home/ana", "~", "/home/ana\nrm"] {
            assert!(uninstall_script(home, None).is_err(), "{home:?}");
        }
    }

    /// Runs an uninstall against the test server and answers its result, the
    /// steps in the order they ended, and the commands the server saw.
    async fn run_uninstall(
        server: &TestServer,
    ) -> (Result<(), String>, Vec<(String, StepStatus)>, Vec<String>) {
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let keys = Keys {
            id: "dev",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let events = Mutex::new(Vec::new());
        let send = |event: InstallEvent| events.lock().unwrap().push(event);
        let done = uninstall(
            &server.target(None),
            Box::new(|_| KeyCheck::Known),
            keys,
            &send,
        )
        .await;
        let steps = events
            .into_inner()
            .unwrap()
            .into_iter()
            .filter_map(|event| match event {
                InstallEvent::Step { id, status } if status != StepStatus::Running => {
                    Some((id, status))
                }
                _ => None,
            })
            .collect();
        (done, steps, server.commands())
    }

    fn ran(commands: &[String]) -> Vec<&str> {
        commands
            .iter()
            .filter_map(|command| command.strip_prefix("# ahp-desk "))
            .map(|rest| rest.split('\n').next().unwrap())
            .collect()
    }

    #[tokio::test]
    async fn uninstall_stops_removes_the_unit_and_the_install_and_leaves_the_config() {
        let server = TestServer::start().await;
        server.answer(
            PROBE_MARK,
            &format!("{DEBIAN}{DESK_INSTALL}ahpdConfig=/home/ana/.config/ahpd\n"),
            0,
        );
        for id in ["found", "stop", "unit", "files"] {
            server.answer(&format!("# ahp-desk {id}\n"), "", 0);
        }
        let (done, steps, commands) = run_uninstall(&server).await;
        done.unwrap();
        assert_eq!(ids(&steps), ["connect", "stop", "unit", "files"]);
        assert!(steps.iter().all(|(_, status)| *status == StepStatus::Done));
        assert_eq!(ran(&commands), ["probe", "found", "stop", "unit", "files"]);
        assert!(commands
            .iter()
            .filter(|command| !command.starts_with(PROBE_MARK))
            .all(|command| !command.contains(".config/ahpd")));
    }

    #[tokio::test]
    async fn uninstall_with_no_unit_skips_the_stop_and_the_unit() {
        let server = TestServer::start().await;
        server.answer(PROBE_MARK, &format!("{DEBIAN}{DESK_INSTALL}"), 0);
        server.answer("# ahp-desk found\n", "", 0);
        server.answer("# ahp-desk stop\n", "", NOTHING_THERE as u32);
        server.answer("# ahp-desk unit\n", "", NOTHING_THERE as u32);
        server.answer("# ahp-desk files\n", "", 0);
        let (done, steps, _) = run_uninstall(&server).await;
        done.unwrap();
        assert_eq!(
            steps,
            [
                ("connect".to_string(), StepStatus::Done),
                ("stop".to_string(), StepStatus::Skipped),
                ("unit".to_string(), StepStatus::Skipped),
                ("files".to_string(), StepStatus::Done)
            ]
        );
    }

    #[tokio::test]
    async fn uninstall_stops_at_a_failing_stop_before_any_removal() {
        let server = TestServer::start().await;
        server.answer(PROBE_MARK, &format!("{DEBIAN}{DESK_INSTALL}"), 0);
        server.answer("# ahp-desk found\n", "", 0);
        server.answer("# ahp-desk stop\n", "Failed to connect to bus\n", 1);
        let (done, steps, commands) = run_uninstall(&server).await;
        assert!(done.unwrap_err().contains("Failed to connect to bus"));
        assert_eq!(
            steps.last().unwrap(),
            &("stop".to_string(), StepStatus::Failed)
        );
        assert!(!ran(&commands).contains(&"unit") && !ran(&commands).contains(&"files"));
    }

    #[tokio::test]
    async fn uninstall_of_a_server_that_does_not_answer_fails_at_connect() {
        let server = TestServer::start().await;
        let mut target = server.target(None);
        drop(server);
        // Nothing listens on the port once the test server is gone.
        target.port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let secrets = Secrets::default();
        let keys = Keys {
            id: "dev",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let events = Mutex::new(Vec::new());
        let send = |event: InstallEvent| events.lock().unwrap().push(event);
        let done = uninstall(&target, Box::new(|_| KeyCheck::Known), keys, &send).await;
        assert!(done.is_err());
        assert_eq!(
            events.into_inner().unwrap().last().unwrap(),
            &InstallEvent::Step {
                id: "connect".into(),
                status: StepStatus::Failed
            }
        );
    }

    #[tokio::test]
    async fn uninstall_host_refuses_a_host_not_installed_by_the_desk_without_connecting() {
        use crate::hosts::{HostInput, HostKind, SshHostView};
        let server = TestServer::start().await;
        let scratch = crate::ssh::known_keys::tests::Scratch::new();
        let store = HostStore::load(
            scratch.0.join("hosts.json"),
            Box::new(crate::tokens::test_vaults::Memory::default()),
        )
        .unwrap();
        store
            .save(
                HostInput {
                    id: "found".into(),
                    name: "found".into(),
                    kind: HostKind::Url,
                    url: "ws://127.0.0.1:9187".into(),
                    ahpd_path: None,
                    on_desk: None,
                    ssh: Some(SshHostView {
                        address: "127.0.0.1".into(),
                        port: server.target(None).port,
                        user: "tester".into(),
                        sign_in: None,
                        key_file: None,
                        reach: Reach::Port,
                        remote_port: 9187,
                        folder: None,
                        by_desk: false,
                        local_port: None,
                    }),
                },
                None,
            )
            .unwrap();
        let send = |_: InstallEvent| {};
        let done = uninstall_host(&store, &scratch.known(), "found", &send).await;
        assert!(done.unwrap_err().contains("leaves it on the server"));
        assert_eq!(server.sign_in_attempts(), 0);
        assert!(server.commands().is_empty());
    }

    const DESK_INSTALL: &str = "deskUnit=yes\nahpd=/home/ana/.local/share/ahpd/node_modules/.bin/ahpd\nahpdNode=/home/ana/.local/share/ahpd/node/bin/node\nahpdVersion=0.11.0\nahpdDesk=yes\n";

    /// Runs an update against the test server and answers its result, the
    /// steps in the order they ran, and the commands the server saw.
    async fn run_update(
        server: &TestServer,
        version: &str,
    ) -> (
        Result<String, String>,
        Vec<(String, StepStatus)>,
        Vec<String>,
    ) {
        let secrets = Secrets::default();
        secrets.set_password("dev", PASSWORD);
        let keys = Keys {
            id: "dev",
            secrets: &secrets,
            kept_password: None,
            signed_in_before: None,
        };
        let events = Mutex::new(Vec::new());
        let send = |event: InstallEvent| events.lock().unwrap().push(event);
        let cancel = AtomicBool::new(false);
        let done = update(
            &server.target(None),
            Box::new(|_| KeyCheck::Known),
            keys,
            version,
            9300,
            &send,
            &cancel,
        )
        .await;
        let steps = events
            .into_inner()
            .unwrap()
            .into_iter()
            .filter_map(|event| match event {
                InstallEvent::Step { id, status } if status != StepStatus::Running => {
                    Some((id, status))
                }
                _ => None,
            })
            .collect();
        (done, steps, server.commands())
    }

    #[tokio::test]
    async fn update_installs_the_version_with_the_units_node_restarts_and_waits() {
        let server = TestServer::start().await;
        answer_install(&server, &format!("{DEBIAN}{DESK_INSTALL}"));
        let (done, steps, commands) = run_update(&server, "0.12.0").await;
        assert_eq!(done.unwrap(), "0.12.0");
        assert_eq!(
            ids(&steps),
            ["connect", "probe", "server", "start", "ready"]
        );
        let ran: Vec<&str> = commands
            .iter()
            .filter_map(|command| command.strip_prefix("# ahp-desk "))
            .map(|rest| rest.split('\n').next().unwrap())
            .collect();
        assert_eq!(
            ran,
            ["probe", "found", "server", "start", "ready", "token", "version"]
        );
        let of = |id: &str| {
            commands
                .iter()
                .find(|command| command.starts_with(&format!("# ahp-desk {id}\n")))
                .unwrap()
                .clone()
        };
        assert!(of("server").contains(
            "/home/ana/.local/share/ahpd/node/bin/npm install --prefix /home/ana/.local/share/ahpd @ahpd/server@0.12.0"
        ));
        assert!(of("server").contains("PATH=/home/ana/.local/share/ahpd/node/bin:"));
        assert!(of("start").ends_with("systemctl --user restart ahpd.service"));
        assert!(of("ready").contains("/home/ana/.local/share/ahpd/node/bin/node"));
        assert!(of("ready").ends_with(" 9300"));
    }

    #[tokio::test]
    async fn update_on_a_server_without_the_desks_install_stops_after_the_probe() {
        let server = TestServer::start().await;
        answer_install(&server, DEBIAN);
        let (done, steps, commands) = run_update(&server, "latest").await;
        assert!(done.unwrap_err().contains("not on the server"));
        assert_eq!(
            steps,
            [
                ("connect".to_string(), StepStatus::Done),
                ("probe".to_string(), StepStatus::Failed)
            ]
        );
        assert!(!commands.iter().any(|c| c.starts_with("# ahp-desk server")));
    }

    #[tokio::test]
    async fn update_of_an_ahpd_the_desk_did_not_install_is_refused() {
        let server = TestServer::start().await;
        answer_install(
            &server,
            &format!("{DEBIAN}ahpd=/usr/local/bin/ahpd\nahpdVersion=0.11.0\n"),
        );
        let (done, _, commands) = run_update(&server, "latest").await;
        assert!(done.unwrap_err().contains("updates through ahpd"));
        assert!(!commands.iter().any(|c| c.starts_with("# ahp-desk server")));
    }

    #[tokio::test]
    async fn update_of_a_version_npm_would_not_read_is_refused_before_connecting() {
        let server = TestServer::start().await;
        let (done, steps, _) = run_update(&server, "1.0; rm -rf ~").await;
        assert!(done.unwrap_err().contains("not an npm version"));
        assert!(steps.is_empty());
        assert_eq!(server.sign_in_attempts(), 0);
    }

    #[tokio::test]
    async fn update_host_refuses_a_host_with_no_ssh_or_not_installed_by_the_desk_without_connecting(
    ) {
        use crate::hosts::{HostInput, HostKind, SshHostView};
        let server = TestServer::start().await;
        let scratch = crate::ssh::known_keys::tests::Scratch::new();
        let store = HostStore::load(
            scratch.0.join("hosts.json"),
            Box::new(crate::tokens::test_vaults::Memory::default()),
        )
        .unwrap();
        let host = |id: &str, ssh: Option<SshHostView>| HostInput {
            id: id.into(),
            name: id.into(),
            kind: HostKind::Url,
            url: "ws://127.0.0.1:9187".into(),
            ahpd_path: None,
            on_desk: None,
            ssh,
        };
        store.save(host("plain", None), None).unwrap();
        store
            .save(
                host(
                    "found",
                    Some(SshHostView {
                        address: "127.0.0.1".into(),
                        port: server.target(None).port,
                        user: "tester".into(),
                        sign_in: None,
                        key_file: None,
                        reach: Reach::Port,
                        remote_port: 9187,
                        folder: None,
                        by_desk: false,
                        local_port: None,
                    }),
                ),
                None,
            )
            .unwrap();
        let known = scratch.known();
        let send = |_: InstallEvent| {};
        let cancel = AtomicBool::new(false);
        let plain = update_host(&store, &known, "plain", "latest", &send, &cancel).await;
        assert!(plain.unwrap_err().contains("not reached over SSH"));
        let found = update_host(&store, &known, "found", "latest", &send, &cancel).await;
        assert!(found.unwrap_err().contains("updates through ahpd"));
        assert_eq!(server.sign_in_attempts(), 0);
        assert!(server.commands().is_empty());
    }
}
