//! The ahpd on this computer: the one set on the host, else the one the desk
//! installed, else one on `PATH` (decision: a local ahpd is found, or installed
//! with the bundled Node).

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::hosts::{HostStore, HOSTS_CHANGED};

/// npm's settings for an install the desk shows: no update notice, no funding
/// or audit summary.
const QUIET_NPM: [(&str, &str); 3] = [
    ("npm_config_update_notifier", "false"),
    ("npm_config_fund", "false"),
    ("npm_config_audit", "false"),
];

/// The oldest ahpd the desk drives: `status --json` came in 0.8.0.
pub const OLDEST_SUPPORTED: (u64, u64, u64) = (0, 8, 0);

/// How to run one ahpd: a program, what goes before ahpd's own arguments, and
/// the environment it needs on top of the desk's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launcher {
    pub program: PathBuf,
    pub before: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
}

impl Launcher {
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.before);
        command.envs(self.env.iter().map(|(key, value)| (key, value)));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW: the desk is a windowed app, and `ahpd.cmd`
            // would otherwise flash a console.
            command.creation_flags(0x0800_0000);
        }
        command
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    /// The path set on the host.
    Set,
    /// The ahpd the desk installed.
    Installed,
    /// The first `ahpd` on `PATH`.
    Path,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub launcher: Launcher,
    pub origin: Origin,
    pub version: String,
}

impl Found {
    pub fn supported(&self) -> bool {
        version_of_text(&self.version).is_some_and(|version| version >= OLDEST_SUPPORTED)
    }
}

/// The Node runtime the installer ships, as `scripts/stage-node.mjs` lays it out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundledNode {
    pub node: PathBuf,
    pub npm_cli: PathBuf,
}

impl BundledNode {
    /// The runtime in `dir`, the `node/` resource; none when it is not there.
    pub fn in_dir(dir: &Path) -> Option<Self> {
        let (node, npm_cli) = if cfg!(windows) {
            (
                dir.join("node.exe"),
                dir.join("node_modules/npm/bin/npm-cli.js"),
            )
        } else {
            (
                dir.join("bin/node"),
                dir.join("lib/node_modules/npm/bin/npm-cli.js"),
            )
        };
        (node.is_file() && npm_cli.is_file()).then_some(BundledNode { node, npm_cli })
    }

    /// `PATH` with this Node's folder first, so the `node` that npm and ahpd
    /// start is this one.
    fn path_var(&self) -> OsString {
        let folder = self
            .node
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let rest = std::env::var_os("PATH").unwrap_or_default();
        std::env::join_paths(std::iter::once(folder).chain(std::env::split_paths(&rest)))
            .unwrap_or(rest)
    }
}

/// Where the desk installs ahpd, and the Node that runs it.
pub struct LocalAhpd {
    /// The npm prefix, `<app local data>/ahpd`.
    pub prefix: PathBuf,
    /// The Node the installer ships; none in a build without it.
    pub node: Option<BundledNode>,
    /// Held while an install runs, so two never write the prefix at once.
    installing: Mutex<()>,
}

impl LocalAhpd {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let prefix = app
            .path()
            .app_local_data_dir()
            .map(without_verbatim)
            .map_err(|err| err.to_string())?
            .join("ahpd");
        let node = app
            .path()
            .resource_dir()
            .ok()
            .map(without_verbatim)
            .and_then(|dir| BundledNode::in_dir(&dir.join("node")));
        Ok(LocalAhpd::at(prefix, node))
    }

    pub fn at(prefix: PathBuf, node: Option<BundledNode>) -> Self {
        LocalAhpd {
            prefix,
            node,
            installing: Mutex::new(()),
        }
    }

    /// The ahpd the desk installed, run by its Node rather than npm's shim,
    /// which would look for a `node` on `PATH`. Its backends are under the
    /// prefix, and its config folder is the usual one (decision 7).
    pub fn installed(&self) -> Option<Launcher> {
        let node = self.node.as_ref()?;
        let script = installed_script(&self.prefix)?;
        Some(Launcher {
            program: node.node.clone(),
            before: vec![script.into_os_string()],
            env: vec![
                (
                    "AHPD_PLUGIN_ROOT".into(),
                    self.prefix.clone().into_os_string(),
                ),
                ("PATH".into(), node.path_var()),
            ],
        })
    }

    /// Installs `@ahpd/server` into the prefix with the bundled npm, then the
    /// `backends` through its `plugin install` (decision 8), sending each line
    /// npm and ahpd write to `line`.
    pub fn install(
        &self,
        backends: &[String],
        line: impl Fn(String) + Send + Sync + Clone + 'static,
    ) -> Result<(), String> {
        check_backends(backends)?;
        let node = self
            .node
            .as_ref()
            .ok_or("this build of the desk has no Node to install ahpd with")?;
        let _held = self
            .installing
            .try_lock()
            .map_err(|_| "ahpd is already being installed".to_string())?;
        std::fs::create_dir_all(&self.prefix)
            .map_err(|err| format!("cannot make {}: {err}", self.prefix.display()))?;
        let npm = Launcher {
            program: node.node.clone(),
            before: vec![node.npm_cli.clone().into_os_string()],
            env: vec![("PATH".into(), node.path_var())],
        };
        let mut command = npm.command();
        command
            .arg("install")
            .arg("--prefix")
            .arg(&self.prefix)
            .arg("@ahpd/server@latest")
            .envs(QUIET_NPM);
        run_streaming("npm install", command, line.clone())?;
        let ahpd = self
            .installed()
            .ok_or("npm finished, but @ahpd/server is not in the prefix")?;
        add_backends(&ahpd, backends, line)
    }
}

