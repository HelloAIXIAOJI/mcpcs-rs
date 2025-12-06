use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use rand_core::OsRng;
use russh::server::{Msg, Server as _, Session};
use russh::{Channel, ChannelId, CryptoVec};
use colored::Colorize;

use crate::server::state::ServerState;
use crate::repl::Repl;

pub struct SshServer {
    state: ServerState,
}

impl SshServer {
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    pub async fn run(self, port: u16) -> anyhow::Result<()> {
        let config = russh::server::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keys: vec![
                russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519)?,
            ],
            ..Default::default()
        };
        let config = Arc::new(config);

        let mut sh = SshHandler {
            state: self.state.clone(),
            input_buffer: Arc::new(Mutex::new(String::new())),
        };

        eprintln!("{} {}", "SSH server listening on port:".green(), port.to_string().cyan().bold());
        eprintln!("{}", "Connect with: ssh localhost -p <port>".dimmed());

        let socket = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        sh.run_on_socket(config, &socket).await?;
        Ok(())
    }
}

#[derive(Clone)]
struct SshHandler {
    state: ServerState,
    input_buffer: Arc<Mutex<String>>,
}

impl russh::server::Server for SshHandler {
    type Handler = Self;

    fn new_client(&mut self, addr: Option<std::net::SocketAddr>) -> Self {
        if let Some(addr) = addr {
            eprintln!("{} {}", "SSH client connected:".green(), addr);
        }
        Self {
            state: self.state.clone(),
            input_buffer: Arc::new(Mutex::new(String::new())),
        }
    }

    fn handle_session_error(&mut self, error: <Self::Handler as russh::server::Handler>::Error) {
        eprintln!("{} {:#?}", "SSH session error:".red(), error);
    }
}

impl russh::server::Handler for SshHandler {
    type Error = russh::Error;

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn auth_none(&mut self, _user: &str) -> Result<russh::server::Auth, Self::Error> {
        Ok(russh::server::Auth::Accept)
    }

    async fn auth_password(
        &mut self,
        _user: &str,
        _password: &str,
    ) -> Result<russh::server::Auth, Self::Error> {
        Ok(russh::server::Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<russh::server::Auth, Self::Error> {
        Ok(russh::server::Auth::Accept)
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let repl = Repl::new(self.state.clone());
        let banner = repl.banner();
        let prompt = repl.prompt();
        
        session.data(channel, CryptoVec::from(banner.replace('\n', "\r\n")))?;
        session.data(channel, CryptoVec::from(prompt))?;
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut buffer = self.input_buffer.lock().await;
        
        for &byte in data {
            match byte {
                // Enter key
                b'\r' | b'\n' => {
                    session.data(channel, CryptoVec::from("\r\n"))?;
                    
                    let input = buffer.clone();
                    buffer.clear();
                    
                    let repl = Repl::new(self.state.clone());
                    let (response, should_exit) = repl.handle_input(&input).await;
                    
                    if !response.is_empty() {
                        let response = response.replace('\n', "\r\n");
                        session.data(channel, CryptoVec::from(response))?;
                    }
                    
                    if should_exit {
                        session.close(channel)?;
                    } else {
                        let prompt = repl.prompt();
                        session.data(channel, CryptoVec::from(prompt))?;
                    }
                }
                // Backspace
                127 | 8 => {
                    if !buffer.is_empty() {
                        buffer.pop();
                        session.data(channel, CryptoVec::from("\x08 \x08"))?;
                    }
                }
                // Ctrl+C
                3 => {
                    buffer.clear();
                    session.data(channel, CryptoVec::from("^C\r\n"))?;
                    let repl = Repl::new(self.state.clone());
                    let prompt = repl.prompt();
                    session.data(channel, CryptoVec::from(prompt))?;
                }
                // Ctrl+D
                4 => {
                    session.data(channel, CryptoVec::from(format!("\r\n{}\r\n", "Goodbye!".cyan())))?;
                    session.close(channel)?;
                }
                // Regular printable character
                32..=126 => {
                    buffer.push(byte as char);
                    session.data(channel, CryptoVec::from(vec![byte]))?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
