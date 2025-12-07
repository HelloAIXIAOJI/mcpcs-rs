use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use serde::{Deserialize, Serialize};
use colored::Colorize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceEntry {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    #[serde(flatten)]
    pub content: ResourceContent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResourceType {
    Text,
    File,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceContent {
    Text { content: String },
    File { path: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub resources: Vec<ResourceEntry>,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            resources: vec![
                ResourceEntry {
                    uri: "text://hello".to_string(),
                    name: "Hello".to_string(),
                    description: Some("Sample text resource".to_string()),
                    resource_type: ResourceType::Text,
                    content: ResourceContent::Text {
                        content: "Hello from MCP server!".to_string(),
                    },
                },
            ],
        }
    }
}

pub struct ResourceManager {
    config_path: PathBuf,
}

impl ResourceManager {
    pub fn new() -> Self {
        let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("./"));
        let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("./"));
        let config_path = exe_dir.join("resource.json");
        
        Self { config_path }
    }

    pub fn load_config(&self) -> anyhow::Result<ResourceConfig> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            // 创建默认配置
            let config = ResourceConfig::default();
            self.save_config(&config)?;
            Ok(config)
        }
    }

    pub fn save_config(&self, config: &ResourceConfig) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add_text_resource(
        &self,
        uri: String,
        name: String,
        description: Option<String>,
        content: String,
    ) -> anyhow::Result<()> {
        let mut config = self.load_config()?;
        
        // 检查 URI 是否已存在
        if config.resources.iter().any(|r| r.uri == uri) {
            return Err(anyhow::anyhow!("Resource with URI '{}' already exists", uri));
        }

        config.resources.push(ResourceEntry {
            uri,
            name,
            description,
            resource_type: ResourceType::Text,
            content: ResourceContent::Text { content },
        });

        self.save_config(&config)?;
        Ok(())
    }

    pub fn add_file_resource(
        &self,
        uri: String,
        name: String,
        description: Option<String>,
        file_path: String,
    ) -> anyhow::Result<()> {
        let mut config = self.load_config()?;
        
        // 检查文件是否存在
        if !Path::new(&file_path).exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }

        // 检查 URI 是否已存在
        if config.resources.iter().any(|r| r.uri == uri) {
            return Err(anyhow::anyhow!("Resource with URI '{}' already exists", uri));
        }

        config.resources.push(ResourceEntry {
            uri,
            name,
            description,
            resource_type: ResourceType::File,
            content: ResourceContent::File { path: file_path },
        });

        self.save_config(&config)?;
        Ok(())
    }

    pub fn remove_resource(&self, uri: &str) -> anyhow::Result<bool> {
        let mut config = self.load_config()?;
        let original_len = config.resources.len();
        config.resources.retain(|r| r.uri != uri);
        
        if config.resources.len() < original_len {
            self.save_config(&config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_resources(&self) -> anyhow::Result<Vec<ResourceEntry>> {
        Ok(self.load_config()?.resources)
    }

    pub fn get_resource(&self, uri: &str) -> anyhow::Result<Option<ResourceEntry>> {
        let config = self.load_config()?;
        Ok(config.resources.into_iter().find(|r| r.uri == uri))
    }

    pub fn get_resource_content(&self, entry: &ResourceEntry) -> anyhow::Result<(String, Option<String>)> {
        match &entry.content {
            ResourceContent::Text { content } => {
                Ok((content.clone(), Some("text/plain".to_string())))
            }
            ResourceContent::File { path } => {
                if !Path::new(path).exists() {
                    return Err(anyhow::anyhow!("File not found: {}", path));
                }
                
                let content = fs::read_to_string(path)?;
                let mime_type = detect_mime_type(Path::new(path));
                Ok((content, Some(mime_type)))
            }
        }
    }

    pub fn format_resource_list(&self, resources: &[ResourceEntry]) -> String {
        if resources.is_empty() {
            return format!("{}\n", "No resources configured".yellow());
        }

        let mut output = format!("{}\n", "📁 Available Resources:".cyan().bold());
        output.push_str(&format!("{} {}\n", "Config:".dimmed(), self.config_path.display()));
        output.push_str(&format!("{}\n", "━".repeat(60).dimmed()));

        for resource in resources {
            let type_str = match resource.resource_type {
                ResourceType::Text => "TEXT".green(),
                ResourceType::File => "FILE".blue(),
            };

            output.push_str(&format!(
                "{} {} {} {}\n",
                "•".blue(),
                resource.name.bold(),
                format!("[{}]", type_str),
                format!("({})", resource.uri).dimmed()
            ));

            if let Some(description) = &resource.description {
                output.push_str(&format!("  {}\n", description.dimmed()));
            }

            match &resource.content {
                ResourceContent::Text { content } => {
                    let preview = if content.len() > 100 {
                        format!("{}...", &content[..100])
                    } else {
                        content.clone()
                    };
                    output.push_str(&format!("  {} {}\n", "Content:".dimmed(), preview.replace('\n', " ")));
                }
                ResourceContent::File { path } => {
                    let status = if Path::new(path).exists() {
                        "✓".green()
                    } else {
                        "✗".red()
                    };
                    output.push_str(&format!("  {} {} {}\n", "File:".dimmed(), path, status));
                }
            }
            output.push('\n');
        }

        output
    }
}

fn detect_mime_type(path: &Path) -> String {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" => "text/plain".to_string(),
        "md" => "text/markdown".to_string(),
        "json" => "application/json".to_string(),
        "xml" => "application/xml".to_string(),
        "html" => "text/html".to_string(),
        "css" => "text/css".to_string(),
        "js" => "application/javascript".to_string(),
        "rs" => "text/x-rust".to_string(),
        "py" => "text/x-python".to_string(),
        "java" => "text/x-java".to_string(),
        "cpp" | "cc" | "cxx" => "text/x-c++".to_string(),
        "c" => "text/x-c".to_string(),
        "h" => "text/x-header".to_string(),
        _ => "text/plain".to_string(),
    }
}
