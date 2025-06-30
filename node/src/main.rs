mod client_to_node_server;
mod p2pclient;
mod p2pserver;

use anyhow::Result;
use blockchain_core::log::init_logger;
use tokio::main;

#[main]
async fn main() -> Result<()> {
    init_logger();
    p2pserver::start().await?;
    client_to_node_server::start().await?;
    Ok(())
}
