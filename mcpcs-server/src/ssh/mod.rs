use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use colored::Colorize;

use crate::server::state::ServerState;
use crate::repl::Repl;

pub struct TcpServer {
    state: ServerState,
}

impl TcpServer {
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    pub async fn run(self, port: u16) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        eprintln!("{} {}", "TCP REPL server listening on port:".green(), port.to_string().cyan().bold());
        eprintln!("{}", "Connect with: telnet localhost <port> or nc localhost <port>".dimmed());

        loop {
            let (socket, addr) = listener.accept().await?;
            let state = self.state.clone();
            
            tokio::spawn(async move {
                eprintln!("{} {}", "Client connected:".green(), addr);
                if let Err(e) = handle_client(socket, state).await {
                    eprintln!("{} {}: {}", "Client error".red(), addr, e);
                }
                eprintln!("{} {}", "Client disconnected:".yellow(), addr);
            });
        }
    }
}

async fn handle_client(
    socket: tokio::net::TcpStream,
    state: ServerState,
) -> anyhow::Result<()> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let repl = Repl::new(state);

    // Send banner
    writer.write_all(repl.banner().as_bytes()).await?;
    writer.write_all(repl.prompt().as_bytes()).await?;
    writer.flush().await?;

    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        let (response, should_exit) = repl.handle_input(&line).await;
        
        if !response.is_empty() {
            writer.write_all(response.as_bytes()).await?;
        }
        
        if should_exit {
            break;
        }
        
        writer.write_all(repl.prompt().as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
