use colored::Colorize;
use crate::server::state::ServerState;

pub struct Repl {
    state: ServerState,
}

impl Repl {
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    pub fn banner(&self) -> String {
        format!(
            "{}\n{}\n  {} {} - Set say content\n  {} {} - Manage resources\n  {} {} - Manage prompts\n  {} - Show this help\n  {} - Disconnect\n\n",
            "mcpcs-server SSH REPL".cyan().bold(),
            "Commands:".yellow(),
            "/say".green(),
            "<content>".dimmed(),
            "/resource".green(),
            "<subcommand>".dimmed(),
            "/prompt".green(),
            "<subcommand>".dimmed(),
            "/help".green(),
            "/exit".green(),
        )
    }

    pub fn prompt(&self) -> String {
        format!("{} ", ">".cyan().bold())
    }

    pub async fn handle_input(&self, input: &str) -> (String, bool) {
        let input = input.trim();
        if input.is_empty() {
            return (String::new(), false);
        }

        let parts: Vec<&str> = input.splitn(6, ' ').collect();
        match parts[0] {
            "/say" => {
                if parts.len() > 1 {
                    let content = parts[1..].join(" ");
                    self.state.set_say(content.clone()).await;
                    (format!("{} {}\n", "Say set to:".green(), content), false)
                } else {
                    (format!("{}\n", "Usage: /say <content>".yellow()), false)
                }
            }
            "/resource" => {
                if parts.len() > 1 {
                    self.handle_resource_command(&parts[1..]).await
                } else {
                    (format!("{}\n{}\n  {} - List resources\n  {} - Add text resource\n  {} - Add file resource\n  {} - Remove resource\n  {} - Reload config\n",
                        "Usage: /resource <subcommand>".yellow(),
                        "Subcommands:".dimmed(),
                        "list".green(),
                        "add text <uri> <content>".green(),
                        "add file <uri> <path>".green(),
                        "rm <uri>".green(),
                        "reload".green(),
                    ), false)
                }
            }
            "/prompt" => {
                if parts.len() > 1 {
                    self.handle_prompt_command(&parts[1..]).await
                } else {
                    (format!("{}\n{}\n  {} - List prompts\n  {} - Show prompt details\n  {} - Test prompt with args\n  {} - Remove prompt\n  {} - Reload config\n",
                        "Usage: /prompt <subcommand>".yellow(),
                        "Subcommands:".dimmed(),
                        "list".green(),
                        "show <name>".green(),
                        "test <name> [args...]".green(),
                        "rm <name>".green(),
                        "reload".green(),
                    ), false)
                }
            }
            "/help" => {
                (self.banner(), false)
            }
            "/exit" | "/quit" => {
                (format!("{}\n", "Goodbye!".cyan()), true)
            }
            _ => {
                (format!("{} {}\n", "Unknown command:".yellow(), parts[0]), false)
            }
        }
    }