/// `ahpd plugin install <backends>` on `ahpd`, which installs them where it
/// loads plugins from and names them in its config (decision 8), sending each
/// line it writes to `line`.
pub fn add_backends(
    ahpd: &Launcher,
    backends: &[String],
    line: impl Fn(String) + Send + Sync + Clone + 'static,
) -> Result<(), String> {
    check_backends(backends)?;
    let mut command = ahpd.command();
    // ahpd runs npm here, and the same settings keep its lines to the install's own.
    command
        .arg("plugin")
        .arg("install")
        .args(backends)
        .envs(QUIET_NPM);
    run_streaming("ahpd plugin install", command, line)
}

/// The plugins `ahpd plugin list --json` has ready, by name.
pub fn ready_plugins(ahpd: &Launcher) -> Result<Vec<String>, String> {
    let output = ahpd
        .command()
        .args(["plugin", "list", "--json"])
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("cannot run {}: {err}", ahpd.program.display()))?;
    let list: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).map_err(|_| {
        format!(
            "ahpd plugin list --json did not answer a list: {}",
            said(&output)
        )
    })?;
    Ok(list
        .iter()
        .filter(|plugin| plugin["state"] == "ready")
        .filter_map(|plugin| plugin["name"].as_str().map(str::to_string))
        .collect())
}

/// The backends Add host offers are `@ahpd/` packages, and ahpd does not start
/// with none (decision 2).
fn check_backends(backends: &[String]) -> Result<(), String> {
    if backends.is_empty() {
        return Err("pick at least one backend; ahpd does not start without one".into());
    }
    for name in backends {
        let bare = name.strip_prefix("@ahpd/").unwrap_or_default();
        let plain = bare
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if bare.is_empty() || !plain {
            return Err(format!("{name} is not an @ahpd/ package name"));
        }
    }
    Ok(())
}

/// Runs `command`, the step `what`, sending each line of its stdout and
/// stderr as it comes, and fails naming the step when it exits non-zero: what
/// it said has gone to `line`, where the person reads it.
fn run_streaming(
    what: &str,
    mut command: Command,
    line: impl Fn(String) + Send + Sync + Clone + 'static,
) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("cannot run {program}: {err}"))?;
    let pump = |pipe: Box<dyn Read + Send>| {
        let line = line.clone();
        std::thread::spawn(move || {
            for text in BufReader::new(pipe).lines().map_while(Result::ok) {
                line(text);
            }
        })
    };
    let readers = [
        pump(Box::new(child.stdout.take().expect("stdout is piped"))),
        pump(Box::new(child.stderr.take().expect("stderr is piped"))),
    ];
    let status = child.wait().map_err(|err| err.to_string())?;
    for reader in readers {
        // A reader that panicked has already sent what it read.
        let _ = reader.join();
    }
    if status.success() {
        Ok(())
    } else {
        Err(format!("{what} exited with {status}"))
    }
}

