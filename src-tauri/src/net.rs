use crate::node::boxpeer_dir;
use crate::node::load_or_generate_keypair;
use anyhow::{anyhow, Result};
use beetswap;
use blockstore::block::CidError;
use blockstore::{block::Block, Blockstore, SledBlockstore};
use cid::Cid;
use futures::channel::{mpsc, oneshot};
use futures::{SinkExt, Stream, StreamExt};
use libp2p::kad::store::MemoryStore;
use libp2p::multiaddr::Protocol;
use libp2p::{
    identify, identity, kad, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, Swarm, SwarmBuilder,
};
use libp2p::{PeerId, StreamProtocol};
use libp2p_kad::RecordKey;
use multihash_codetable::{Code, MultihashDigest};
use sled;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{fs};
use tokio::select;
use tracing::{info, warn};

const BOXPEER_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/0.1.0");

struct FileBlock(Vec<u8>);

impl Block<64> for FileBlock {
    fn cid(&self) -> Result<cid::CidGeneric<64>, CidError> {
        let hash = Code::Sha2_256.digest(self.0.as_ref());
        Ok(Cid::new_v1(0x55, hash))
    }

    fn data(&self) -> &[u8] {
        &self.0.as_ref()
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    bitswap: beetswap::Behaviour<64, SledBlockstore>,
    mdns: mdns::tokio::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
}

pub struct P2PCDNClient {
    blockstore: Arc<SledBlockstore>,
    command_sender: mpsc::Sender<Command>,
}

impl P2PCDNClient {
    pub async fn new(
        bootstrap_peers: Option<Vec<Multiaddr>>,
        secret_key_seed: Option<u8>,
    ) -> std::result::Result<
        (P2PCDNClient, impl Stream<Item = kad::Event>, EventLoop),
        Box<dyn Error>,
    > {
        let id_keys = match secret_key_seed {
            Some(seed) => {
                let mut bytes = [0u8; 32];
                bytes[0] = seed;
                identity::Keypair::ed25519_from_bytes(bytes)?
            }
            None => load_or_generate_keypair(),
        };

        let peer_id = id_keys.public().to_peer_id();
        let path = boxpeer_dir().await.expect("Error with cache");
        let db: sled::Db;

        loop {
            match sled::open(&path) {
                Ok(opened_db) => {
                    db = opened_db;
                    break;
                }
                Err(_) => {
                    let fallback_path = format!("{}_fallback", path);
                    info!(
                        "DB is still locked, falling back to DB2 at {}",
                        fallback_path
                    );
                    db = sled::open(fallback_path)?;
                    break;
                }
            }
        }

        let identify = identify::Behaviour::new(identify::Config::new(
            BOXPEER_PROTO_NAME.to_string(),
            id_keys.public().clone(),
        ));

        let blockstore = Arc::new(SledBlockstore::new(db).await.expect("Err"));
        let mut cfg = kad::Config::new(BOXPEER_PROTO_NAME);

        cfg.set_periodic_bootstrap_interval(Some(Duration::from_secs(10)));
        cfg.set_record_ttl(None);

        let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_quic()
            .with_behaviour(|key| Behaviour {
                kademlia: kad::Behaviour::with_config(
                    peer_id,
                    MemoryStore::new(key.public().to_peer_id()),
                    cfg,
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )
                .expect("Error with mdns configuring"),
                bitswap: beetswap::Behaviour::new(blockstore.clone()),
                identify,
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))
            })
            .build();

        swarm
            .behaviour_mut()
            .kademlia
            .set_mode(Some(kad::Mode::Server));

        let address: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1"
            .parse()
            .expect("Error with address");
        swarm.listen_on(address).expect("Error listening");

        // Dial bootstrap peers if provided
        if let Some(peers) = bootstrap_peers {
            for peer in peers {
                if let Err(e) = swarm.dial(peer.clone()) {
                    eprintln!("Failed to dial peer {}: {}", peer, e);
                } else {
                    println!("Dialing bootstrap peer: {}", peer);
                }
            }
        }

