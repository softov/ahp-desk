//! Which server keys the desk trusts: one in `~/.ssh/known_hosts` for the
//! host, or one the person confirmed before, kept in `known_keys.json` beside
//! `hosts.json`. A new key refuses the connection until the person trusts it;
//! a changed one refuses it naming both fingerprints.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use russh::keys::ssh_key::{Algorithm, HashAlg, PublicKey};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::ServerCheck;
use crate::hosts::write_json;

/// A key as OpenSSH shows it: its type and `SHA256:<base64>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fingerprint {
    pub key_type: String,
    pub sha256: String,
}

pub fn fingerprint(key: &PublicKey) -> Fingerprint {
    let key_type = match key.algorithm() {
        Algorithm::Ed25519 => "ED25519".to_string(),
        Algorithm::Rsa { .. } => "RSA".to_string(),
        Algorithm::Ecdsa { .. } => "ECDSA".to_string(),
        other => other.as_str().to_uppercase(),
    };
    Fingerprint {
        key_type,
        sha256: key.fingerprint(HashAlg::Sha256).to_string(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyCheck {
    Known,
    New {
        offered: Fingerprint,
    },
    /// The server offers another key than the one known for it; the old one
    /// came from `known_hosts` when `in_known_hosts`, else from the desk.
    Changed {
        known: Fingerprint,
        offered: Fingerprint,
        in_known_hosts: bool,
    },
}

/// One confirmed key in `known_keys.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    /// `host:port`.
    server: String,
    key_type: String,
    /// The key in OpenSSH form.
    key: String,
    fingerprint: String,
    /// When the person confirmed it, in seconds since 1970.
    confirmed: u64,
}

#[derive(Clone)]
pub struct KnownKeys(Arc<Inner>);

struct Inner {
    path: PathBuf,
    known_hosts: PathBuf,
    entries: Mutex<Vec<Entry>>,
    /// The last key refused per `host:port`, for this process only: the one
    /// Trust records, once its fingerprint matches what the person saw.
    refused: Mutex<HashMap<String, PublicKey>>,
}

fn server(address: &str, port: u16) -> String {
    format!("{address}:{port}")
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl KnownKeys {
    /// Reads `path`, or starts empty when it does not exist; `known_hosts` is
    /// read at each check, so a key the person adds with `ssh` counts at once.
    pub fn load(path: PathBuf, known_hosts: PathBuf) -> Result<Self, String> {
        let entries: Vec<Entry> = match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|err| format!("{} is not a list of server keys: {err}", path.display()))?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
        };
        Ok(KnownKeys(Arc::new(Inner {
            path,
            known_hosts,
            entries: Mutex::new(entries),
            refused: Mutex::new(HashMap::new()),
        })))
    }

    /// What `known_hosts` and the desk say of `key` for the server. Only a key
    /// of the same type is compared, as OpenSSH and `russh` do.
    pub fn check(&self, address: &str, port: u16, key: &PublicKey) -> KeyCheck {
        let offered = fingerprint(key);
        let from_known_hosts =
            russh::keys::known_hosts::known_host_keys_path(address, port, &self.0.known_hosts)
                .unwrap_or_else(|err| {
                    // A line russh cannot read fails the whole file; the desk's own
                    // keys still count, and an unknown key is asked, never let through.
                    eprintln!(
                        "ahp-desk: {} is not readable as known_hosts: {err}",
                        self.0.known_hosts.display()
                    );
                    Vec::new()
                })
                .into_iter()
                .map(|(_, recorded)| (recorded, true));
        let from_desk = lock(&self.0.entries)
            .iter()
            .filter(|entry| entry.server == server(address, port))
            .filter_map(|entry| PublicKey::from_openssh(&entry.key).ok())
            .map(|recorded| (recorded, false))
            .collect::<Vec<_>>();
        let mut changed = None;
        for (recorded, in_known_hosts) in from_known_hosts.chain(from_desk) {
            if recorded.algorithm() != key.algorithm() {
                continue;
            }
            if recorded.key_data() == key.key_data() {
                return KeyCheck::Known;
            }
            // known_hosts names the old key first, as it is the one `ssh` uses.
            changed.get_or_insert((fingerprint(&recorded), in_known_hosts));
        }
        match changed {
            Some((known, in_known_hosts)) => KeyCheck::Changed {
                known,
                offered,
                in_known_hosts,
            },
            None => KeyCheck::New { offered },
        }
    }

    /// The check a connection to the server uses: it answers at once, and keeps
    /// a refused key for `confirm`.
    pub fn checker(&self, address: &str, port: u16) -> ServerCheck {
        let known = self.clone();
        let address = address.to_string();
        Box::new(move |key| {
            let answer = known.check(&address, port, key);
            if answer != KeyCheck::Known {
                lock(&known.0.refused).insert(server(&address, port), key.clone());
            }
            answer
        })
    }

    /// Trusts the key last refused for the server, only when its fingerprint
    /// is the one the person was shown.
    pub fn confirm(&self, address: &str, port: u16, sha256: &str) -> Result<(), String> {
        let at = server(address, port);
        let key = lock(&self.0.refused)
            .get(&at)
            .cloned()
            .ok_or_else(|| format!("no key from {at} waits to be trusted"))?;
        let seen = fingerprint(&key);
        if seen.sha256 != sha256 {
            return Err(format!("{at} offered {}, not {sha256}", seen.sha256));
        }
        let mut entries = lock(&self.0.entries);
        entries.retain(|entry| entry.server != at);
        entries.push(Entry {
            server: at.clone(),
            key_type: seen.key_type,
            key: key.to_openssh().map_err(|err| err.to_string())?,
            fingerprint: seen.sha256,
            confirmed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |since| since.as_secs()),
        });
        write_json(&self.0.path, &entries[..])?;
        lock(&self.0.refused).remove(&at);
        Ok(())
    }

    /// Forgets the desk's key for the server, so the next connection asks to
    /// trust its key as a first one. `known_hosts` is never edited.
    pub fn forget(&self, address: &str, port: u16) -> Result<(), String> {
        let at = server(address, port);
        let mut entries = lock(&self.0.entries);
        entries.retain(|entry| entry.server != at);
        write_json(&self.0.path, &entries[..])
    }
}

