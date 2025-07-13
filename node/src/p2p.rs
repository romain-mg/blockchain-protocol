use std::{error::Error, time::Duration};

use blockchain_core::log;
use futures::prelude::*;
use libp2p::{Multiaddr, noise, ping, swarm::SwarmEvent, tcp, yamux};
use tracing_subscriber::EnvFilter;

pub async fn start() -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Starting p2p server");
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))) // Allows us to observe pings indefinit
        .build();
    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    // if let Some(addr) = std::env::args().nth(1) {
    //     let remote: Multiaddr = addr.parse()?;
    //     swarm.dial(remote)?;
    //     log::info!("Dialed {addr}")
    // }
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => log::info!("Listening on {address:?}"),
            SwarmEvent::Behaviour(event) => log::info!("{event:?}"),
            _ => {}
        }
    }
}