/// `path` without Windows' `\\?\` prefix on a drive path, which Tauri's
/// folders carry and through which Node cannot resolve a script (EISDIR on
/// the drive). A UNC path keeps it.
fn without_verbatim(path: PathBuf) -> PathBuf {
    let drive = path
        .to_str()
        .and_then(|text| text.strip_prefix(r"\\?\"))
        .filter(|rest| rest.as_bytes().get(1) == Some(&b':'))
        .map(PathBuf::from);
    drive.unwrap_or(path)
}

/// `<prefix>/node_modules/@ahpd/server`'s `bin.ahpd`, when it is installed there.
pub fn installed_script(prefix: &Path) -> Option<PathBuf> {
    let package = prefix.join("node_modules").join("@ahpd").join("server");
    let text = std::fs::read_to_string(package.join("package.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&text).ok()?;
    let bin = match &manifest["bin"] {
        serde_json::Value::String(path) => path.as_str(),
        serde_json::Value::Object(bins) => bins.get("ahpd")?.as_str()?,
        _ => return None,
    };
    // `./dist/main.js` as `dist\main.js`, so the path the shell shows is plain.
    let script = Path::new(bin)
        .components()
        .filter(|part| *part != std::path::Component::CurDir)
        .fold(package, |path, part| path.join(part));
    script.is_file().then_some(script)
}

/// The first `ahpd` in `path_var`, as `PATH` lists its folders.
pub fn on_path(path_var: Option<OsString>) -> Option<PathBuf> {
    let names: &[&str] = if cfg!(windows) {
        &["ahpd.exe", "ahpd.cmd"]
    } else {
        &["ahpd"]
    };
    std::env::split_paths(&path_var?)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

/// The ahpd the desk uses, in the order decision 1 gives; `None` when there is none.
/// A set path that is not there is an error, never a reason to use another.
pub fn find(
    set: Option<&Path>,
    installed: Option<Launcher>,
    path_var: Option<OsString>,
    version: impl Fn(&Launcher) -> Result<String, String>,
) -> Result<Option<Found>, String> {
    let (launcher, origin) = if let Some(set) = set {
        if !set.is_file() {
            return Err(format!("ahpd is not at {}", set.display()));
        }
        (plain(set), Origin::Set)
    } else if let Some(installed) = installed {
        (installed, Origin::Installed)
    } else if let Some(found) = on_path(path_var) {
        (plain(&found), Origin::Path)
    } else {
        return Ok(None);
    };
    let version = version(&launcher)?;
    Ok(Some(Found {
        launcher,
        origin,
        version,
    }))
}

fn plain(program: &Path) -> Launcher {
    Launcher {
        program: program.to_path_buf(),
        before: Vec::new(),
        env: Vec::new(),
    }
}

/// `ahpd --version`, which prints the version alone.
pub fn version(launcher: &Launcher) -> Result<String, String> {
    let output = launcher
        .command()
        .arg("--version")
        .output()
        .map_err(|err| format!("cannot run {}: {err}", launcher.program.display()))?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() || version_of_text(&text).is_none() {
        let said = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "{} --version did not give a version: {}",
            launcher.program.display(),
            if said.is_empty() { &text } else { &said }
        ));
    }
    Ok(text)
}

/// `major.minor.patch` from text such as `0.11.0` or `0.12.0-rc.1`.
fn version_of_text(text: &str) -> Option<(u64, u64, u64)> {
    let core = text
        .trim()
        .trim_start_matches('v')
        .split(['-', '+'])
        .next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
    let version = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(version)
}

/// A local ahpd that is on, as `ahpd status --json` and its record give it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Running {
    pub pid: u32,
    /// Where it listens, with no token.
    pub url: String,
    /// The token its record's `connectUrl` carries; none when it takes none.
    pub token: Option<String>,
}

/// ahpd's config folder, as ahpd reads it: `$XDG_CONFIG_HOME/ahpd`, else
/// `<home>/.config/ahpd` (decision 7).
pub fn config_dir(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = match xdg.filter(|dir| !dir.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(home.filter(|dir| !dir.is_empty())?).join(".config"),
    };
    Some(base.join("ahpd"))
}

/// The config folder of the ahpd the desk runs, from the desk's environment.
fn this_config_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" });
    config_dir(std::env::var_os("XDG_CONFIG_HOME"), home)
        .ok_or_else(|| "no home folder to find ahpd's config folder in".to_string())
}

/// `ahpd status --json`: `None` for `{"running":false}`, which ahpd answers
/// with exit 1. The token is read from the record in `config`, and only when
/// the record names the same process.
pub fn status(launcher: &Launcher, config: &Path) -> Result<Option<Running>, String> {
    let output = launcher
        .command()
        .args(["status", "--json"])
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("cannot run {}: {err}", launcher.program.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let answer: serde_json::Value = serde_json::from_str(text.trim())
        .map_err(|_| format!("ahpd status --json did not answer JSON: {}", said(&output)))?;
    if answer["running"] == serde_json::Value::Bool(false) {
        return Ok(None);
    }
    let (Some(pid), Some(url)) = (answer["pid"].as_u64(), answer["url"].as_str()) else {
        return Err(format!(
            "ahpd status --json gave no pid and url: {}",
            text.trim()
        ));
    };
    let pid = u32::try_from(pid).map_err(|_| format!("ahpd status --json gave pid {pid}"))?;
    Ok(Some(Running {
        pid,
        url: url.to_string(),
        token: token_in_record(&config.join("daemon.json"), pid),
    }))
}

/// The `tkn` of the record's `connectUrl`, when the record is that process's.
fn token_in_record(record: &Path, pid: u32) -> Option<String> {
    let text = std::fs::read_to_string(record).ok()?;
    let record: serde_json::Value = serde_json::from_str(&text).ok()?;
    if record["pid"].as_u64() != Some(u64::from(pid)) {
        return None;
    }
    let url = tauri::Url::parse(record["connectUrl"].as_str()?).ok()?;
    let token = url.query_pairs().find(|(key, _)| key == "tkn")?.1;
    Some(token.into_owned())
}

/// `ahpd start` in `folder`, the folder it serves when its config names none,
/// with the desk's token file (decision 5); then what is on.
pub fn start(
    launcher: &Launcher,
    config: &Path,
    folder: &Path,
    token_file: &Path,
) -> Result<Running, String> {
    // ahpd writes a missing token file, but not the folder it goes in.
    if let Some(folder) = token_file.parent() {
        std::fs::create_dir_all(folder)
            .map_err(|err| format!("cannot make {}: {err}", folder.display()))?;
    }
    // The log keeps every run; only what this start adds says why it failed.
    let log = config.join("daemon.log");
    let before = std::fs::metadata(&log).map(|meta| meta.len()).unwrap_or(0);
    let output = launcher
        .command()
        .arg("start")
        .arg("--connection-token-file")
        .arg(token_file)
        .current_dir(folder)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("cannot run {}: {err}", launcher.program.display()))?;
    if !output.status.success() {
        let added = log_since(&log, before);
        return Err(format!("{}\n{}", said(&output), added.trim())
            .trim()
            .to_string());
    }
    status(launcher, config)?.ok_or_else(|| "ahpd start said it began, but none is on".to_string())
}

/// What `log` holds past byte `from`; all of it when the log is shorter, as a
/// new log is.
fn log_since(log: &Path, from: u64) -> String {
    let bytes = std::fs::read(log).unwrap_or_default();
    let from = usize::try_from(from).unwrap_or(0);
    let added = bytes.get(from..).unwrap_or(&bytes);
    String::from_utf8_lossy(added).into_owned()
}

/// `ahpd stop`, which on Windows reaches the daemon through its control pipe
/// from the first ahpd release after 0.11.0.
pub fn stop(launcher: &Launcher) -> Result<(), String> {
    let output = launcher
        .command()
        .arg("stop")
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("cannot run {}: {err}", launcher.program.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(said(&output))
    }
}

/// What a command wrote, stderr first, for an error.
fn said(output: &std::process::Output) -> String {
    let both = [&output.stderr, &output.stdout]
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string());
    both.into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The ahpd as the shell sees it.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundView {
    pub path: PathBuf,
    pub origin: Origin,
    pub version: String,
    pub supported: bool,
    /// The plugins it has ready (decision 12); none asked of one too old.
    pub plugins: Vec<String>,
}

impl From<&Found> for FoundView {
    fn from(found: &Found) -> Self {
        FoundView {
            // The installed one is its script; the program is the Node that runs it.
            path: found
                .launcher
                .before
                .first()
                .map(PathBuf::from)
                .unwrap_or_else(|| found.launcher.program.clone()),
            origin: found.origin,
            version: found.version.clone(),
            supported: found.supported(),
            plugins: Vec::new(),
        }
    }
}

/// The ahpd the desk would use, with `ahpd_path` as the path set on the host.
#[tauri::command]
pub async fn find_local_ahpd(
    local: State<'_, LocalAhpd>,
    ahpd_path: Option<PathBuf>,
) -> Result<Option<FoundView>, String> {
    let installed = local.installed();
    let path_var = std::env::var_os("PATH");
    tauri::async_runtime::spawn_blocking(move || {
        let Some(found) = find(ahpd_path.as_deref(), installed, path_var, version)? else {
            return Ok(None);
        };
        let mut view = FoundView::from(&found);
        if found.supported() {
            view.plugins = ready_plugins(&found.launcher)?;
        }
        Ok(Some(view))
    })
    .await
    .map_err(|err| err.to_string())?
}

/// Adds `backends` to the ahpd a local host would use (decision 12), then
/// answers it again.
#[tauri::command]
pub async fn add_local_backends(
    app: AppHandle,
    ahpd_path: Option<String>,
    backends: Vec<String>,
    events: Channel<InstallEvent>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let launcher = launcher_for(&app.state::<LocalAhpd>(), ahpd_path.as_deref())?;
        add_backends(&launcher, &backends, move |text| {
            // A shell that went away mid-install does not stop npm.
            let _ = events.send(InstallEvent::Line { text });
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

/// A local host's ahpd as the shell shows it.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalState {
    pub on: bool,
    pub started_by_desk: bool,
}

/// The ahpd a local host uses: its set path, else the desk's install, else
/// `PATH`; refused when there is none or it is too old (decision 9).
fn launcher_for(local: &LocalAhpd, ahpd_path: Option<&str>) -> Result<Launcher, String> {
    let found = find(
        ahpd_path.map(Path::new),
        local.installed(),
        std::env::var_os("PATH"),
        version,
    )?
    .ok_or("there is no ahpd on this computer")?;
    if !found.supported() {
        let (major, minor, patch) = OLDEST_SUPPORTED;
        return Err(format!(
            "ahpd {} is older than {major}.{minor}.{patch}, the oldest the desk runs",
            found.version
        ));
    }
    Ok(found.launcher)
}

/// The folder an ahpd whose config names none serves (decision 10).
fn home_folder() -> Result<PathBuf, String> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "no home folder to start ahpd in".to_string())
}

/// Saves what the local ahpd says and tells every window.
fn publish(
    app: &AppHandle,
    id: &str,
    on: Option<&Running>,
    started_by_desk: bool,
) -> Result<LocalState, String> {
    let store = app.state::<HostStore>();
    let list = store.set_local(
        id,
        on.map(|on| (on.url.as_str(), on.token.as_deref())),
        on.is_some() && started_by_desk,
    )?;
    app.emit(HOSTS_CHANGED, list)
        .map_err(|err| err.to_string())?;
    Ok(LocalState {
        on: on.is_some(),
        started_by_desk: on.is_some() && started_by_desk,
    })
}

/// Whether the local host's ahpd is on, taking its address and token from its
/// record (decision 4).
#[tauri::command]
pub async fn local_ahpd_status(app: AppHandle, id: String) -> Result<LocalState, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (ahpd_path, started) = app.state::<HostStore>().local(&id)?;
        let launcher = launcher_for(&app.state::<LocalAhpd>(), ahpd_path.as_deref())?;
        let running = status(&launcher, &this_config_dir()?)?;
        publish(&app, &id, running.as_ref(), started)
    })
    .await
    .map_err(|err| err.to_string())?
}

