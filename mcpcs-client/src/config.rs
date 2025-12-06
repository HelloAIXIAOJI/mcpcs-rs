use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

impl McpConfig {
    pub fn load() -> Result<Self> {
        let mut config = McpConfig::default();
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".mcpcsrs").join("mcps");

        if !config_dir.exists() {
             return Ok(config);
        }

        let entries = std::fs::read_dir(config_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                // We assume the JSON file content structure matches McpConfig (has mcpServers key)
                // or is it just the map?
                // The prompt says "define MCP servers (one json can define multiple)".
                // Usually `mcpServers` is the key.
                match serde_json::from_str::<McpConfig>(&content) {
                    Ok(partial_config) => {
                        config.mcp_servers.extend(partial_config.mcp_servers);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn create_new(name: &str) -> Result<std::path::PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".mcpcsrs").join("mcps");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let mut path = config_dir.join(name);
        if path.extension().is_none() {
            path.set_extension("json");
        }
        
        if path.exists() {
             anyhow::bail!("File already exists: {}", path.display());
        }

        let empty_config = serde_json::json!({
            "mcpServers": {}
        });

        let content = serde_json::to_string_pretty(&empty_config)?;
        std::fs::write(&path, content)?;

        Ok(path)
    }
}
