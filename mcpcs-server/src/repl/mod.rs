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
            "{}\n{}\n  {} {} - Set say content\n  {} - Show this help\n  {} - Disconnect\n\n",
            "mcpcs-server SSH REPL".cyan().bold(),
            "Commands:".yellow(),
            "/say".green(),
            "<content>".dimmed(),
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

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        match parts[0] {
            "/say" => {
                if parts.len() > 1 {
                    let content = parts[1].to_string();
                    self.state.set_say(content.clone()).await;
                    (format!("{} {}\n", "Say set to:".green(), content), false)
                } else {
                    (format!("{}\n", "Usage: /say <content>".yellow()), false)
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
}