/// Starts the local host's ahpd, or takes the one already on, which the desk
/// then did not start. Blocks while ahpd starts.
pub fn start_host(app: &AppHandle, id: &str) -> Result<LocalState, String> {
    let (ahpd_path, started) = app.state::<HostStore>().local(id)?;
    let local = app.state::<LocalAhpd>();
    let launcher = launcher_for(&local, ahpd_path.as_deref())?;
    let config = this_config_dir()?;
    if let Some(running) = status(&launcher, &config)? {
        return publish(app, id, Some(&running), started);
    }
    let token_file = local.prefix.join("connection-token");
    let running = start(&launcher, &config, &home_folder()?, &token_file)?;
    publish(app, id, Some(&running), true)
}

/// Stops the local host's ahpd, only when the desk started it. Blocks while
/// ahpd stops.
pub fn stop_host(app: &AppHandle, id: &str) -> Result<LocalState, String> {
    let (ahpd_path, started) = app.state::<HostStore>().local(id)?;
    let launcher = launcher_for(&app.state::<LocalAhpd>(), ahpd_path.as_deref())?;
    if !started {
        return match status(&launcher, &this_config_dir()?)? {
            None => publish(app, id, None, false),
            Some(_) => Err("the desk did not start this ahpd, so it leaves it on".to_string()),
        };
    }
    stop(&launcher)?;
    publish(app, id, None, false)
}

