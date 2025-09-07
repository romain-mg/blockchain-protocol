use anyhow::{Result, bail};
use base64::prelude::*;
use blockchain_core::{self, log};
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
    StreamExt,
};
use libp2p::{
    PeerId,
    bytes::BufMut,
    identity::{self, Keypair},
    kad::{store::MemoryStore, self}, noise,
    swarm::{NetworkBehaviour, StreamProtocol, SwarmEvent, Swarm},
    tcp, yamux,
    request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel},
    core::Multiaddr,
    multiaddr::Protocol
};
use std::{
    env,
    num::NonZeroUsize,
    ops::Add,
    time::{Duration, Instant},
    collections::{hash_map, HashMap},
    error::Error,
};
use tracing_subscriber::EnvFilter;
use serde::{Serialize, Deserialize};


const BOOTNODE_ID: &str = "12D3KooWCxCPBaitzgsjvogRgcLEJ3i1CfHk5HhxUF3ykTg7fs2U";
const BOOTNODE_MULTIADDR: &str =
    "/ip4/192.168.12.37/tcp/4001/p2p/12D3KooWCxCPBaitzgsjvogRgcLEJ3i1CfHk5HhxUF3ykTg7fs2U";
const PROTO_NAME: StreamProtocol = StreamProtocol::new("/blockchain/1.0.0");

/// Creates the network components, namely:
///
/// - The network client to interact with the network layer from anywhere within your application.
///
/// - The network event stream, e.g. for incoming requests.
///
/// - The network task driving the network itself.
pub(crate) async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop), Box<dyn Error>> {
    // Create a public/private key pair, either random or based on a seed.
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = id_keys.public().to_peer_id();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| Behaviour {
            kademlia: kad::Behaviour::new(
                peer_id,
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            ),
            request_response: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/blockchain/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    let (command_sender, command_receiver) = mpsc::channel(0);
    let (event_sender, event_receiver) = mpsc::channel(0);

    Ok((
        Client {
            sender: command_sender,
        },
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}

#[derive(Clone)]
pub(crate) struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    /// Listen for incoming connections on the given address.
    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Dial the given peer at the given address.
    pub(crate) async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub(crate) async fn request_blockchain_sync(
        &mut self,
        peer: PeerId,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestBlockchainSync {
                peer,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    pub(crate) async fn respond_blockchain_sync(
        &mut self,
        serialized_blockchain: Vec<u8>,
        channel: ResponseChannel<BlockchainSyncResponse>,
    ) {
        self.sender
            .send(Command::RespondBlockchainSync { serialized_blockchain, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }
}


pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_request_blockchain_sync:
        HashMap<OutboundRequestId, oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>>,
}

impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_request_blockchain_sync: Default::default(),
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None=>  return,
                },
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(_)) => {}
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
               channel, ..
                } => {
                    self.event_sender
                        .send(Event::InboundRequest {
                            request: String::from("blockchain sync request"),
                            channel,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_request_blockchain_sync
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(Ok(response.0));
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                let _ = self
                    .pending_request_blockchain_sync
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(Box::new(error)));
            }
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {}
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                log::info!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => eprintln!("Dialing {peer_id}"),
            e => panic!("{e:?}"),
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, peer_addr.clone());
                    match self.swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(Box::new(e)));
                        }
                    }
                } else {
                    todo!("Already dialing peer.");
                }
            }
            Command::RequestBlockchainSync {
                peer,
                sender,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, BlockchainSyncRequest());
                self.pending_request_blockchain_sync.insert(request_id, sender);
            }
            Command::RespondBlockchainSync { serialized_blockchain, channel } => {
                self.swarm
                    .behaviour_mut()
                    .request_response // SAME
                    .send_response(channel, BlockchainSyncResponse(serialized_blockchain))
                    .expect("Connection to peer to be still open.");
            }
        }
    }
}


#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    RequestBlockchainSync {
        peer: PeerId,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
    RespondBlockchainSync {
        serialized_blockchain: Vec<u8>,
        channel: ResponseChannel<BlockchainSyncResponse>,
    },
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    request_response: request_response::cbor::Behaviour<BlockchainSyncRequest, BlockchainSyncResponse>,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

#[derive(Debug)]
pub(crate) enum Event {
    InboundRequest {
        request: String,
        channel: ResponseChannel<BlockchainSyncResponse>,
    },
}

// Simple blockchain sync protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BlockchainSyncRequest();
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BlockchainSyncResponse(Vec<u8>);


















pub async fn start_node() -> Result<()> {
    log::info!("Starting p2p node");
    let key_pair = identity::Keypair::generate_ed25519();
    launch_swarm(key_pair, false).await
}

pub async fn start_bootstrap_node() -> Result<()> {
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
    launch_swarm(decoded_keys, true).await
}

async fn launch_swarm(
    key_pair: Keypair,
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
            let mut cfg = kad::Config::new(PROTO_NAME);
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

    loop {
        let event = swarm.select_next_some().await;
        match event {
            SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed {
                result: kad::QueryResult::GetClosestPeers(Ok(ok)),
                ..
            }) => {
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
