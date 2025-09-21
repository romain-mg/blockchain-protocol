mod p2p_node;
use anyhow::Result;
use blockchain_core::{blockchain::{Blockchain}, miner::Miner, log};
use clap::Parser;
use futures::StreamExt;
use libp2p::{core::Multiaddr, multiaddr::Protocol};
use primitive_types::U256;
use tokio::task::{spawn};
use tracing_subscriber::EnvFilter;
use std::{error::Error, sync::{Mutex, Arc, atomic::{AtomicBool, Ordering}}, thread};
use serde_json;

const TARGET_DURATION_BETWEEN_BLOCKS: u64 = 1;
const MAX_TRANSACTIONS_PER_BLOCK: usize = 3;
const BLOCKS_BETWEEN_DIFFICULTY_ADJUSTMENT: u64 = 10;
const BOOTNODE_ID: &str = "bootnode_id";
const BOOTNODE_MULTIADDR: &str =
    "/ip_address/bootnode_id";


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::init_logger();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let opt = Opt::parse();

    let (mut node_client, mut node_events, node_event_loop) = p2p_node::new(opt.bootnode, opt.secret_key_seed).await?;

    // Spawn the network task for it to run in the background.
    spawn(node_event_loop.run());

    // In case we want to spin up a bootnode, use the bootnode address. Otherwise, if an address
    // was provided, use it. Else, listen on any address.
    if let Some(true) = opt.bootnode {
            node_client
                .start_listening(BOOTNODE_MULTIADDR.parse()?)
                .await
                .expect("Listening not to fail.");
    } else {
        match opt.listen_address {
            Some(addr) => node_client
                .start_listening(addr)
                .await
                .expect("Listening not to fail."),
            None => node_client
                .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
                .await
                .expect("Listening not to fail."),
        };
    }
    
    // In case the user wants to sync, dial with the bootnode
    if let Some(true) = opt.sync {
        node_client
            .dial(BOOTNODE_ID.parse()?, BOOTNODE_MULTIADDR.parse()?)
            .await
            .expect("Dial to succeed");
        log::info!("Dialed with bootnode at {:?}", BOOTNODE_MULTIADDR);
    }

    // In case the user provided an address of a peer on the CLI, dial it.
    if let Some(addr) = opt.peer {
        let Some(Protocol::P2p(peer_id)) = addr.iter().last() else {
            return Err("Expect peer multiaddr to contain peer ID.".into());
        };
        node_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    if let Some(true) = opt.bootnode {
        let difficulty_divisor: i32 = 20000;
        let difficulty: U256 = U256::MAX / difficulty_divisor;
        
        // run 2 tasks with a lock, 1 starts producing blocks, 1 listens to request, serializes blockchain and sends it 
        let blockchain: Arc<Mutex<Blockchain>> = Arc::new(Mutex::new(Blockchain::create_blockchain(difficulty, TARGET_DURATION_BETWEEN_BLOCKS, MAX_TRANSACTIONS_PER_BLOCK, BLOCKS_BETWEEN_DIFFICULTY_ADJUSTMENT)));
        

        // Missing: introduce an atomic bool to pause miner thread?

        let mut miner: Miner = Miner::new();
        let miner_chain_reference = Arc::clone(&blockchain);

        let can_miner_run = Arc::new(AtomicBool::new(true));
        let can_miner_run_clone = Arc::clone(&can_miner_run);
        thread::spawn(move || {
            let mut hash = String::from("");
            loop {
                log::info!("Checking if miner can mine...");
                if can_miner_run_clone.load(Ordering::Relaxed) {
                let mut locked_miner_chain = miner_chain_reference.lock().expect("Write lock to be acquired");
                log::info!("Lock acquired by miner");
                hash = miner.compute_next_block(&mut *locked_miner_chain, hash).expect("Next block to be computed");
                log::info!("Next block computed, going to next loop iter");
                }
                else {
                    log::info!("Cannot mine anymore, not acquiring lock and yielding");
                    thread::yield_now(); 
                }
            }   
    });
        loop {
            match node_events.next().await {
                Some(p2p_node::Event::InboundRequest { channel }) => {
                    log::info!("Indicating miner to not mine anymore");
                    can_miner_run.store(false, Ordering::Relaxed);
                    log::info!("Received event for inbound request in main function");
                    let sync_chain_reference = Arc::clone(&blockchain);
                    let locked_sync_chain = sync_chain_reference.lock().expect("Read lock to be acquired");
                    log::info!("Locked sync chain in main function");
                    let serialized_blockchain = serde_json::to_vec(&(*locked_sync_chain)).expect("Blockchain to be serialized");
                    log::info!("Serialized blockchain in main function");
                        node_client
                            .respond_blockchain_sync(serialized_blockchain, channel)
                            .await;
                    log::info!("Responded to blockchain sync request in main function");
                    can_miner_run.store(true, Ordering::Relaxed);
                }
                e => todo!("{:?}", e),
            }
        }
    } else if let Some(true) = opt.sync {
        log::info!("Requesting blockchain from bootnode");
        // request blockchain to boot node
        let serialized_chain = node_client.request_blockchain_sync(BOOTNODE_ID.parse()?).await;
        match serialized_chain {
            Ok(chain) => {
                let blockchain: Blockchain = serde_json::from_slice(&chain).expect("Blockchain to be deserialized");
                log::info!("Retrived blockchain: {:?}", blockchain)
            },
            Err(e) => {
                log::error!("{:?}", e)
            }
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "Blockchain simulation")]
struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[arg(long)]
    secret_key_seed: Option<u8>,

    #[arg(long)]
    peer: Option<Multiaddr>,

    #[arg(long)]
    listen_address: Option<Multiaddr>,

    #[arg(long)]
    bootnode: Option<bool>,

    #[arg(long)]
    sync: Option<bool>,
}