mod server;
mod ssh;
mod repl;
mod resources;
mod prompts;
mod sse;

use anyhow::Result;
use rand::Rng;
use rmcp::ServiceExt;
use colored::Colorize;
use clap::{Parser, Subcommand};

use server::{McpServer, state::ServerState};
use ssh::SshServer;
use sse::SseApp;

#[derive(Parser)]
#[command(name = "mcpcs-server")]
#[command(about = "MCP Server with multiple transport options")]
struct Cli {
    #[command(subcommand)]
    mode: Option<ServerMode>,
}

#[derive(Subcommand)]
enum ServerMode {
    /// Run server with SSE transport (HTTP Server-Sent Events)
    Sse {
        /// SSE server port (default: 12121)
        #[arg(short, long, default_value = "12121")]
        port: u16,
    },
    /// Run server with stdio transport (default mode)
    Stdio,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

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

    // Create MCP server
    let mcp_server = McpServer::new(state);

    match cli.mode.unwrap_or(ServerMode::Stdio) {
        ServerMode::Sse { port } => {
            // Start SSE server
            eprintln!("{} {}", "SSH REPL port:".yellow(), ssh_port.to_string().cyan().bold());
            let sse_app = SseApp::new(mcp_server, port);
            sse_app.run().await?;
        }
        ServerMode::Stdio => {
            // Start MCP server on stdio (default behavior)
            eprintln!("{}", "Starting MCP server on stdio...".green());
            eprintln!("{} {}", "SSH REPL port:".yellow(), ssh_port.to_string().cyan().bold());

            let service = mcp_server.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
            service.waiting().await?;
        }
    }

    Ok(())
}
