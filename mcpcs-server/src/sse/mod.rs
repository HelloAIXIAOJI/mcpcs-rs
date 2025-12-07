use anyhow::Result;
use colored::Colorize;
use rmcp::transport::sse_server::SseServer;
use rmcp::ServiceExt;

use crate::server::McpServer;

pub struct SseApp {
    pub mcp_server: McpServer,
    pub port: u16,
}

impl SseApp {
    pub fn new(mcp_server: McpServer, port: u16) -> Self {
        Self { mcp_server, port }
    }

    pub async fn run(self) -> Result<()> {
        eprintln!(
            "{} {}",
            "Starting SSE server on:".green(),
            format!("http://localhost:{}/sse", self.port).cyan().bold()
        );

        // Use the simpler serve_with_config method
        let bind_addr = format!("0.0.0.0:{}", self.port);
        let mut sse_server = SseServer::serve_with_config(rmcp::transport::sse_server::SseServerConfig {
            bind: bind_addr.parse()?,
            sse_path: "/sse".to_string(),
            post_path: "/message".to_string(),
            sse_keep_alive: Some(std::time::Duration::from_secs(15)),
            ct: tokio_util::sync::CancellationToken::new(),
        }).await?;
        
        // Process incoming SSE connections
        while let Some(transport) = sse_server.next_transport().await {
            eprintln!("{}", "New SSE client connected".green());
            
            // Clone MCP server for this connection
            let mcp_server = self.mcp_server.clone();
            
            // Spawn task to handle this SSE connection
            tokio::spawn(async move {
                match mcp_server.serve(transport).await {
                    Ok(service) => {
                        eprintln!("{}", "SSE MCP service started".green());
                        if let Err(e) = service.waiting().await {
                            eprintln!("{} {}", "SSE service error:".red(), e);
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {}", "Failed to start SSE MCP service:".red(), e);
                    }
                }
                eprintln!("{}", "SSE client disconnected".yellow());
            });
        }
        
        Ok(())
    }
}
