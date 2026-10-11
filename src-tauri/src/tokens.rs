//! Where host tokens live: the OS keyring first, and the host's own `token`
//! field in `hosts.json` when the keyring cannot be used, as on a Linux
//! desktop with no Secret Service.

use crate::hosts::Host;

/// A store of secrets by host id.
pub trait Vault: Send + Sync {
    fn set(&self, id: &str, token: &str) -> Result<(), String>;
    /// `Ok(None)` when the vault works and holds nothing for `id`.
    fn get(&self, id: &str) -> Result<Option<String>, String>;
    /// Deleting what is not there is not an error.
    fn delete(&self, id: &str) -> Result<(), String>;
}

/// The OS keyring: the Windows Credential Manager, the macOS Keychain, or the
/// Secret Service on Linux. Each token is one entry under the service `ahp-desk`.
pub struct Keyring;

const SERVICE: &str = "ahp-desk";

impl Vault for Keyring {
    fn set(&self, id: &str, token: &str) -> Result<(), String> {
        keyring::Entry::new(SERVICE, id)
            .and_then(|entry| entry.set_password(token))
            .map_err(|err| err.to_string())
    }

    fn get(&self, id: &str) -> Result<Option<String>, String> {
        match keyring::Entry::new(SERVICE, id).and_then(|entry| entry.get_password()) {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        match keyring::Entry::new(SERVICE, id).and_then(|entry| entry.delete_credential()) {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
}

/// Keeps `token` for `host`: in the vault, or in the host's `token` field when
/// the vault fails. An empty token deletes both copies.
pub fn set(vault: &dyn Vault, host: &mut Host, token: &str) {
    if token.is_empty() {
        // A keyring that cannot be reached holds nothing to delete.
        let _ = vault.delete(&host.id);
        host.token = None;
        return;
    }
    match vault.set(&host.id, token) {
        Ok(()) => host.token = None,
        Err(_) => host.token = Some(token.to_string()),
    }
}

/// The host's token: the vault's copy first, then the one in `hosts.json`.
pub fn get(vault: &dyn Vault, host: &Host) -> Option<String> {
    match vault.get(&host.id) {
        Ok(Some(token)) => Some(token),
        _ => host.token.clone(),
    }
}

/// Moves a token kept in `hosts.json` into the vault when the vault works.
/// Returns true when the host changed and the file needs writing.
pub fn move_to_vault(vault: &dyn Vault, host: &mut Host) -> bool {
    let Some(token) = host.token.clone() else {
        return false;
    };
    if vault.set(&host.id, &token).is_err() {
        return false;
    }
    host.token = None;
    true
}

/// The vaults the tests use: one in memory, and one that always fails, as a
/// machine with no keyring does.
#[cfg(test)]
pub mod test_vaults {
    use super::Vault;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    pub struct Memory(pub Mutex<HashMap<String, String>>);

    impl Vault for Memory {
        fn set(&self, id: &str, token: &str) -> Result<(), String> {
            self.0.lock().unwrap().insert(id.into(), token.into());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<Option<String>, String> {
            Ok(self.0.lock().unwrap().get(id).cloned())
        }
        fn delete(&self, id: &str) -> Result<(), String> {
            self.0.lock().unwrap().remove(id);
            Ok(())
        }
    }

    pub struct Broken;

    impl Vault for Broken {
        fn set(&self, _: &str, _: &str) -> Result<(), String> {
            Err("no keyring".into())
        }
        fn get(&self, _: &str) -> Result<Option<String>, String> {
            Err("no keyring".into())
        }
        fn delete(&self, _: &str) -> Result<(), String> {
            Err("no keyring".into())
        }
    }
}
