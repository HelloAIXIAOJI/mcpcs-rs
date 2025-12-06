use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ServerState {
    pub ssh_port: u16,
    pub say_content: Arc<RwLock<String>>,
}

impl ServerState {
    pub fn new(ssh_port: u16) -> Self {
        Self {
            ssh_port,
            say_content: Arc::new(RwLock::new(String::new())),
        }
    }

    pub async fn set_say(&self, content: String) {
        let mut say = self.say_content.write().await;
        *say = content;
    }

    pub async fn get_say(&self) -> String {
        let say = self.say_content.read().await;
        say.clone()
    }
}
