mod client_to_node_server;
mod p2p_node;
use blockchain_core::log;
use clap::Parser;
use dotenv::dotenv;
use libp2p::PeerId;
use tokio::task;

#[derive(Parser, Debug)]
#[command(name = "libp2p Kademlia DHT")]
struct Args {
    #[arg(long)]
    secondary: bool,
    #[arg(long)]
    bootstrap: bool,
    #[command(subcommand)]
    kademilia_op: Option<KademiliaOp>,
}

#[derive(Debug, Parser, Clone)]

enum KademiliaOp {
    GetPeers {
        #[arg(long)]
        peer_id: Option<PeerId>,
    },
    PutPkRecord {},
}

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    log::init_logger();
    let args = Args::parse();
    let secondary = args.secondary;
    let bootstrap = args.bootstrap;
    let kademilia_op = args.kademilia_op;
    let p2p_handle;
    if bootstrap {
        p2p_handle = task::spawn(p2p_node::start_bootstrap_node(kademilia_op));
    } else {
        p2p_handle = task::spawn(p2p_node::start_node(kademilia_op));
    }
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