#[tauri::command]
pub async fn start_local_ahpd(app: AppHandle, id: String) -> Result<LocalState, String> {
    tauri::async_runtime::spawn_blocking(move || start_host(&app, &id))
        .await
        .map_err(|err| err.to_string())?
}

#[tauri::command]
pub async fn stop_local_ahpd(app: AppHandle, id: String) -> Result<LocalState, String> {
    tauri::async_runtime::spawn_blocking(move || stop_host(&app, &id))
        .await
        .map_err(|err| err.to_string())?
}

/// At launch: starts each local host's ahpd set to start, off the main thread,
/// so the host list opens while they come up. A start that fails leaves the
/// host down, which its row shows.
pub fn start_on_launch(app: &AppHandle) {
    for id in app.state::<HostStore>().to_start_on_launch() {
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(err) = start_host(&app, &id) {
                eprintln!("ahp-desk: could not start the ahpd of {id}: {err}");
            }
        });
    }
}

/// At quit: stops each local host's ahpd set to stop that the desk started,
/// before the process ends (decision 3).
pub fn stop_on_quit(app: &AppHandle) {
    for id in app.state::<HostStore>().to_stop_on_quit() {
        if let Err(err) = stop_host(app, &id) {
            eprintln!("ahp-desk: could not stop the ahpd of {id}: {err}");
        }
    }
}

/// One step of an install, as the shell shows it.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum InstallEvent {
    /// A line npm or ahpd wrote.
    Line { text: String },
}

