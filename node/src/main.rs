mod client_to_node_server;
mod p2p;
use blockchain_core::log;
use clap::Parser;
use tokio::task;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    secondary: bool,
}

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    log::init_logger();
    let args = Args::parse();
    let secondary = args.secondary;
    let p2p_handle = task::spawn(p2p::start());
    let client_to_node_handle;
    if secondary {
        client_to_node_handle = task::spawn(client_to_node_server::start_secondary());
    } else {
        client_to_node_handle = task::spawn(client_to_node_server::start());
    }
    let (p2p_result, client_to_node_result) = tokio::join!(p2p_handle, client_to_node_handle);
    if let Err(e) = p2p_result {
        log::error!("P2P server error: {:?}", e);
    }
    if let Err(e) = client_to_node_result {
        log::error!("Client to node server error: {:?}", e);
    }

    Ok(())
}
