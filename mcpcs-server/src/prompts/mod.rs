use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use serde::{Deserialize, Serialize};
use colored::Colorize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptEntry {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
    pub template: String, // 模板字符串，支持 {{argument}} 替换
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    #[serde(rename = "type")]
    pub arg_type: String, // "string", "number", "boolean"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptConfig {
    pub prompts: Vec<PromptEntry>,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            prompts: vec![
                PromptEntry {
                    name: "code_review".to_string(),
                    title: Some("AI Code Review".to_string()),
                    description: Some("Analyze code and provide improvement suggestions".to_string()),
                    arguments: vec![
                        PromptArgument {
                            name: "code".to_string(),
                            description: Some("The code to review".to_string()),
                            required: true,
                            arg_type: "string".to_string(),
                        },
                        PromptArgument {
                            name: "language".to_string(),
                            description: Some("Programming language (optional)".to_string()),
                            required: false,
                            arg_type: "string".to_string(),
                        },
                    ],
                    template: "Please review this {{language}} code and provide suggestions for improvement:\n\n{{code}}\n\nFocus on:\n- Code quality and readability\n- Performance optimizations\n- Best practices\n- Potential bugs".to_string(),
                },
                PromptEntry {
                    name: "explain_code".to_string(),
                    title: Some("Code Explainer".to_string()),
                    description: Some("Explain how code works in detail".to_string()),
                    arguments: vec![
                        PromptArgument {
                            name: "code".to_string(),
                            description: Some("The code to explain".to_string()),
                            required: true,
                            arg_type: "string".to_string(),
                        },
                    ],
                    template: "Please explain how this code works step by step:\n\n{{code}}\n\nBreak it down into:\n- What each part does\n- The overall logic flow\n- Any important concepts".to_string(),
                },
                PromptEntry {
                    name: "summarize".to_string(),
                    title: Some("Document Summarizer".to_string()),
                    description: Some("Create a concise summary of text content".to_string()),
                    arguments: vec![
                        PromptArgument {
                            name: "content".to_string(),
                            description: Some("The content to summarize".to_string()),
                            required: true,
                            arg_type: "string".to_string(),
                        },
                        PromptArgument {
                            name: "style".to_string(),
                            description: Some("Summary style: 'brief', 'detailed', or 'bullet'".to_string()),
                            required: false,
                            arg_type: "string".to_string(),
                        },
                    ],
                    template: "Please create a {{style}} summary of the following content:\n\n{{content}}".to_string(),
                },
            ],
        }
    }
}

pub struct PromptManager {
    config_path: PathBuf,
}

