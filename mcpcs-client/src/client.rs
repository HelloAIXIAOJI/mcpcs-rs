use std::collections::HashMap;
use crate::config::McpServerConfig;
use anyhow::Result;
use rmcp::{
    model::CallToolRequestParam,
    service::{RunningService, ServiceExt},
    transport::{ConfigureCommandExt, TokioChildProcess},
    RoleClient,
};
use tokio::process::Command;
use std::sync::Arc;
use colored::Colorize;

pub struct ClientManager {
    clients: HashMap<String, Arc<RunningService<RoleClient, ()>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }

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

    pub fn list_servers(&self) -> Vec<String> {
        let mut names: Vec<String> = self.clients.keys().cloned().collect();
        names.sort();
        names
    }

    pub async fn list_tools(&self) -> Result<()> {
        if self.clients.is_empty() {
            println!("{}", "No connected servers.".yellow());
            return Ok(());
        }
        
        let mut tool_to_servers: HashMap<String, Vec<String>> = HashMap::new();
        let mut server_tools: Vec<(String, Vec<(String, String)>)> = Vec::new();
        
        for (server_name, client) in &self.clients {
            let result = client.list_tools(Default::default()).await;
            match result {
                Ok(response) => {
                    let tools: Vec<(String, String)> = response.tools.iter()
                        .map(|t| (t.name.to_string(), t.description.clone().unwrap_or_default().to_string()))
                        .collect();
                    
                    for (tool_name, _) in &tools {
                        tool_to_servers.entry(tool_name.clone())
                            .or_default()
                            .push(server_name.clone());
                    }
                    server_tools.push((server_name.clone(), tools));
                }
                Err(e) => {
                    server_tools.push((server_name.clone(), vec![]));
                    eprintln!("{} {}: {}", "Error listing tools from".red(), server_name, e);
                }
            }
        }
        
        let conflicts: HashMap<&String, &Vec<String>> = tool_to_servers.iter()
            .filter(|(_, servers)| servers.len() > 1)
            .collect();
        
        if !conflicts.is_empty() {
            println!("{}", "WARNING: Tool name conflicts detected:".yellow().bold());
            for (tool_name, servers) in &conflicts {
                println!("  '{}' exists in: {}", tool_name.red(), servers.join(", ").cyan());
            }
            println!("  {}", "Use /call server_name/tool_name to specify which server.".dimmed());
            println!();
        }
        
        for (server_name, tools) in &server_tools {
            println!("{} {}", "Server:".bold(), server_name.cyan().bold());
            if tools.is_empty() {
                println!("  {}", "(No tools available)".dimmed());
            } else {
                for (tool_name, description) in tools {
                    if conflicts.contains_key(tool_name) {
                        println!("  {} {} {}: {}", "-".dimmed(), tool_name.green(), "[CONFLICT]".red().bold(), description.dimmed());
                    } else {
                        println!("  {} {}: {}", "-".dimmed(), tool_name.green(), description.dimmed());
                    }
                }
            }
        }
        Ok(())
    }

    /// Get detailed info about a tool. Supports optional server prefix.
    pub async fn tool_info(&self, tool_spec: &str) -> Result<()> {
        let (specified_server, tool_name) = if let Some(pos) = tool_spec.find('/') {
            (Some(&tool_spec[..pos]), &tool_spec[pos + 1..])
        } else {
            (None, tool_spec)
        };
        
        let mut found_tools: Vec<(String, rmcp::model::Tool)> = Vec::new();
        
        let servers_to_check: Vec<_> = if let Some(server_name) = specified_server {
            if let Some(client) = self.clients.get(server_name) {
                vec![(server_name.to_string(), client.clone())]
            } else {
                println!("{} '{}'", "Server not found:".yellow(), server_name);
                return Ok(());
            }
        } else {
            self.clients.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };
        
        for (server_name, client) in servers_to_check {
            if let Ok(response) = client.list_tools(Default::default()).await {
                for tool in response.tools {
                    if tool.name == tool_name {
                        found_tools.push((server_name.clone(), tool));
                    }
                }
            }
        }
        
        if found_tools.is_empty() {
            println!("{} '{}'", "Tool not found:".yellow(), tool_name);
            return Ok(());
        }
        
        for (server_name, tool) in &found_tools {
            println!("{}", format!("=== {} (from: {}) ===", tool.name, server_name).cyan().bold());
            if let Some(title) = &tool.title {
                println!("{} {}", "Title:".bold(), title);
            }
            if let Some(desc) = &tool.description {
                println!("{} {}", "Description:".bold(), desc);
            }
            println!("{}", "Input Schema:".bold());
            println!("{}", serde_json::to_string_pretty(&*tool.input_schema)?.dimmed());
            if let Some(output_schema) = &tool.output_schema {
                println!("{}", "Output Schema:".bold());
                println!("{}", serde_json::to_string_pretty(&**output_schema)?.dimmed());
            }
            if let Some(annotations) = &tool.annotations {
                println!("{}", "Annotations:".bold());
                if let Some(read_only) = annotations.read_only_hint {
                    let val = if read_only { "true".green() } else { "false".red() };
                    println!("  {}: {}", "read_only".dimmed(), val);
                }
                if let Some(destructive) = annotations.destructive_hint {
                    let val = if destructive { "true".red() } else { "false".green() };
                    println!("  {}: {}", "destructive".dimmed(), val);
                }
                if let Some(idempotent) = annotations.idempotent_hint {
                    let val = if idempotent { "true".green() } else { "false".yellow() };
                    println!("  {}: {}", "idempotent".dimmed(), val);
                }
                if let Some(open_world) = annotations.open_world_hint {
                    let val = if open_world { "true".yellow() } else { "false".dimmed() };
                    println!("  {}: {}", "open_world".dimmed(), val);
                }
            }
            println!();
        }
        Ok(())
    }

    /// Call a tool. Supports optional server prefix: "server_name/tool_name" or just "tool_name"
    pub async fn call_tool(&self, tool_spec: &str, args: serde_json::Value) -> Result<()> {
        let args_obj = args.as_object().cloned().unwrap_or_default();
        
        // Parse tool_spec: "server_name/tool_name" or just "tool_name"
        let (specified_server, tool_name) = if let Some(pos) = tool_spec.find('/') {
            (Some(&tool_spec[..pos]), &tool_spec[pos + 1..])
        } else {
            (None, tool_spec)
        };
        
        if let Some(server_name) = specified_server {
            if let Some(client) = self.clients.get(server_name) {
                return self.call_tool_on_server(server_name, client, tool_name, args_obj).await;
            } else {
                println!("{} '{}'", "Server not found:".yellow(), server_name);
                return Ok(());
            }
        }
        
        // No server specified - find which servers have this tool
        let mut servers_with_tool: Vec<String> = Vec::new();
        for (server_name, client) in &self.clients {
            if let Ok(response) = client.list_tools(Default::default()).await {
                if response.tools.iter().any(|t| t.name == tool_name) {
                    servers_with_tool.push(server_name.clone());
                }
            }
        }
        
        match servers_with_tool.len() {
            0 => {
                println!("{} '{}'", "Tool not found:".yellow(), tool_name);
            }
            1 => {
                let server_name = &servers_with_tool[0];
                let client = self.clients.get(server_name).unwrap();
                self.call_tool_on_server(server_name, client, tool_name, args_obj).await?;
            }
            _ => {
                println!("{} '{}' {}: {}", 
                    "Conflict:".yellow().bold(), 
                    tool_name.green(), 
                    "exists in multiple servers".yellow(),
                    servers_with_tool.join(", ").cyan());
                println!("{} /call {}/{} <args>", "Specify server:".dimmed(), "<server>".cyan(), tool_name.green());
            }
        }
        Ok(())
    }
    
    async fn call_tool_on_server(
        &self,
        server_name: &str,
        client: &RunningService<RoleClient, ()>,
        tool_name: &str,
        args_obj: serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        println!("{} '{}' {} '{}'...", 
            "Calling".dimmed(), tool_name.green(), "on".dimmed(), server_name.cyan());
        
        let result = client.call_tool(CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: Some(args_obj),
        }).await?;
        
        println!("{} {}:", "Result from".bold(), server_name.cyan());
        for content in result.content {
            println!("{}", serde_json::to_string_pretty(&content)?);
        }
        if let Some(err) = result.is_error {
            if err {
                println!("{}", "(Tool reported an error state)".red());
            }
        }
        Ok(())
    }
}
