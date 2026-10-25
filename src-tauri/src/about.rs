//! What the About screen shows: the desk's version, the ahpw it carries and the
//! Node it ships.

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::local_ahpd::{Launcher, LocalAhpd};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct About {
    desk: String,
    /// Baked in at build from the staged `@ahpd/web`; none when the build found none.
    ahpw: Option<String>,
    /// The bundled Node's own answer, so About shows what ships; none when no Node ships.
    node: Option<String>,
}

#[tauri::command]
pub async fn about(app: AppHandle, local: State<'_, LocalAhpd>) -> Result<About, String> {
    let node = local.node.clone();
    let node = tauri::async_runtime::spawn_blocking(move || {
        let launcher = Launcher {
            program: node?.node,
            before: Vec::new(),
            env: Vec::new(),
        };
        let output = launcher.command().arg("--version").output().ok()?;
        if !output.status.success() {
            return None;
        }
        node_version(&String::from_utf8_lossy(&output.stdout))
    })
    .await
    .map_err(|err| err.to_string())?;
    Ok(About {
        desk: app.package_info().version.to_string(),
        ahpw: option_env!("AHP_DESK_AHPW_VERSION")
            .filter(|version| !version.is_empty())
            .map(str::to_string),
        node,
    })
}

/// `v22.23.3` as Node prints it, without its `v`.
fn node_version(stdout: &str) -> Option<String> {
    let version = stdout.trim().trim_start_matches('v');
    (!version.is_empty()).then(|| version.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_node_version_drops_the_v_and_the_newline() {
        assert_eq!(node_version("v22.23.3\n"), Some("22.23.3".to_string()));
        assert_eq!(node_version("  \n"), None);
    }
}
