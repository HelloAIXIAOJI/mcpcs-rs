mod server;
mod ssh;
mod repl;

use anyhow::Result;
use rand::Rng;
use rmcp::ServiceExt;
use colored::Colorize;

use server::{McpServer, state::ServerState};
use ssh::SshServer;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Generate random port: 12312 + random(1-100)
    let random_offset: u16 = rand::thread_rng().gen_range(1..=100);
    let ssh_port = 12312 + random_offset;

    // Create shared state
    let state = ServerState::new(ssh_port);

    // Start SSH REPL server in background
    let ssh_state = state.clone();
    tokio::spawn(async move {
        let ssh_server = SshServer::new(ssh_state);
        if let Err(e) = ssh_server.run(ssh_port).await {
            eprintln!("{} {}", "SSH server error:".red(), e);
        }
    });

    // Start MCP server on stdio
    eprintln!("{}", "Starting MCP server on stdio...".green());
    eprintln!("{} {}", "SSH REPL port:".yellow(), ssh_port.to_string().cyan().bold());

    let mcp_server = McpServer::new(state);
    let service = mcp_server.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
    service.waiting().await?;

    Ok(())
}
