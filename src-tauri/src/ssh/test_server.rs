//! An SSH server in the test process, on `127.0.0.1:0`, with a host key made
//! for it: it takes one password and one public key, and answers a few
//! commands with canned output.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use russh::keys::ssh_key::{Cipher, Kdf, LineEnding};
use russh::keys::{Algorithm, PrivateKey, PublicKey};
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId};
use tokio::net::TcpListener;

use super::SshTarget;

pub const USER: &str = "tester";
pub const PASSWORD: &str = "right horse";

pub struct TestServer {
    port: u16,
    /// The one key the server takes, which `key_file` writes out.
    client_key: PrivateKey,
}

impl TestServer {
    pub async fn start() -> Self {
        let client_key = random_key();
        let config = Arc::new(server::Config {
            keys: vec![random_key()],
            auth_rejection_time: std::time::Duration::from_millis(1),
            auth_rejection_time_initial: Some(std::time::Duration::ZERO),
            // The developer's agent may hold many keys, each one an attempt.
            max_auth_attempts: 100,
            ..Default::default()
        });
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let accepted = client_key.public_key().clone();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let handler = Handler {
                    accepted: accepted.clone(),
                };
                if let Ok(running) = server::run_stream(config.clone(), stream, handler).await {
                    tokio::spawn(running);
                }
            }
        });
        TestServer { port, client_key }
    }

    pub fn target(&self, key_file: Option<&Path>) -> SshTarget {
        SshTarget::new(
            "127.0.0.1",
            self.port,
            USER,
            key_file.map(Path::to_path_buf),
        )
    }

    /// The accepted key in an OpenSSH file, encrypted with `passphrase` when given.
    pub fn key_file(&self, passphrase: Option<&str>) -> KeyFile {
        let key = match passphrase {
            // One bcrypt round, so a debug build decrypts it at once.
            Some(passphrase) => self
                .client_key
                .encrypt_with(
                    Cipher::Aes256Ctr,
                    Kdf::Bcrypt {
                        salt: vec![7; 16],
                        rounds: 1,
                    },
                    0x5eed,
                    passphrase,
                )
                .unwrap(),
            None => self.client_key.clone(),
        };
        let file = KeyFile::new();
        std::fs::write(&file.0, key.to_openssh(LineEnding::LF).unwrap().as_bytes()).unwrap();
        file
    }
}

fn random_key() -> PrivateKey {
    PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519).unwrap()
}

/// A key file in the temp folder, gone at the end of the test.
pub struct KeyFile(PathBuf);

impl KeyFile {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        KeyFile(std::env::temp_dir().join(format!(
            "ahp-desk-key-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for KeyFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Handler {
    accepted: PublicKey,
}

impl server::Handler for Handler {
    type Error = russh::Error;

    async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error> {
        Ok(if user == USER && password == PASSWORD {
            Auth::Accept
        } else {
            Auth::reject()
        })
    }

    async fn auth_publickey(&mut self, user: &str, key: &PublicKey) -> Result<Auth, Self::Error> {
        Ok(
            if user == USER && key.key_data() == self.accepted.key_data() {
                Auth::Accept
            } else {
                Auth::reject()
            },
        )
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let home = format!("/home/{USER}");
        // (stdout, stderr, exit code); the order the client reads them is the order sent.
        let (out, err, code): (&[&str], &[&str], u32) = match data {
            b"printf %s \"$HOME\"" => (&[home.as_str()], &[], 0),
            b"lines" => (&["one\ntw", "o\n"], &["warn\n"], 0),
            b"fail" => (&["partial\n"], &["bad\n"], 3),
            _ => (&[], &["unknown command\n"], 127),
        };
        session.channel_success(channel)?;
        for chunk in out {
            session.data(channel, chunk.as_bytes().to_vec())?;
        }
        for chunk in err {
            session.extended_data(channel, 1, chunk.as_bytes().to_vec())?;
        }
        if data == b"lines" {
            session.data(channel, b"three".to_vec())?;
        }
        session.exit_status_request(channel, code)?;
        session.eof(channel)?;
        session.close(channel)?;
        Ok(())
    }
}
