use crate::KademiliaOp;
use anyhow::{Result, bail};
use base64::prelude::*;
use blockchain_core::log;
use futures::prelude::*;
use libp2p::{
    PeerId,
    bytes::BufMut,
    identity::{self, Keypair},
    kad::{self, NoKnownPeers},
    noise,
    swarm::{StreamProtocol, SwarmEvent},
    tcp, yamux,
};
use std::{
    env,
    num::NonZeroUsize,
    ops::Add,
    time::{Duration, Instant},
};
use tracing_subscriber::EnvFilter;

const BOOTNODE_ID: &str = "12D3KooWCxCPBaitzgsjvogRgcLEJ3i1CfHk5HhxUF3ykTg7fs2U";
const BOOTNODE_MULTIADDR: &str =
    "/ip4/192.168.12.37/tcp/4001/p2p/12D3KooWCxCPBaitzgsjvogRgcLEJ3i1CfHk5HhxUF3ykTg7fs2U";

const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/blockchain/kad/1.0.0");

pub async fn start_node(kademilia_op: Option<KademiliaOp>) -> Result<()> {
    log::info!("Starting p2p node");
    let key_pair = identity::Keypair::generate_ed25519();
    launch_swarm(key_pair, kademilia_op, false).await
}

pub async fn start_bootstrap_node(kademilia_op: Option<KademiliaOp>) -> Result<()> {
    log::info!("Starting bootstrap p2p node");
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let encoded_bootstrap_node_keys = env::var("BOOTSTRAP_NODE_KEYS").expect("keys not found");
    let decoded: Vec<u8> = BASE64_STANDARD
        .decode(&encoded_bootstrap_node_keys)
        .expect("invalid base64");
    let decoded_keys = identity::Keypair::from_protobuf_encoding(&decoded).unwrap();
    let bootnode_id = decoded_keys.public().to_peer_id().to_base58();
    if &bootnode_id != BOOTNODE_ID {
        panic!("Wrong bootnode ID");
    }
    launch_swarm(decoded_keys, kademilia_op, true).await
}

async fn launch_swarm(
    key_pair: Keypair,
    kademilia_op: Option<KademiliaOp>,
    bootstrap: bool,
) -> Result<()> {
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key_pair.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key_pair| {
            // Create a Kademlia behaviour.
            let mut cfg = kad::Config::new(IPFS_PROTO_NAME);
            cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let store = kad::store::MemoryStore::new(key_pair.public().to_peer_id());
            kad::Behaviour::with_config(key_pair.public().to_peer_id(), store, cfg)
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))) // Allows us to observe pings indefinit
        .build();
    if bootstrap {
        swarm.behaviour_mut().set_mode(Some(kad::Mode::Server));
        swarm.listen_on(BOOTNODE_MULTIADDR.parse()?)?;
    } else {
        // problem it works even with garbage address so no real guarantee that it works
        if let kad::RoutingUpdate::Success = swarm
            .behaviour_mut()
            .add_address(&BOOTNODE_ID.parse()?, BOOTNODE_MULTIADDR.parse()?)
        {
            log::info!("Bootnode added to the routing table");
        } else {
            bail!("Failed to add bootnode to the routing table");
        }
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        if let Err(no_known_peers) = swarm.behaviour_mut().bootstrap() {
            log::error!("Failed to bootstrap: no known peers");
            return Err(no_known_peers.into());
        } else {
            log::info!("Bootstrapped successfully");
        }
    }

    swarm.behaviour_mut().set_mode(Some(kad::Mode::Server));

    match kademilia_op {
        Some(KademiliaOp::GetPeers { peer_id }) => {
            let peer_id = peer_id.unwrap_or(PeerId::random());
            log::info!("Searching for the closest peers to {peer_id}");
            swarm.behaviour_mut().get_closest_peers(peer_id);
        }
        Some(KademiliaOp::PutPkRecord {}) => {
            log::info!("Putting PK record into the DHT");
            let mut pk_record_key = vec![];
            pk_record_key.put_slice("/pk/".as_bytes());
            pk_record_key.put_slice(swarm.local_peer_id().to_bytes().as_slice());

            let mut pk_record =
                kad::Record::new(pk_record_key, key_pair.public().encode_protobuf());
            pk_record.publisher = Some(*swarm.local_peer_id());
            pk_record.expires = Some(Instant::now().add(Duration::from_secs(60)));

            swarm
                .behaviour_mut()
                .put_record(pk_record, kad::Quorum::N(NonZeroUsize::new(3).unwrap()))?;
        }
        None => {}
    }
    loop {
        let event = swarm.select_next_some().await;
        match event {
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::GetClosestPeers(Ok(ok)),
                ..
            }) => {
                // The example is considered failed as there
                // should always be at least 1 reachable peer.
                if ok.peers.is_empty() {
                    bail!("Query finished with no closest peers.")
                }

                log::info!("Query finished with closest peers: {:#?}", ok.peers);

                return Ok(());
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                result:
                    kad::QueryResult::GetClosestPeers(Err(kad::GetClosestPeersError::Timeout {
                        ..
                    })),
                ..
            }) => {
                bail!("Query for closest peers timed out")
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::PutRecord(Ok(_)),
                ..
            }) => {
                log::info!("Successfully inserted the PK record");

                return Ok(());
            }
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::PutRecord(Err(err)),
                ..
            }) => {
                bail!(anyhow::Error::new(err).context("Failed to insert the PK record"));
            }
            _ => {}
        }
    }
}
