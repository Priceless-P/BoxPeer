use crate::node::load_or_generate_keypair;
use anyhow::{anyhow, Result};
use beetswap;
use blockstore::block::CidError;
use blockstore::{block::Block, Blockstore, SledBlockstore};
use cid::Cid;
use futures::channel::{mpsc, oneshot};
use futures::{SinkExt, Stream, StreamExt};
use libp2p::kad::store::MemoryStore;
use libp2p::{
    identity, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, Swarm, SwarmBuilder,
};
use sled;
use libp2p_core::{muxing::StreamMuxerBox, Transport};
use libp2p_identity::PeerId;
use libp2p_kad::RecordKey;
use libp2p_webrtc as webrtc;
use multihash_codetable::{Code, MultihashDigest};
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, io};
use tokio::select;
use tokio::time::interval;
use crate::node::boxpeer_dir;

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
    bitswap: beetswap::Behaviour<64, SledBlockstore>,
    mdns: mdns::tokio::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
}

pub struct P2PCDNClient {
    blockstore: Arc<SledBlockstore>,
    queries: HashMap<beetswap::QueryId, Cid>,
    kad_queries: HashMap<libp2p_kad::QueryId, Cid>,
    command_sender: mpsc::Sender<Command>,
}

impl P2PCDNClient {
    pub async fn new(
        bootstrap_peers: Option<Vec<Multiaddr>>,
        secret_key_seed: Option<u8>,
    ) -> std::result::Result<(P2PCDNClient, impl Stream<Item = Event>, EventLoop), Box<dyn Error>>
    {
        let id_keys = match secret_key_seed {
            Some(seed) => {
                let mut bytes = [0u8; 32];
                bytes[0] = seed;
                identity::Keypair::ed25519_from_bytes(bytes)?
            }
            None => load_or_generate_keypair(),
        };

        let peer_id = id_keys.public().to_peer_id();
        let path = boxpeer_dir().await.expect("Error");
        let db = sled::open(path)?;
        let blockstore = Arc::new(SledBlockstore::new(db).await.expect("Err"));

        let swarm = SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_other_transport(|id_keys| {
                Ok(webrtc::tokio::Transport::new(
                    id_keys.clone(),
                    webrtc::tokio::Certificate::generate(&mut thread_rng())?,
                )
                .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
            })?
            .with_behaviour(|key| Behaviour {
                kademlia: kad::Behaviour::new(peer_id, MemoryStore::new(key.public().to_peer_id())),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )
                .expect("Error"),
                bitswap: beetswap::Behaviour::new(blockstore.clone()),
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))
            })
            .build();

        let (command_sender, command_receiver) = mpsc::channel(0);
        let (event_sender, event_receiver) = mpsc::channel(0);

        Ok((
            P2PCDNClient {
                blockstore: blockstore.clone(),
                queries: HashMap::new(),
                kad_queries: HashMap::new(),
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

    pub(crate) async fn find_providers(&mut self, cid: Cid) -> HashSet<PeerId> {
        let cid_key = RecordKey::new(&cid.to_bytes());
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetProviders {
                cid: cid_key,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub(crate) async fn get_listening_addr(
        &mut self,
    ) -> std::result::Result<String, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.command_sender
            .send(Command::GetActualListeningAddress { sender })
            .await
            .expect("No listening address found");
        receiver.await.expect("Failed to receive listening address")
    }

    pub async fn request_file(&mut self, cid: Cid) -> Result<Vec<u8>> {
        if !self.blockstore.has(&cid).await.expect("Error") {
            println!("CID {:?} not found in local blockstore.", cid);
            println!("Block store: {:?}", self.blockstore);
        }

        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::RequestFile {
                cid,
                sender,
            })
            .await?;

        let file_data = receiver.await??;
        Ok(file_data)
    }

    pub(crate) async fn find_peers(&mut self, target_peer_id: PeerId) {
        self.command_sender
            .send(Command::FindPeers { target_peer_id })
            .await
            .expect("Command receiver not to be dropped.");
    }
}

enum Command {
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
    FindPeers {
        target_peer_id: PeerId,
    },
    GetActualListeningAddress {
        sender: oneshot::Sender<std::result::Result<String, Box<dyn Error + Send>>>,
    },
    GetPeers {
        sender: oneshot::Sender<std::result::Result<Vec<PeerId>, Box<dyn Error + Send>>>,
    },
    GetProviders {
        cid: RecordKey,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
}


pub enum Event {
    FileUploaded(Cid),
    FileRequested(Cid),
    PeerDiscovered(PeerId),
}
pub struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
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
        event_sender: mpsc::Sender<Event>,
        blockstore: Arc<SledBlockstore>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
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
            SwarmEvent::Behaviour(BehaviourEvent::Bitswap(bitswap)) => {
                println!("Here");
                match bitswap {
                    beetswap::Event::GetQueryResponse { query_id, data } => {
                         self.queries.get(&query_id);

                        if let Some(sender) = self.pending_requests.remove(&query_id) {
                            // Send the received file data back to the original requester
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
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns_event)) => {
                if let mdns::Event::Discovered(peers) = mdns_event {
                    for (peer_id, multiaddr) in peers {
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, multiaddr.clone());
                        println!("Discovered Peer: {:?}", multiaddr);
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad_event)) => {
                if let kad::Event::OutboundQueryProgressed { id, result, .. } = kad_event {
                    if let kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { providers, .. })) = result {
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
            },
            _ => {
                println!("Did not match any specific event: {:?}", event);
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

                println!("Uploading file with CID: {}", cid);
                self.blockstore
                    .put_keyed(&cid, block.data())
                    .await
                    .map_err(|e| anyhow!("Failed to store block: {:?}", e))?;

                let cid_key = RecordKey::new(&cid.to_bytes());
                println!("Record Key {:?}", &cid_key);
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
            Command::RequestFile {
                cid,
                sender,
            } => {
                let query_id = self.swarm.behaviour_mut().bitswap.get(&cid);
                let kad_query_id = self.swarm.behaviour_mut().kademlia.get_providers(RecordKey::new(&cid.to_bytes()));
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
            Command::FindPeers { target_peer_id } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_closest_peers(target_peer_id);
                println!("{:?}", query_id);
            }
            Command::GetActualListeningAddress { sender } => {
                let mut listeners = self.swarm.listeners();
                if let Some(listener) = listeners.next() {
                    sender
                        .send(Ok(listener.to_string()))
                        .map_err(|e| anyhow!("Failed to send listener address: {:?}", e))?;
                } else {
                    sender
                        .send(Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            "No listening address found",
                        ))))
                        .map_err(|e| anyhow!("Failed to send error: {:?}", e))?;
                }
            }
            Command::GetPeers { sender } => {
                let peers: Vec<PeerId> = self.swarm.connected_peers().cloned().collect();
                sender
                    .send(Ok(peers))
                    .map_err(|e| anyhow!("Failed to send peers: {:?}", e))?;
            }
            Command::GetProviders { cid, sender } => {
                let query_id = self.swarm.behaviour_mut().kademlia.get_providers(cid);
                self.pending_get_providers.insert(query_id, sender);
                println!("Searching for providers for CID, query ID: {:?}", query_id);
            }
        }

        Ok(())
    }

    pub async fn run(mut self) {
        let mut listen_interval = interval(Duration::from_secs(300));

        loop {
            select! {
                    event = self.swarm.select_next_some() => {
                        // Handle swarm event
                        if let Err(e) = self.handle_event(event).await {
                            eprintln!("Error in event handling: {:?}", e);
                        }
                    },
                    command = self.command_receiver.next() => {
                        // Handle command
                        if let Some(command) = command {
                            if let Err(e) = self.handle_command(command).await {
                                eprintln!("Error in command handling: {:?}", e);
                            }
                        }
                    },
                    _ = listen_interval.tick() => {
                    // let listening_addr = self.swarm.listeners().next().cloned().expect("Error getting listening addr");

                    // if let Err(e) = self.swarm.listen_on(listening_addr) {
                    //     eprintln!("Failed to restart listening: {:?}", e);
                    // }
                }
            }
        }
    }
}