    async fn handle_resource_command(&self, args: &[&str]) -> (String, bool) {
        if args.is_empty() {
            return (format!("{}\n", "Usage: /resource <subcommand>".yellow()), false);
        }

        match args[0] {
            "list" => {
                match self.state.resources.list_resources() {
                    Ok(resources) => {
                        let output = self.state.resources.format_resource_list(&resources);
                        (output, false)
                    }
                    Err(e) => (format!("{} {}\n", "Error:".red(), e), false)
                }
            }
            "add" => {
                if args.len() >= 4 && args[1] == "text" {
                    let uri = args[2];
                    let content = args[3..].join(" ");
                    let name = uri.split("://").last().unwrap_or(uri);

                    match self.state.resources.add_text_resource(
                        uri.to_string(),
                        name.to_string(),
                        Some("Added via REPL".to_string()),
                        content,
                    ) {
                        Ok(_) => (format!("{} {}\n", "Text resource added:".green(), uri), false),
                        Err(e) => (format!("{} {}\n", "Error:".red(), e), false),
                    }
                } else if args.len() >= 4 && args[1] == "file" {
                    let uri = args[2];
                    let path = args[3];
                    let name = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);

                    match self.state.resources.add_file_resource(
                        uri.to_string(),
                        name.to_string(),
                        Some(format!("File: {}", path)),
                        path.to_string(),
                    ) {
                        Ok(_) => (format!("{} {} -> {}\n", "File resource added:".green(), uri, path), false),
                        Err(e) => (format!("{} {}\n", "Error:".red(), e), false),
                    }
                } else {
                    (format!("{}\n", "Usage: /resource add text <uri> <content> | /resource add file <uri> <path>".yellow()), false)
                }
            }
            "rm" => {
                if args.len() >= 2 {
                    let uri = args[1];
                    match self.state.resources.remove_resource(uri) {
                        Ok(true) => (format!("{} {}\n", "Resource removed:".green(), uri), false),
                        Ok(false) => (format!("{} {}\n", "Resource not found:".yellow(), uri), false),
                        Err(e) => (format!("{} {}\n", "Error:".red(), e), false),
                    }
                } else {
                    (format!("{}\n", "Usage: /resource rm <uri>".yellow()), false)
                }
            }
            "reload" => {
                match self.state.resources.load_config() {
                    Ok(_) => (format!("{}\n", "Resource config reloaded".green()), false),
                    Err(e) => (format!("{} {}\n", "Error reloading config:".red(), e), false),
                }
            }
            _ => {
                (format!("{} {}\n", "Unknown resource subcommand:".yellow(), args[0]), false)
            }
        }
    }

    async fn handle_prompt_command(&self, args: &[&str]) -> (String, bool) {
        if args.is_empty() {
            return (format!("{}\n", "Usage: /prompt <subcommand>".yellow()), false);
        }

        match args[0] {
            "list" => {
                match self.state.prompts.list_prompts() {
                    Ok(prompts) => {
                        let output = self.state.prompts.format_prompt_list(&prompts);
                        (output, false)
                    }
                    Err(e) => (format!("{} {}\n", "Error:".red(), e), false)
                }
            }
            "show" => {
                if args.len() >= 2 {
                    let name = args[1];
                    match self.state.prompts.get_prompt(name) {
                        Ok(Some(prompt)) => {
                            let output = self.state.prompts.format_prompt_detail(&prompt);
                            (output, false)
                        }
                        Ok(None) => (format!("{} {}\n", "Prompt not found:".yellow(), name), false),
                        Err(e) => (format!("{} {}\n", "Error:".red(), e), false),
                    }
                } else {
                    (format!("{}\n", "Usage: /prompt show <name>".yellow()), false)
                }
            }
            "test" => {
                if args.len() >= 2 {
                    let name = args[1];
                    
                    // 解析参数 key=value 格式
                    let mut test_args = std::collections::HashMap::new();
                    for arg in &args[2..] {
                        if let Some((key, value)) = arg.split_once('=') {
                            test_args.insert(key.to_string(), value.to_string());
                        }
                    }

                    match self.state.prompts.render_prompt(name, &test_args) {
                        Ok(rendered) => {
                            let mut output = format!("{} {}\n", "Generated prompt:".green(), name);
                            output.push_str(&format!("{}\n", "━".repeat(60).dimmed()));
                            output.push_str(&rendered);
                            output.push('\n');
                            (output, false)
                        }
                        Err(e) => (format!("{} {}\n", "Error:".red(), e), false),
                    }
                } else {
                    (format!("{}\n", "Usage: /prompt test <name> [key=value...]".yellow()), false)
                }
            }
            "rm" => {
                if args.len() >= 2 {
                    let name = args[1];
                    match self.state.prompts.remove_prompt(name) {
                        Ok(true) => (format!("{} {}\n", "Prompt removed:".green(), name), false),
                        Ok(false) => (format!("{} {}\n", "Prompt not found:".yellow(), name), false),
                        Err(e) => (format!("{} {}\n", "Error:".red(), e), false),
                    }
                } else {
                    (format!("{}\n", "Usage: /prompt rm <name>".yellow()), false)
                }
            }
            "reload" => {
                match self.state.prompts.load_config() {
                    Ok(_) => (format!("{}\n", "Prompt config reloaded".green()), false),
                    Err(e) => (format!("{} {}\n", "Error reloading config:".red(), e), false),
                }
            }
            _ => {
                (format!("{} {}\n", "Unknown prompt subcommand:".yellow(), args[0]), false)
            }
        }
    }
}