impl PromptManager {
    pub fn new() -> Self {
        let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("./"));
        let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("./"));
        let config_path = exe_dir.join("prompt.json");
        
        Self { config_path }
    }

    pub fn load_config(&self) -> anyhow::Result<PromptConfig> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            // 创建默认配置
            let config = PromptConfig::default();
            self.save_config(&config)?;
            Ok(config)
        }
    }

    pub fn save_config(&self, config: &PromptConfig) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn list_prompts(&self) -> anyhow::Result<Vec<PromptEntry>> {
        Ok(self.load_config()?.prompts)
    }

    pub fn get_prompt(&self, name: &str) -> anyhow::Result<Option<PromptEntry>> {
        let config = self.load_config()?;
        Ok(config.prompts.into_iter().find(|p| p.name == name))
    }

    pub fn add_prompt(&self, prompt: PromptEntry) -> anyhow::Result<()> {
        let mut config = self.load_config()?;
        
        // 检查名称是否已存在
        if config.prompts.iter().any(|p| p.name == prompt.name) {
            return Err(anyhow::anyhow!("Prompt with name '{}' already exists", prompt.name));
        }

        config.prompts.push(prompt);
        self.save_config(&config)?;
        Ok(())
    }

    pub fn remove_prompt(&self, name: &str) -> anyhow::Result<bool> {
        let mut config = self.load_config()?;
        let original_len = config.prompts.len();
        config.prompts.retain(|p| p.name != name);
        
        if config.prompts.len() < original_len {
            self.save_config(&config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn render_prompt(&self, name: &str, args: &HashMap<String, String>) -> anyhow::Result<String> {
        let prompt = self.get_prompt(name)?
            .ok_or_else(|| anyhow::anyhow!("Prompt '{}' not found", name))?;

        let mut template = prompt.template.clone();
        
        // 替换必需参数
        for arg in &prompt.arguments {
            if arg.required && !args.contains_key(&arg.name) {
                return Err(anyhow::anyhow!("Missing required argument: {}", arg.name));
            }
        }

        // 执行模板替换
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            template = template.replace(&placeholder, value);
        }

        // 处理可选参数的默认值
        for arg in &prompt.arguments {
            if !args.contains_key(&arg.name) {
                let placeholder = format!("{{{{{}}}}}", arg.name);
                let default_value = match arg.name.as_str() {
                    "language" => "",
                    "style" => "brief",
                    _ => "",
                };
                template = template.replace(&placeholder, default_value);
            }
        }

        Ok(template)
    }

    pub fn format_prompt_list(&self, prompts: &[PromptEntry]) -> String {
        if prompts.is_empty() {
            return format!("{}\n", "No prompts configured".yellow());
        }

        let mut output = format!("{}\n", "🎯 Available Prompts:".cyan().bold());
        output.push_str(&format!("{} {}\n", "Config:".dimmed(), self.config_path.display()));
        output.push_str(&format!("{}\n", "━".repeat(60).dimmed()));

        for prompt in prompts {
            let title = prompt.title.as_deref().unwrap_or(&prompt.name);
            output.push_str(&format!(
                "{} {} {}\n",
                "•".blue(),
                prompt.name.green().bold(),
                format!("- {}", title).dimmed()
            ));

            if let Some(description) = &prompt.description {
                output.push_str(&format!("  {}\n", description.dimmed()));
            }

            if !prompt.arguments.is_empty() {
                output.push_str(&format!("  {}\n", "Arguments:".bold()));
                for arg in &prompt.arguments {
                    let required_str = if arg.required { "required".red() } else { "optional".green() };
                    output.push_str(&format!(
                        "    {} {} {} - {}\n",
                        "○".dimmed(),
                        arg.name.yellow(),
                        format!("[{}]", required_str),
                        arg.description.as_deref().unwrap_or("No description")
                    ));
                }
            }
            output.push('\n');
        }

        output
    }

    pub fn format_prompt_detail(&self, prompt: &PromptEntry) -> String {
        let mut output = format!("{}\n", format!("🎯 Prompt: {}", prompt.name).cyan().bold());
        output.push_str(&format!("{}\n", "━".repeat(60).dimmed()));

        if let Some(title) = &prompt.title {
            output.push_str(&format!("{} {}\n", "Title:".bold(), title));
        }

        if let Some(description) = &prompt.description {
            output.push_str(&format!("{} {}\n", "Description:".bold(), description));
        }

        if !prompt.arguments.is_empty() {
            output.push_str(&format!("{}\n", "Arguments:".bold()));
            for arg in &prompt.arguments {
                let required_str = if arg.required { "required".red() } else { "optional".green() };
                output.push_str(&format!(
                    "  {} {} {} ({})\n",
                    "•".blue(),
                    arg.name.yellow(),
                    format!("[{}]", required_str),
                    arg.arg_type.cyan()
                ));
                if let Some(desc) = &arg.description {
                    output.push_str(&format!("    {}\n", desc.dimmed()));
                }
            }
        }

        output.push_str(&format!("{}\n", "Template:".bold()));
        output.push_str(&format!("{}\n", "─".repeat(40).dimmed()));
        output.push_str(&format!("{}\n", prompt.template));

        output
    }
}