        let (command_sender, command_receiver) = mpsc::channel(0);
        let (event_sender, event_receiver) = mpsc::channel(0);
        Ok((
            P2PCDNClient {
                blockstore: blockstore.clone(),
                command_sender,
            },
            event_receiver,
            EventLoop::new(swarm, command_receiver, event_sender, blockstore),
        ))
    }

    pub(crate) async fn get_peers_count(
        &mut self,
    ) -> std::result::Result<Vec<PeerId>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetPeers { sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub(crate) async fn start_listening(&mut self, addr: Multiaddr) -> Result<String> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn upload_file(&mut self, file_path: PathBuf) -> Result<String> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::UploadFile { file_path, sender })
            .await?;

        let cid = receiver.await??;
        Ok(cid.to_string())
    }
    pub async fn get_all_files(&mut self, cids: Vec<Cid>) -> Result<Vec<Vec<u8>>> {
        let mut contents = Vec::new();
        for cid in cids {
            let content = self.request_file(cid).await.expect("An error occurred");
            contents.push(content);
        }
        Ok(contents)
    }

    pub async fn owned_file(&mut self, cid: Cid) -> Result<bool> {
        if self
            .blockstore
            .has(&cid)
            .await
            .expect("Error with blockstore")
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn request_file(&mut self, cid: Cid) -> Result<Vec<u8>> {
        if !self
            .blockstore
            .has(&cid)
            .await
            .expect("Error with blockstore")
        {
            info!("CID {:?} not found in local blockstore.", cid);
        }

        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::RequestFile { cid, sender })
            .await?;

        let file_data = receiver.await??;
        Ok(file_data)
    }

    pub async fn lock_file(&mut self, cid: Cid) -> Result<String, anyhow::Error> {
        // Check if the file exists in the local blockstore
        if let Ok(true) = self.blockstore.has(&cid).await {
            // File already exists locally, retrieve it
            if let Ok(_data) = self.blockstore.get(&cid).await {
                return Ok("You are already providing this file".to_string());
            }
        }

        // File not found in local blockstore, request it from peers
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::RequestFile { cid, sender })
            .await
            .map_err(|e| anyhow!("Failed to send request for file: {:?}", e))?;
        let file_data = receiver.await??;

        // Store the retrieved file in the local blockstore
        self.blockstore
            .put_keyed(&cid, &file_data)
            .await
            .map_err(|e| anyhow!("Failed to store block in blockstore: {:?}", e))?;

        Ok(format!("You are now providing file {:?}", &cid))
    }

}
pub enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<String>>,
    },
    UploadFile {
        file_path: PathBuf,
        sender: oneshot::Sender<Result<Cid>>,
    },
    RequestFile {
        cid: Cid,
        sender: oneshot::Sender<Result<Vec<u8>>>,
    },
    GetProviders {
        cid: RecordKey,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    GetPeers {
        sender: oneshot::Sender<std::result::Result<Vec<PeerId>, Box<dyn Error + Send>>>,
    },
}

