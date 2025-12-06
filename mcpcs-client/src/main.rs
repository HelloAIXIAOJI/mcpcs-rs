mod config;
mod client;
mod repl;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    repl::run().await
}