/// Installs ahpd and `backends` with the bundled Node, then answers the ahpd
/// the desk now uses.
#[tauri::command]
pub async fn install_local_ahpd(
    app: AppHandle,
    backends: Vec<String>,
    events: Channel<InstallEvent>,
) -> Result<Option<FoundView>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let local = app.state::<LocalAhpd>();
        local.install(&backends, move |text| {
            // A shell that went away mid-install does not stop npm.
            let _ = events.send(InstallEvent::Line { text });
        })?;
        let installed = local
            .installed()
            .ok_or("npm finished, but @ahpd/server is not in the prefix")?;
        find(None, Some(installed), None, version).map(|found| found.as_ref().map(FoundView::from))
    })
    .await
    .map_err(|err| err.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A folder of its own, gone at the end of the test.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "ahp-desk-local-ahpd-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
        fn file(&self, relative: &str, text: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, text).unwrap();
            path
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const SHIM: &str = if cfg!(windows) { "ahpd.cmd" } else { "ahpd" };

    fn says(version: &'static str) -> impl Fn(&Launcher) -> Result<String, String> {
        move |_| Ok(version.to_string())
    }

    fn installed(at: &Path) -> Launcher {
        Launcher {
            program: at.join("node"),
            before: vec![at.join("main.js").into_os_string()],
            env: Vec::new(),
        }
    }

    #[test]
    fn local_ahpd_the_set_path_wins() {
        let scratch = Scratch::new();
        let set = scratch.file("set/ahpd", "");
        scratch.file(&format!("bin/{SHIM}"), "");
        let path_var = Some(scratch.0.join("bin").into_os_string());
        let found = find(
            Some(&set),
            Some(installed(&scratch.0)),
            path_var,
            says("0.11.0"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(found.origin, Origin::Set);
        assert_eq!(found.launcher.program, set);
    }

    #[test]
    fn local_ahpd_a_set_path_that_is_gone_is_an_error() {
        let scratch = Scratch::new();
        scratch.file(&format!("bin/{SHIM}"), "");
        let path_var = Some(scratch.0.join("bin").into_os_string());
        let error = find(
            Some(&scratch.0.join("gone")),
            None,
            path_var,
            says("0.11.0"),
        )
        .unwrap_err();
        assert!(error.starts_with("ahpd is not at "), "{error}");
    }

    #[test]
    fn local_ahpd_the_installed_one_comes_before_path() {
        let scratch = Scratch::new();
        scratch.file(&format!("bin/{SHIM}"), "");
        let path_var = Some(scratch.0.join("bin").into_os_string());
        let found = find(None, Some(installed(&scratch.0)), path_var, says("0.11.0"))
            .unwrap()
            .unwrap();
        assert_eq!(found.origin, Origin::Installed);
        assert_eq!(FoundView::from(&found).path, scratch.0.join("main.js"));
    }

    #[test]
    fn local_ahpd_path_is_searched_in_order() {
        let scratch = Scratch::new();
        scratch.file("empty/readme", "");
        let first = scratch.file(&format!("first/{SHIM}"), "");
        scratch.file(&format!("second/{SHIM}"), "");
        let path_var = std::env::join_paths([
            scratch.0.join("empty"),
            scratch.0.join("first"),
            scratch.0.join("second"),
        ])
        .ok();
        let found = find(None, None, path_var, says("0.11.0")).unwrap().unwrap();
        assert_eq!(found.origin, Origin::Path);
        assert_eq!(found.launcher.program, first);
    }

    #[test]
    fn local_ahpd_none_found() {
        let scratch = Scratch::new();
        let path_var = Some(scratch.0.clone().into_os_string());
        assert_eq!(find(None, None, path_var, says("0.11.0")).unwrap(), None);
        assert_eq!(find(None, None, None, says("0.11.0")).unwrap(), None);
    }

    #[test]
    fn local_ahpd_an_old_ahpd_is_found_and_not_supported() {
        let scratch = Scratch::new();
        scratch.file(&format!("bin/{SHIM}"), "");
        let path_var = || Some(scratch.0.join("bin").into_os_string());
        let old = find(None, None, path_var(), says("0.7.0"))
            .unwrap()
            .unwrap();
        assert!(!old.supported());
        assert!(find(None, None, path_var(), says("0.8.0"))
            .unwrap()
            .unwrap()
            .supported());
        assert!(find(None, None, path_var(), says("0.12.0-rc.1"))
            .unwrap()
            .unwrap()
            .supported());
    }

    #[test]
    fn local_ahpd_a_version_that_cannot_be_read_is_an_error() {
        let scratch = Scratch::new();
        scratch.file(&format!("bin/{SHIM}"), "");
        let path_var = Some(scratch.0.join("bin").into_os_string());
        assert!(find(None, None, path_var, |_| Err("no".into())).is_err());
    }

    #[test]
    fn local_ahpd_versions_are_read() {
        assert_eq!(version_of_text("0.11.0\n"), Some((0, 11, 0)));
        assert_eq!(version_of_text("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(version_of_text("0.12.0-rc.1"), Some((0, 12, 0)));
        assert_eq!(version_of_text("0.11"), None);
        assert_eq!(version_of_text("ahpd 0.11.0"), None);
    }

    #[test]
    fn local_ahpd_the_installed_script_is_the_manifests_bin() {
        let scratch = Scratch::new();
        assert_eq!(installed_script(&scratch.0), None);
        scratch.file(
            "node_modules/@ahpd/server/package.json",
            r#"{"bin":{"ahpd":"./dist/main.js"}}"#,
        );
        assert_eq!(installed_script(&scratch.0), None, "no script yet");
        let script = scratch.file("node_modules/@ahpd/server/dist/main.js", "");
        assert_eq!(installed_script(&scratch.0), Some(script));
    }

    #[test]
    fn local_ahpd_no_bundled_node_means_no_installed_one() {
        let scratch = Scratch::new();
        scratch.file(
            "node_modules/@ahpd/server/package.json",
            r#"{"bin":"./main.js"}"#,
        );
        scratch.file("node_modules/@ahpd/server/main.js", "");
        let local = LocalAhpd::at(scratch.0.clone(), None);
        assert_eq!(local.installed(), None);
        let node = BundledNode {
            node: scratch.0.join("node"),
            npm_cli: scratch.0.join("npm-cli.js"),
        };
        let local = LocalAhpd::at(scratch.0.clone(), Some(node));
        let launcher = local.installed().unwrap();
        assert_eq!(launcher.program, scratch.0.join("node"));
        let root = (
            "AHPD_PLUGIN_ROOT".into(),
            scratch.0.clone().into_os_string(),
        );
        assert!(launcher.env.contains(&root));
    }

    #[test]
    fn local_ahpd_the_bundled_node_needs_node_and_npm() {
        let scratch = Scratch::new();
        let (node, npm) = if cfg!(windows) {
            ("node.exe", "node_modules/npm/bin/npm-cli.js")
        } else {
            ("bin/node", "lib/node_modules/npm/bin/npm-cli.js")
        };
        scratch.file(node, "");
        assert_eq!(BundledNode::in_dir(&scratch.0), None, "no npm yet");
        scratch.file(npm, "");
        let bundled = BundledNode::in_dir(&scratch.0).unwrap();
        assert_eq!(bundled.node, scratch.0.join(node));
        let first = std::env::split_paths(&bundled.path_var()).next().unwrap();
        assert_eq!(first, bundled.node.parent().unwrap());
    }

    #[test]
    fn local_ahpd_an_install_with_no_node_says_so() {
        let local = LocalAhpd::at(std::env::temp_dir().join("never-made"), None);
        let error = local
            .install(&["@ahpd/agent-claude".into()], |_| {})
            .unwrap_err();
        assert!(error.contains("no Node"), "{error}");
    }

    #[test]
    fn local_ahpd_backends_are_ahpd_packages_and_at_least_one() {
        let names = |list: &[&str]| list.iter().map(|one| one.to_string()).collect::<Vec<_>>();
        assert!(check_backends(&names(&["@ahpd/agent-claude", "@ahpd/bot"])).is_ok());
        assert!(check_backends(&[]).is_err());
        for bad in [
            "agent-claude",
            "@ahpd/",
            "@other/agent",
            "@ahpd/agent claude",
            "@ahpd/../x",
            "--registry=x",
        ] {
            assert!(check_backends(&names(&[bad])).is_err(), "{bad}");
        }
    }

    #[test]
    fn local_ahpd_a_streamed_step_sends_its_lines_and_names_itself() {
        let command = if cfg!(windows) {
            let mut command = Command::new("cmd");
            command.args(["/C", "echo one && echo two 1>&2 && exit 3"]);
            command
        } else {
            let mut command = Command::new("sh");
            command.args(["-c", "echo one; echo two 1>&2; exit 3"]);
            command
        };
        let heard = Arc::new(Mutex::new(Vec::new()));
        let sink = heard.clone();
        let error = run_streaming("the test", command, move |text| {
            sink.lock().unwrap().push(text.trim().to_string())
        })
        .unwrap_err();
        let mut heard = heard.lock().unwrap().clone();
        heard.sort();
        assert_eq!(heard, ["one", "two"]);
        assert!(error.starts_with("the test exited with"), "{error}");
    }

    #[test]
    fn local_ahpd_a_verbatim_drive_path_is_made_plain() {
        let plain = without_verbatim(PathBuf::from(r"\\?\F:\github\desk\target\debug"));
        assert_eq!(plain, PathBuf::from(r"F:\github\desk\target\debug"));
        let unc = PathBuf::from(r"\\?\UNC\server\share\desk");
        assert_eq!(
            without_verbatim(unc.clone()),
            unc,
            "a UNC path keeps its prefix"
        );
        let already = PathBuf::from(r"C:\Users\me");
        assert_eq!(without_verbatim(already.clone()), already);
    }

    /// A real install from npm, with the Node `pnpm stage:node` put in
    /// `resources/node`, into a scratch prefix and a scratch config folder.
    /// On Windows it needs an ahpd whose `plugin install` runs npm there.
    #[test]
    #[ignore = "needs the network and a staged Node"]
    fn local_ahpd_a_real_install() {
        let scratch = Scratch::new();
        // ahpd's config folder is $XDG_CONFIG_HOME/ahpd: never the user's own here.
        std::env::set_var("XDG_CONFIG_HOME", scratch.0.join("config"));
        let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/node");
        let local = LocalAhpd::at(scratch.0.join("ahpd"), BundledNode::in_dir(&resources));
        local
            .install(&["@ahpd/agent-claude".into()], |text| println!("| {text}"))
            .unwrap();
        let found = find(None, local.installed(), None, version)
            .unwrap()
            .unwrap();
        assert_eq!(found.origin, Origin::Installed);
        assert!(found.supported());
        let backend = scratch
            .0
            .join("ahpd/node_modules/@ahpd/agent-claude/package.json");
        assert!(backend.is_file(), "the backend is under the prefix");
        let config = fs::read_to_string(scratch.0.join("config/ahpd/config.json")).unwrap();
        assert!(config.contains("@ahpd/agent-claude"), "{config}");
    }

    #[test]
    fn local_ahpd_a_failed_start_reads_only_its_own_log_lines() {
        let scratch = Scratch::new();
        let log = scratch.file("daemon.log", "an earlier run failed\n");
        let before = fs::metadata(&log).unwrap().len();
        fs::write(&log, "an earlier run failed\nthis run failed\n").unwrap();
        assert_eq!(log_since(&log, before), "this run failed\n");
        assert_eq!(
            log_since(&log, 0),
            "an earlier run failed\nthis run failed\n"
        );
        assert_eq!(log_since(&scratch.0.join("none.log"), 0), "");
        // A log rolled shorter than where it was is read whole.
        assert_eq!(
            log_since(&log, 10_000),
            "an earlier run failed\nthis run failed\n"
        );
    }

    #[test]
    fn local_ahpd_the_config_folder_is_ahpds() {
        let dir = |xdg: Option<&str>, home: Option<&str>| {
            config_dir(xdg.map(Into::into), home.map(Into::into))
        };
        assert_eq!(
            dir(Some("/x"), Some("/h")),
            Some(PathBuf::from("/x").join("ahpd"))
        );
        assert_eq!(
            dir(Some(""), Some("/h")),
            Some(PathBuf::from("/h").join(".config").join("ahpd"))
        );
        assert_eq!(
            dir(None, Some("/h")),
            Some(PathBuf::from("/h").join(".config").join("ahpd"))
        );
        assert_eq!(dir(None, None), None);
    }

    #[test]
    fn local_ahpd_the_token_is_the_records_for_that_process() {
        let scratch = Scratch::new();
        let record = scratch.file(
            "daemon.json",
            r#"{"pid":42,"url":"ws://127.0.0.1:9187","connectUrl":"ws://127.0.0.1:9187/?tkn=a%2Bb%3D"}"#,
        );
        assert_eq!(token_in_record(&record, 42).as_deref(), Some("a+b="));
        assert_eq!(
            token_in_record(&record, 43),
            None,
            "another process's record"
        );
        assert_eq!(token_in_record(&scratch.0.join("gone.json"), 42), None);
        let record = scratch.file(
            "open.json",
            r#"{"pid":42,"connectUrl":"ws://127.0.0.1:9187/"}"#,
        );
        assert_eq!(token_in_record(&record, 42), None, "an ahpd with no token");
    }

    /// The bundled Node running a script that answers `status --json` with
    /// `FAKE_STATUS` and exits with `FAKE_CODE`.
    fn fake_ahpd(scratch: &Scratch, status: &str, code: i32) -> Launcher {
        let node = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/node");
        let node = BundledNode::in_dir(&node).expect("pnpm stage:node first");
        let script = scratch.file(
            "fake-ahpd.js",
            "process.stdout.write(process.env.FAKE_STATUS + '\\n'); process.exit(Number(process.env.FAKE_CODE));",
        );
        Launcher {
            program: node.node,
            before: vec![script.into_os_string()],
            env: vec![
                ("FAKE_STATUS".into(), status.into()),
                ("FAKE_CODE".into(), code.to_string().into()),
            ],
        }
    }

    #[test]
    fn local_ahpd_only_ready_plugins_count() {
        let scratch = Scratch::new();
        let list = r#"[{"name":"@ahpd/agent-claude","state":"ready"},{"name":"@ahpd/agent-pi","state":"missing"},{"name":"x","state":"ready"}]"#;
        assert_eq!(
            ready_plugins(&fake_ahpd(&scratch, list, 0)).unwrap(),
            ["@ahpd/agent-claude", "x"]
        );
        assert!(ready_plugins(&fake_ahpd(&scratch, "[]", 0))
            .unwrap()
            .is_empty());
        assert!(ready_plugins(&fake_ahpd(&scratch, "ahpd: no", 2)).is_err());
    }

    #[test]
    fn local_ahpd_status_reads_on_and_off() {
        let scratch = Scratch::new();
        scratch.file(
            "daemon.json",
            r#"{"pid":7,"url":"ws://127.0.0.1:9187","connectUrl":"ws://127.0.0.1:9187/?tkn=secret"}"#,
        );
        let off = fake_ahpd(&scratch, r#"{"running":false}"#, 1);
        assert_eq!(status(&off, &scratch.0).unwrap(), None);
        let on = fake_ahpd(
            &scratch,
            r#"{"pid":7,"url":"ws://127.0.0.1:9187","paths":[]}"#,
            0,
        );
        assert_eq!(
            status(&on, &scratch.0).unwrap(),
            Some(Running {
                pid: 7,
                url: "ws://127.0.0.1:9187".into(),
                token: Some("secret".into())
            })
        );
        let broken = fake_ahpd(&scratch, "ahpd: something", 2);
        assert!(status(&broken, &scratch.0)
            .unwrap_err()
            .contains("did not answer JSON"));
    }

    /// Starts, reads and stops a real ahpd in a scratch config folder:
    /// `AHP_DESK_TEST_AHPD` is its `dist/main.js`, `AHP_DESK_TEST_PLUGINS` a
    /// plugin root holding `@ahpd/agent-claude`.
    #[test]
    #[ignore = "needs an ahpd build and its backend"]
    fn local_ahpd_a_real_start_and_stop() {
        let scratch = Scratch::new();
        let main = std::env::var_os("AHP_DESK_TEST_AHPD").expect("AHP_DESK_TEST_AHPD");
        let plugins = std::env::var_os("AHP_DESK_TEST_PLUGINS").expect("AHP_DESK_TEST_PLUGINS");
        let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/node");
        let node = BundledNode::in_dir(&resources).expect("pnpm stage:node first");
        let config = scratch.0.join("config/ahpd");
        scratch.file(
            "config/ahpd/config.json",
            r#"{"port":9324,"plugins":["@ahpd/agent-claude"]}"#,
        );
        let folder = scratch.0.join("work");
        fs::create_dir_all(&folder).unwrap();
        let ahpd = Launcher {
            program: node.node.clone(),
            before: vec![main],
            env: vec![
                (
                    "XDG_CONFIG_HOME".into(),
                    scratch.0.join("config").into_os_string(),
                ),
                ("AHPD_PLUGIN_ROOT".into(), plugins),
                ("PATH".into(), node.path_var()),
            ],
        };
        assert_eq!(status(&ahpd, &config).unwrap(), None);
        let token_file = scratch.0.join("not-made-yet").join("token");
        let on = start(&ahpd, &config, &folder, &token_file).unwrap();
        println!("{on:?}");
        assert_eq!(on.url, "ws://127.0.0.1:9324");
        let written = fs::read_to_string(&token_file).unwrap();
        assert_eq!(
            on.token.as_deref(),
            Some(written.trim()),
            "the record's token is the file's"
        );
        assert_eq!(status(&ahpd, &config).unwrap(), Some(on));
        stop(&ahpd).unwrap();
        assert_eq!(status(&ahpd, &config).unwrap(), None);
    }

    /// Runs the real `ahpd --version` of the first ahpd on this machine's `PATH`.
    #[test]
    #[ignore = "needs an ahpd on PATH"]
    fn local_ahpd_the_real_path() {
        let found = find(None, None, std::env::var_os("PATH"), version).unwrap();
        println!(
            "{found:?} supported={:?}",
            found.as_ref().map(Found::supported)
        );
    }
}
