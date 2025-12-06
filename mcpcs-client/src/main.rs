mod config;
mod client;

use std::io::{self, Write};
use anyhow::Result;
use crate::config::McpConfig;
use crate::client::ClientManager;
use tokio::io::{AsyncBufReadExt, BufReader};
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut manager = ClientManager::new();

    println!("{}", "mcpcs-client REPL".cyan().bold());
    println!("{}", "Commands:".yellow());
    println!("  {}              - Reload configuration from ~/.mcpcsrs/mcps/*.json", "/reload".green());
    println!("  {}            - List connected MCP servers", "/list mcp".green());
    println!("  {}           - List available tools from all servers", "/list tool".green());
    println!("  {} {} - Call a tool with JSON arguments (use server/tool for conflicts)", "/call".green(), "<tool> <json>".dimmed());
    println!("  {} {}    - Show detailed info about a tool", "/info tool".green(), "<name>".dimmed());
    println!("  {} {}    - Create a new empty MCP configuration file", "/newconfig".green(), "<name>".dimmed());
    println!("  {}                - Exit the REPL", "/exit".green());
    println!();

    println!("{}", "Loading configuration...".dimmed());
    match McpConfig::load() {
        Ok(config) => {
             manager.load_from_config(&config).await?;
        }
        Err(e) => {
             eprintln!("{} {}", "Failed to load config:".red(), e);
        }
    }

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("{} ", ">".cyan().bold());
        io::stdout().flush()?;
        
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0] {
            "/reload" => {
                println!("{}", "Reloading configuration...".dimmed());
                 match McpConfig::load() {
                    Ok(config) => {
                         manager.load_from_config(&config).await?;
                    }
                    Err(e) => {
                         eprintln!("{} {}", "Failed to load config:".red(), e);
                    }
                }
            }
            "/list" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "mcp" => {
                            let servers = manager.list_servers();
                            if servers.is_empty() {
                                println!("{}", "No connected servers.".yellow());
                            } else {
                                for s in servers {
                                    println!("{} {}", "-".dimmed(), s.cyan());
                                }
                            }
                        }
                        "tool" => {
                            if let Err(e) = manager.list_tools().await {
                                eprintln!("{} {:?}", "Error listing tools:".red(), e);
                            }
                        }
                        _ => println!("{}", "Unknown list command. Usage: /list mcp | /list tool".yellow()),
                    }
                } else {
                    println!("{}", "Usage: /list mcp | /list tool".yellow());
                }
            }
            "/call" => {
                if parts.len() < 2 {
                    println!("{}", "Usage: /call <tool_name> [json_args]".yellow());
                } else {
                    let tool_name = parts[1];
                    let args_start_index = input.find(tool_name).map(|i| i + tool_name.len()).unwrap_or(input.len());
                    let json_str = input[args_start_index..].trim();
                    
                    let json_arg = if json_str.is_empty() { "{}" } else { json_str };

                    match serde_json::from_str::<serde_json::Value>(json_arg) {
                        Ok(args) => {
                            if let Err(e) = manager.call_tool(tool_name, args).await {
                                eprintln!("{} {:?}", "Error calling tool:".red(), e);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", "Invalid JSON arguments:".red(), e);
                        }
                    }
                }
            }
            "/info" => {
                if parts.len() >= 3 && parts[1] == "tool" {
                    let tool_name = parts[2];
                    if let Err(e) = manager.tool_info(tool_name).await {
                        eprintln!("{} {:?}", "Error getting tool info:".red(), e);
                    }
                } else {
                    println!("{}", "Usage: /info tool <tool_name>".yellow());
                }
            }
            "/newconfig" => {
                if parts.len() < 2 {
                     println!("{}", "Usage: /newconfig <name>".yellow());
                } else {
                     let name = parts[1];
                     match McpConfig::create_new(name) {
                         Ok(path) => println!("{} {}", "Created config file:".green(), path.display()),
                         Err(e) => eprintln!("{} {}", "Failed to create config:".red(), e),
                     }
                }
            }
            "/exit" | "/quit" => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            _ => {
                println!("{} {}", "Unknown command:".yellow(), parts[0]);
            }
        }
    }

    Ok(())
}
