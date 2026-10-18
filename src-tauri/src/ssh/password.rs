//! An SSH password the person asked to keep: in the OS keyring under
//! `ssh-password:<host id>`, beside the host's token, or in the host's
//! `ssh.password` in `hosts.json` when there is no keyring and the person
//! ticked the box that says so. A passphrase is never kept.

use tauri::State;

use crate::hosts::{Host, HostStore};
use crate::tokens::Vault;

fn entry(id: &str) -> String {
    format!("ssh-password:{id}")
}

/// Whether the vault can be used: a read answers, with or without an entry.
pub fn keyring_works(vault: &dyn Vault) -> bool {
    vault.get("ssh-keyring-check").is_ok()
}

/// Keeps `password` for `host`: in the vault, or in `host.ssh.password` when
/// the vault fails. The form calls it on a failing vault only when the person
/// ticked the box that names the plain file.
pub fn keep(vault: &dyn Vault, host: &mut Host, password: &str) {
    let Some(ssh) = host.ssh.as_mut() else {
        return;
    };
    match vault.set(&entry(&host.id), password) {
        Ok(()) => ssh.password = None,
        Err(_) => ssh.password = Some(password.to_string()),
    }
}

/// The kept password: the vault's copy first, then the one in `hosts.json`.
pub fn kept(vault: &dyn Vault, host: &Host) -> Option<String> {
    match vault.get(&entry(&host.id)) {
        Ok(Some(password)) => Some(password),
        _ => host.ssh.as_ref().and_then(|ssh| ssh.password.clone()),
    }
}

/// Deletes the vault's copy of `id`'s password; the copy in the file goes with its host.
pub fn forget(vault: &dyn Vault, id: &str) {
    // A keyring that cannot be reached holds no copy to delete.
    let _ = vault.delete(&entry(id));
}

/// Moves a password kept in `hosts.json` into the vault when the vault works.
/// Returns true when the host changed and the file needs writing.
pub fn move_to_vault(vault: &dyn Vault, host: &mut Host) -> bool {
    let Some(password) = host.ssh.as_ref().and_then(|ssh| ssh.password.clone()) else {
        return false;
    };
    if vault.set(&entry(&host.id), &password).is_err() {
        return false;
    }
    if let Some(ssh) = host.ssh.as_mut() {
        ssh.password = None;
    }
    true
}

/// Whether a kept password would go in the keyring, so the form says where.
#[tauri::command]
pub fn ssh_keyring_works(store: State<'_, HostStore>) -> bool {
    store.keyring_works()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosts::{HostKind, OnDesk, Reach, SshHost};
    use crate::tokens::test_vaults::{Broken, Memory};

    fn host() -> Host {
        Host {
            id: "dev".into(),
            name: "dev02".into(),
            kind: HostKind::Url,
            url: "ws://127.0.0.1:9187".into(),
            ahpd_path: None,
            token: None,
            started_by_desk: false,
            ssh: Some(SshHost {
                address: "dev02".into(),
                port: 22,
                user: "me".into(),
                sign_in: None,
                key_file: None,
                reach: Reach::Port,
                remote_port: 9187,
                folder: None,
                by_desk: false,
                local_port: None,
                password: None,
            }),
            on_desk: OnDesk::default(),
        }
    }

    #[test]
    fn password_is_kept_in_the_vault_and_not_in_the_host() {
        let vault = Memory::default();
        let mut host = host();
        keep(&vault, &mut host, "pw");
        assert_eq!(host.ssh.as_ref().unwrap().password, None);
        assert_eq!(kept(&vault, &host).as_deref(), Some("pw"));
        assert_eq!(
            vault.get("ssh-password:dev").unwrap().as_deref(),
            Some("pw")
        );
    }

    #[test]
    fn password_with_no_keyring_is_kept_in_the_host() {
        let mut host = host();
        keep(&Broken, &mut host, "pw");
        assert_eq!(host.ssh.as_ref().unwrap().password.as_deref(), Some("pw"));
        assert_eq!(kept(&Broken, &host).as_deref(), Some("pw"));
    }

    #[test]
    fn password_is_forgotten_and_moved_to_a_vault_that_works() {
        let vault = Memory::default();
        let mut host = host();
        keep(&Broken, &mut host, "pw");
        assert!(move_to_vault(&vault, &mut host));
        assert_eq!(host.ssh.as_ref().unwrap().password, None);
        assert_eq!(kept(&vault, &host).as_deref(), Some("pw"));
        assert!(!move_to_vault(&vault, &mut host), "nothing left to move");
        forget(&vault, "dev");
        assert_eq!(kept(&vault, &host), None);
        forget(&vault, "dev");
    }

    #[test]
    fn password_keyring_works_answers_from_a_read() {
        assert!(keyring_works(&Memory::default()));
        assert!(!keyring_works(&Broken));
    }
}