#[tauri::command]
pub fn confirm_ssh_key(
    known: State<'_, KnownKeys>,
    address: String,
    port: u16,
    fingerprint: String,
) -> Result<(), String> {
    known.confirm(&address, port, &fingerprint)
}

#[tauri::command]
pub fn forget_ssh_key(
    known: State<'_, KnownKeys>,
    address: String,
    port: u16,
) -> Result<(), String> {
    known.forget(&address, port)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use russh::keys::PrivateKey;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A folder for `known_keys.json` and a `known_hosts`, gone at the end of the test.
    pub struct Scratch(pub PathBuf);
    impl Scratch {
        pub fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "ahp-desk-keys-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
        pub fn known(&self) -> KnownKeys {
            KnownKeys::load(self.0.join("known_keys.json"), self.0.join("known_hosts")).unwrap()
        }
        pub fn known_hosts(&self, line: &str) {
            fs::write(self.0.join("known_hosts"), format!("{line}\n")).unwrap();
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn key() -> PublicKey {
        PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519)
            .unwrap()
            .public_key()
            .clone()
    }

    fn line(host: &str, key: &PublicKey) -> String {
        format!("{host} {}", key.to_openssh().unwrap())
    }

    /// Refuses `key` through the checker, as a connection would, then trusts it.
    fn trust(known: &KnownKeys, address: &str, port: u16, key: &PublicKey) {
        assert_ne!(known.checker(address, port)(key), KeyCheck::Known);
        known
            .confirm(address, port, &fingerprint(key).sha256)
            .unwrap();
    }

    #[test]
    fn known_keys_a_fingerprint_reads_as_openssh_prints_it() {
        let print = fingerprint(&key());
        assert_eq!(print.key_type, "ED25519");
        assert!(print.sha256.starts_with("SHA256:"));
        assert!(!print.sha256.ends_with('='));
    }

    #[test]
    fn known_keys_a_key_in_known_hosts_passes_and_a_missing_file_is_not_an_error() {
        let scratch = Scratch::new();
        let server_key = key();
        assert_eq!(
            scratch.known().check("dev", 22, &server_key),
            KeyCheck::New {
                offered: fingerprint(&server_key)
            }
        );
        scratch.known_hosts(&line("dev", &server_key));
        assert_eq!(
            scratch.known().check("dev", 22, &server_key),
            KeyCheck::Known
        );
        scratch.known_hosts(&line("[dev]:2222", &server_key));
        assert_eq!(
            scratch.known().check("dev", 2222, &server_key),
            KeyCheck::Known
        );
    }

    #[test]
    fn known_keys_a_confirmed_key_passes_and_is_kept() {
        let scratch = Scratch::new();
        let server_key = key();
        trust(&scratch.known(), "dev", 22, &server_key);
        assert_eq!(
            scratch.known().check("dev", 22, &server_key),
            KeyCheck::Known
        );
        assert_eq!(
            scratch.known().check("dev", 2200, &server_key),
            KeyCheck::New {
                offered: fingerprint(&server_key)
            },
            "a key is trusted for its host and port only"
        );
    }

    #[test]
    fn known_keys_a_different_key_is_changed_and_says_where_the_old_one_was() {
        let scratch = Scratch::new();
        let (old, new) = (key(), key());
        scratch.known_hosts(&line("dev", &old));
        assert_eq!(
            scratch.known().check("dev", 22, &new),
            KeyCheck::Changed {
                known: fingerprint(&old),
                offered: fingerprint(&new),
                in_known_hosts: true
            }
        );
        let desk = Scratch::new();
        trust(&desk.known(), "dev", 22, &old);
        assert_eq!(
            desk.known().check("dev", 22, &new),
            KeyCheck::Changed {
                known: fingerprint(&old),
                offered: fingerprint(&new),
                in_known_hosts: false
            }
        );
    }

    #[test]
    fn known_keys_confirm_takes_only_the_fingerprint_that_was_shown() {
        let scratch = Scratch::new();
        let known = scratch.known();
        let (seen, other) = (key(), key());
        assert!(
            known
                .confirm("dev", 22, &fingerprint(&seen).sha256)
                .is_err(),
            "nothing refused yet"
        );
        known.checker("dev", 22)(&seen);
        assert!(known
            .confirm("dev", 22, &fingerprint(&other).sha256)
            .is_err());
        assert_ne!(known.check("dev", 22, &seen), KeyCheck::Known);
        known
            .confirm("dev", 22, &fingerprint(&seen).sha256)
            .unwrap();
        assert_eq!(known.check("dev", 22, &seen), KeyCheck::Known);
    }

    #[test]
    fn known_keys_forget_makes_the_next_check_new_and_leaves_known_hosts() {
        let scratch = Scratch::new();
        let (old, new) = (key(), key());
        let known = scratch.known();
        trust(&known, "dev", 22, &old);
        known.forget("dev", 22).unwrap();
        assert_eq!(
            known.check("dev", 22, &new),
            KeyCheck::New {
                offered: fingerprint(&new)
            }
        );
        scratch.known_hosts(&line("dev", &old));
        known.forget("dev", 22).unwrap();
        assert!(matches!(
            known.check("dev", 22, &new),
            KeyCheck::Changed {
                in_known_hosts: true,
                ..
            }
        ));
    }

    #[test]
    fn known_keys_an_unreadable_known_hosts_leaves_the_desks_keys() {
        let scratch = Scratch::new();
        let server_key = key();
        scratch.known_hosts("@cert-authority *.example ssh-ed25519 not-base64");
        assert!(matches!(
            scratch.known().check("dev", 22, &server_key),
            KeyCheck::New { .. }
        ));
        trust(&scratch.known(), "dev", 22, &server_key);
        assert_eq!(
            scratch.known().check("dev", 22, &server_key),
            KeyCheck::Known
        );
    }
}
