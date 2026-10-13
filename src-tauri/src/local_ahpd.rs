//! The ahpd on this computer: the one set on the host, else the one the desk
//! installed, else one on `PATH` (decision: a local ahpd is found, or installed
//! with the bundled Node).

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

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
            .map_err(|err| err.to_string())?
            .join("ahpd");
        let node = app
            .path()
            .resource_dir()
            .ok()
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

    /// Installs `@ahpd/server` into the prefix with the bundled npm, sending
    /// each line npm writes to `line`.
    pub fn install(
        &self,
        line: impl Fn(String) + Send + Sync + Clone + 'static,
    ) -> Result<(), String> {
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
            env: vec![
                ("PATH".into(), node.path_var()),
                ("npm_config_update_notifier".into(), "false".into()),
                ("npm_config_fund".into(), "false".into()),
                ("npm_config_audit".into(), "false".into()),
            ],
        };
        let mut command = npm.command();
        command
            .arg("install")
            .arg("--prefix")
            .arg(&self.prefix)
            .arg("@ahpd/server@latest");
        run_streaming(command, line)
    }
}

/// Runs `command`, sending each line of its stdout and stderr as it comes,
/// and fails with its last lines when it exits non-zero.
fn run_streaming(
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
    let last = Arc::new(Mutex::new(Vec::<String>::new()));
    let pump = |pipe: Box<dyn Read + Send>| {
        let (line, last) = (line.clone(), last.clone());
        std::thread::spawn(move || {
            for text in BufReader::new(pipe).lines().map_while(Result::ok) {
                let mut kept = last.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                kept.push(text.clone());
                if kept.len() > 20 {
                    kept.remove(0);
                }
                drop(kept);
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
        return Ok(());
    }
    let kept = last.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    Err(format!("npm exited with {status}:\n{}", kept.join("\n")))
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

/// The ahpd as the shell sees it.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundView {
    pub path: PathBuf,
    pub origin: Origin,
    pub version: String,
    pub supported: bool,
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
        find(ahpd_path.as_deref(), installed, path_var, version)
            .map(|found| found.as_ref().map(FoundView::from))
    })
    .await
    .map_err(|err| err.to_string())?
}

/// One step of an install, as the shell shows it.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum InstallEvent {
    /// A line npm wrote.
    Line { text: String },
}

/// Installs ahpd with the bundled Node, then answers the ahpd the desk now uses.
#[tauri::command]
pub async fn install_local_ahpd(
    app: AppHandle,
    events: Channel<InstallEvent>,
) -> Result<Option<FoundView>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let local = app.state::<LocalAhpd>();
        local.install(move |text| {
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
        let error = local.install(|_| {}).unwrap_err();
        assert!(error.contains("no Node"), "{error}");
    }

    #[test]
    fn local_ahpd_a_failed_command_answers_its_last_lines() {
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
        let error = run_streaming(command, move |text| {
            sink.lock().unwrap().push(text.trim().to_string())
        })
        .unwrap_err();
        let mut heard = heard.lock().unwrap().clone();
        heard.sort();
        assert_eq!(heard, ["one", "two"]);
        assert!(error.contains("one") && error.contains("two"), "{error}");
    }

    /// A real install from npm, with the Node `pnpm stage:node` put in
    /// `resources/node`, into a scratch prefix.
    #[test]
    #[ignore = "needs the network and a staged Node"]
    fn local_ahpd_a_real_install() {
        let scratch = Scratch::new();
        let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/node");
        let local = LocalAhpd::at(scratch.0.join("ahpd"), BundledNode::in_dir(&resources));
        local.install(|text| println!("npm: {text}")).unwrap();
        let found = find(None, local.installed(), None, version)
            .unwrap()
            .unwrap();
        println!("{found:?}");
        assert_eq!(found.origin, Origin::Installed);
        assert!(found.supported());
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