pub struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<kad::Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    queries: HashMap<beetswap::QueryId, Cid>,
    kad_queries: HashMap<libp2p_kad::QueryId, Cid>,
    pending_requests: HashMap<beetswap::QueryId, oneshot::Sender<Result<Vec<u8>>>>,
    pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    blockstore: Arc<SledBlockstore>,
}
impl EventLoop {
    pub(crate) fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<kad::Event>,
        blockstore: Arc<SledBlockstore>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            queries: Default::default(),
            kad_queries: Default::default(),
            pending_requests: Default::default(),
            pending_get_providers: Default::default(),
            blockstore,
        }
    }

    async fn handle_event(
        &mut self,
        event: SwarmEvent<BehaviourEvent>,
    ) -> Result<(), anyhow::Error> {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Bitswap(bitswap)) => match bitswap {
                beetswap::Event::GetQueryResponse { query_id, data } => {
                    self.queries.get(&query_id);
                    if let Some(sender) = self.pending_requests.remove(&query_id) {
                        sender
                            .send(Ok(data))
                            .map_err(|e| anyhow!("Failed to send file data: {:?}", e))?;
                    }
                }
                beetswap::Event::GetQueryError { query_id, error } => {
                    if let Some(sender) = self.pending_requests.remove(&query_id) {
                        sender
                            .send(Err(anyhow!("Error for CID {:?}: {:?}", query_id, error)))
                            .map_err(|e| anyhow!("Failed to send error: {:?}", e))?;
                    }
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns_event)) => {
                if let mdns::Event::Discovered(peers) = mdns_event {
                    for (peer_id, multiaddr) in peers {
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, multiaddr.clone());
                        info!("Discovered Peer MDNS: {:?}", &multiaddr);
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad_event)) => match kad_event {
                kad::Event::RoutingUpdated {
                    peer, addresses, ..
                } => {
                    if let address = addresses.first() {
                        info!("Discovered peer via Kademlia: {:?} at {:?}", peer, address);
                        match self.swarm.dial(address.clone()) {
                            Ok(()) => {
                                info!("Dialing peer: {:?}\n", peer);
                            }
                            Err(e) => {
                                warn!("Error Dialing peer: {:?}\n", e);
                            }
                        }
                    }
                }
                kad::Event::OutboundQueryProgressed { id, result, .. } => {
                    if let kad::QueryResult::GetProviders(Ok(
                        kad::GetProvidersOk::FoundProviders { providers, .. },
                    )) = result
                    {
                        if let Some(sender) = self.pending_get_providers.remove(&id) {
                            sender.send(providers).expect("Receiver not to be dropped");
                            self.swarm
                                .behaviour_mut()
                                .kademlia
                                .query_mut(&id)
                                .unwrap()
                                .finish();
                        }
                    }
                }
                _ => {
                    info!("Other Kademlia event: {:?}", kad_event);
                }
            },
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                warn!(
                    "Failed to connect to peer: {:?}, error: {:?}",
                    peer_id, error
                );
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
                info!("Connection established with peer: {:?}", peer_id);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                info!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id))
                );
            }

            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!(
                    "Connection closed with peer: {:?}, reason: {:?}",
                    peer_id, cause
                );
            }
            SwarmEvent::IncomingConnection {
                local_addr,
                send_back_addr,
                ..
            } => {
                info!(
                    "Incoming connection attempt from {:?} to {:?}",
                    send_back_addr, local_addr
                );
            }
            SwarmEvent::IncomingConnectionError {
                local_addr,
                send_back_addr,
                error,
                ..
            } => {
                warn!(
                    "Failed incoming connection from {:?} to {:?}, error: {:?}",
                    send_back_addr, local_addr, error
                );
            }
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => {
                info!("Dialing peer: {:?}", peer_id);
            }
            SwarmEvent::ExpiredListenAddr { address, .. } => {
                println!("Listen address expired: {:?}", address);
                self.swarm.listen_on(address).expect("Error listening");
            }
            SwarmEvent::ListenerClosed {
                addresses, reason, ..
            } => {
                warn!(
                    "Listener closed for addresses: {:?}, reason: {:?}",
                    addresses, reason
                );
            }
            SwarmEvent::ListenerError {
                listener_id, error, ..
            } => {
                warn!("Listener error on address {:?}: {:?}", listener_id, error);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on new address: {:?}", address);
            }

            _ => {
                warn!("Did not match any specific event: {:?}", event);
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Result<(), anyhow::Error> {
        match command {
            Command::UploadFile { file_path, sender } => {
                // Read the file as binary data
                let file_data = fs::read(&file_path)
                    .map_err(|e| anyhow!("Failed to read file from {:?}: {:?}", file_path, e))?;

                // Create the file block
                let block = FileBlock(file_data);

                // Generate the CID
                let cid = block
                    .cid()
                    .map_err(|e| anyhow!("Failed to generate CID: {:?}", e))?;

                info!("Uploading file with CID: {}", cid);
                self.blockstore
                    .put_keyed(&cid, block.data())
                    .await
                    .map_err(|e| anyhow!("Failed to store block: {:?}", e))?;

                let cid_key = RecordKey::new(&cid.to_bytes());
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(cid_key)
                    .map_err(|e| anyhow!("Failed to start providing the CID: {:?}", e))?;

                // Send the CID as the result of the upload
                sender
                    .send(Ok(cid))
                    .map_err(|e| anyhow!("Failed to send CID result: {:?}", e))?;
            }
            Command::RequestFile { cid, sender } => {
                let query_id = self.swarm.behaviour_mut().bitswap.get(&cid);
                let kad_query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(RecordKey::new(&cid.to_bytes()));
                self.queries.insert(query_id, cid);
                self.kad_queries.insert(kad_query_id, cid);
                self.pending_requests.insert(query_id, sender);
            }
            Command::StartListening { addr, sender } => {
                let peer_id = *self.swarm.local_peer_id();
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());

                let result = self
                    .swarm
                    .listen_on(addr)
                    .map(|_| peer_id.to_string())
                    .map_err(|e| anyhow!("Failed to listen on address: {:?}", e));
                sender
                    .send(result)
                    .map_err(|e| anyhow!("Failed to send start listening result: {:?}", e))?;
            }

            Command::GetProviders { cid, sender } => {
                let query_id = self.swarm.behaviour_mut().kademlia.get_providers(cid);
                self.pending_get_providers.insert(query_id, sender);
                info!("Searching for providers for CID, query ID: {:?}", query_id);
            }
            Command::GetPeers { sender } => {
                let peers: Vec<PeerId> = self.swarm.connected_peers().cloned().collect();
                sender
                    .send(Ok(peers))
                    .map_err(|e| anyhow!("Failed to send peers: {:?}", e))?;
            }
        }

        Ok(())
    }

    pub async fn run(mut self) {
        loop {
            select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await.expect("Error handling event"),
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await.expect("Error handling command"),
                    None=>  return,
                },
            }
        }
    }
}
