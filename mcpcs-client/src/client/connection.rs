use crate::config::McpServerConfig;
use anyhow::Result;
use colored::Colorize;
use rmcp::{
    service::{RunningService, ServiceExt},
    transport::{ConfigureCommandExt, TokioChildProcess},
    RoleClient,
};
use std::sync::Arc;
use tokio::process::Command;

use super::ClientManager;

impl ClientManager {
    pub async fn load_from_config(&mut self, config: &crate::config::McpConfig) -> Result<()> {
        self.clients.clear();
        for (name, server_conf) in &config.mcp_servers {
            match self.connect(server_conf).await {
                Ok(client) => {
                    self.clients.insert(name.clone(), Arc::new(client));
                    println!("{} {}", "Connected:".green(), name.cyan());
                }
                Err(e) => {
                    eprintln!("{} '{}': {:#}", "Failed to connect:".red(), name, e);
                }
            }
        }
        Ok(())
    }

    async fn connect(&self, config: &McpServerConfig) -> Result<RunningService<RoleClient, ()>> {
        let cmd = Command::new(&config.command);
        let args = config.args.clone();
        let env = config.env.clone();

        let transport = TokioChildProcess::new(cmd.configure(move |c| {
            c.args(&args);
            if let Some(e) = &env {
                c.envs(e);
            }
        }))?;

        let client = ().serve(transport).await?;
        Ok(client)
    }
}
