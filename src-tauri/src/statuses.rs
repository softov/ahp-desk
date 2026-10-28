//! Each host's status, as the host list window sees it, relayed to every
//! window. The host list holds one AHP connection per host and publishes the
//! map here; this keeps the last map for a window that opens later.
//!
//! Who computes a status is not part of the contract, so the watching can
//! move here, into Rust, without a change in any UI.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// The event each change sends, with every host's status.
pub const HOSTS_STATUS: &str = "hosts-status";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostStatus {
    Connected,
    Connecting,
    Down,
}

/// The last map the host list published.
#[derive(Default)]
pub struct HostStatuses(Mutex<HashMap<String, HostStatus>>);

impl HostStatuses {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, HostStatus>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn replace(&self, statuses: HashMap<String, HostStatus>) {
        *self.lock() = statuses;
    }

    pub fn all(&self) -> HashMap<String, HostStatus> {
        self.lock().clone()
    }

    /// Forgets a removed host.
    pub fn forget(&self, id: &str) {
        self.lock().remove(id);
    }
}

/// Keeps every host's status and sends it to every window. Only the host list may call it.
#[tauri::command]
pub fn publish_host_statuses(
    app: AppHandle,
    statuses: State<'_, HostStatuses>,
    map: HashMap<String, HostStatus>,
) -> Result<(), String> {
    statuses.replace(map.clone());
    app.emit(HOSTS_STATUS, map).map_err(|err| err.to_string())
}

/// Every host's status, as last published.
#[tauri::command]
pub fn host_statuses(statuses: State<'_, HostStatuses>) -> HashMap<String, HostStatus> {
    statuses.all()
}

/// The event each change of a running turn sends, with each host's running turns.
pub const HOSTS_RUNNING: &str = "hosts-running";

/// The titles of each host's chats with a turn in progress, as the host list
/// last published them; a host with none is absent. Kept apart from the
/// statuses, whose map of dots is ahpw's embedder contract (decision
/// `the-host-list-publishes-each-hosts-running-turns`).
#[derive(Default)]
pub struct HostRunning(Mutex<HashMap<String, Vec<String>>>);

impl HostRunning {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Vec<String>>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn replace(&self, running: HashMap<String, Vec<String>>) {
        *self.lock() = running;
    }

    pub fn all(&self) -> HashMap<String, Vec<String>> {
        self.lock().clone()
    }

    /// Forgets a removed host.
    pub fn forget(&self, id: &str) {
        self.lock().remove(id);
    }
}

/// Keeps each host's running turns and sends them to every window. Only the host list may call it.
#[tauri::command]
pub fn publish_host_running(
    app: AppHandle,
    running: State<'_, HostRunning>,
    map: HashMap<String, Vec<String>>,
) -> Result<(), String> {
    running.replace(map.clone());
    app.emit(HOSTS_RUNNING, map).map_err(|err| err.to_string())
}

/// Each host's running turns, as last published.
#[tauri::command]
pub fn host_running(running: State<'_, HostRunning>) -> HashMap<String, Vec<String>> {
    running.all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_keeps_the_last_map_and_forgets_a_removed_host() {
        let statuses = HostStatuses::default();
        statuses.replace(HashMap::from([
            ("home".to_string(), HostStatus::Connected),
            ("office".to_string(), HostStatus::Down),
        ]));
        assert_eq!(statuses.all().len(), 2);
        statuses.forget("office");
        assert_eq!(
            statuses.all(),
            HashMap::from([("home".to_string(), HostStatus::Connected)])
        );
    }

    #[test]
    fn statuses_reads_and_writes_the_words_a_ui_uses() {
        let parsed: HashMap<String, HostStatus> =
            serde_json::from_str(r#"{"a":"connected","b":"connecting","c":"down"}"#).unwrap();
        assert_eq!(parsed["b"], HostStatus::Connecting);
        assert!(serde_json::from_str::<HashMap<String, HostStatus>>(r#"{"a":"up"}"#).is_err());
    }

    #[test]
    fn running_keeps_the_last_map_and_forgets_a_removed_host() {
        let running = HostRunning::default();
        running.replace(HashMap::from([
            ("home".to_string(), vec!["Fix the build".to_string()]),
            ("office".to_string(), vec!["Plan".to_string(), "Review".to_string()]),
        ]));
        assert_eq!(running.all()["office"].len(), 2);
        running.forget("office");
        assert_eq!(
            running.all(),
            HashMap::from([("home".to_string(), vec!["Fix the build".to_string()])])
        );
        // A host with no running turn is not in the map.
        assert!(!running.all().contains_key("office"));
    }
}
