mod server;
mod ssh;
mod repl;

use anyhow::Result;
use rand::Rng;
use rmcp::ServiceExt;
use colored::Colorize;

use server::{McpServer, state::ServerState};
use ssh::TcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Generate random port: 12312 + random(1-100)
    let random_offset: u16 = rand::thread_rng().gen_range(1..=100);
    let tcp_port = 12312 + random_offset;

    // Create shared state
    let state = ServerState::new(tcp_port);

    // Start TCP REPL server in background
    let tcp_state = state.clone();
    tokio::spawn(async move {
        let tcp_server = TcpServer::new(tcp_state);
        if let Err(e) = tcp_server.run(tcp_port).await {
            eprintln!("{} {}", "TCP server error:".red(), e);
        }
    });

    // Start MCP server on stdio
    eprintln!("{}", "Starting MCP server on stdio...".green());
    eprintln!("{} {}", "TCP REPL port:".yellow(), tcp_port.to_string().cyan().bold());

    let mcp_server = McpServer::new(state);
    let service = mcp_server.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
    service.waiting().await?;

    Ok(())
}
