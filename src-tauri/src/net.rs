use futures::{SinkExt, Stream, StreamExt};
use libp2p::{kad, mdns, noise, tcp, yamux, swarm::{NetworkBehaviour, SwarmEvent}, Multiaddr, PeerId as Id, identity, Swarm, SwarmBuilder};
use libp2p_bitswap::{Bitswap, BitswapConfig, BitswapEvent};
use libp2p_core::{Endpoint, PeerId};
use std::error::Error;
use anyhow::Result;
use blockstore::{
    block::{Block},
    Blockstore, InMemoryBlockstore,
};
use beetswap;
use cid::Cid;
use libipld::{ store::DefaultParams};
use std::time::Duration;
use futures::channel::{mpsc, oneshot};
use tokio::select;
use std::collections::{HashMap, HashSet};
use std::task::{Context, Poll};
use blockstore::block::CidError;
use multihash_codetable::{Code, MultihashDigest};
use libp2p::swarm::{ConnectionDenied, ConnectionId, FromSwarm, THandler, THandlerInEvent, THandlerOutEvent, ToSwarm};
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use libp2p::{
    kad::{store::MemoryStore, Record},
};
use libp2p_bitswap::{BitswapStore};
use libp2p_kad::RecordKey;
use tauri::api::path::cache_dir;
use tracing::{debug, info, error};
use crate::node::load_or_generate_keypair;

const MAX_MULTIHASH_SIZE: usize = 64;

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
    bitswap: beetswap::Behaviour<64, InMemoryBlockstore<64>>,
    mdns: mdns::tokio::Behaviour,
    kademlia: kad::Behaviour<MemoryStore>,
}

pub struct P2PCDNClient {
    blockstore: Arc<InMemoryBlockstore<64>>,
    queries: HashMap<beetswap::QueryId, Cid>,
    command_sender: mpsc::Sender<Command>,
}

impl P2PCDNClient {
    pub async fn new(
        bootstrap_peers: Option<Vec<Multiaddr>>,
        secret_key_seed: Option<u8>,
    ) -> std::result::Result<(P2PCDNClient, impl Stream<Item=Event>, EventLoop), Box<dyn Error>> {

        let id_keys = match secret_key_seed {
            Some(seed) => {
                let mut bytes = [0u8; 32];
                bytes[0] = seed;
                identity::Keypair::ed25519_from_bytes(bytes)?
            }
            None => load_or_generate_keypair(),
        };

        let peer_id = id_keys.public().to_peer_id();
        let blockstore = Arc::new(InMemoryBlockstore::new());

        let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| Behaviour {
                kademlia: kad::Behaviour::new(
                    peer_id,
                    MemoryStore::new(key.public().to_peer_id()),
                ),
                mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id()).expect("Error"),
                bitswap: beetswap::Behaviour::new(blockstore.clone()),
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60000)))
            .build();

        let (command_sender, command_receiver) = mpsc::channel(0);
        let (event_sender, event_receiver) = mpsc::channel(0);

        Ok((P2PCDNClient {
            blockstore,
            queries: HashMap::new(),
            command_sender,

        },
           event_receiver,
           EventLoop::new(swarm, command_receiver, event_sender)))
    }
    pub(crate) async fn get_peers_count(&mut self) -> std::result::Result<Vec<PeerId>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::GetPeers { sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<String> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub async fn upload_file(&mut self, file_path: PathBuf) -> Result<Cid> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender.send(Command::UploadFile {
            file_path,
            sender,
        }).await?;

        let cid = receiver.await??;
        Ok(cid)
    }
    pub(crate) async fn get_listening_addr(&mut self) -> std::result::Result<String, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.command_sender
            .send(Command::GetActualListeningAddress { sender })
            .await
            .expect("No listening address found");
        receiver.await.expect("Failed to receive listening address")
    }

    pub async fn request_file(&mut self, cid: Cid, save_path: PathBuf) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.command_sender.send(Command::RequestFile { cid, save_path, sender})
            .await.expect("Error requesting file");
        receiver.await.expect("Failed to receive listening address").expect("TODO: panic message");

        Ok(())
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
        save_path: PathBuf,
        sender: oneshot::Sender<Result<()>>,
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
}

pub(crate) enum Event {
    FileUploaded(Cid),
    FileRequested(Cid),
    PeerDiscovered(PeerId),
}
pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    queries: HashMap<beetswap::QueryId, Cid>,
}
impl EventLoop {
    pub(crate) fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            queries: Default::default(),
        }
    }
    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Bitswap(bitswap)) => {
                match bitswap {
                    beetswap::Event::GetQueryResponse { query_id, data } => {
                        println!("GetQueryResponse triggered for query_id: {:?}", query_id);
                        if let Some(cid) = self.queries.get(&query_id) {
                            info!("Received response for CID {:?}: {:?}", cid, data);

                            if let Some(cache_path) = cache_dir() {
                                let mut file_path = PathBuf::from(cache_path);
                                file_path.push("Boxpeer");
                                file_path.push(cid.to_string());
                                println!("{:?}", &file_path);

                                if let Err(e) = fs::create_dir_all(&file_path.parent().unwrap()) {
                                    println!("Failed to create cache directory: {:?}", e);
                                }

                                if let Err(e) = fs::write(&file_path, data) {
                                    println!("Failed to save the file: {:?}", e);
                                } else {
                                    println!("File saved to: {:?}", file_path);
                                }
                            } else {
                                println!("Could not find cache dir");
                            }
                        }
                    },
                    beetswap::Event::GetQueryError { query_id, error } => {
                        if let Some(cid) = self.queries.get(&query_id) {
                            info!("Error for CID {:?}: {:?}", cid, error);
                        }
                    },
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns_event)) => {
                if let mdns::Event::Discovered(peers) = mdns_event {
                    for (peer_id, multiaddr) in peers {
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, multiaddr.clone());
                        println!("Discovered Peer: {:?}", multiaddr)
                    }
                }
            },
            // SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad_event)) => {
            //     if let kad::Event::OutboundQueryProgressed { id, result, .. } = kad_event {
            //         if let kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { providers, .. })) = result {
            //             if let Some(mut cid) = self.queries.get(&id) {
            //                 if providers.is_empty() {
            //                     println!("No providers found for CID: {:?}", cid);
            //                 }
            //                 for (peer) in providers {
            //                     let query_id = self.swarm.behaviour_mut().kademlia.get_closest_peers(peer);
            //                     println!("Found provider: {:?}", peer);
            //
            //                     println!("Requesting file with CID: {:?}", cid);
            //
            //                     // Use Bitswap to request the file from the provider.
            //                     let query_id = self.swarm.behaviour_mut().bitswap.get(&(*cid));
            //                     println!("Here query: {:?}", query_id);
            //                     self.queries.insert(query_id, *cid);
            //                 }
            //             }
            //         }
            //     }
            // },
            _ => {}
        }
    }


    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::UploadFile { file_path, sender } => {
                let blockstore: Arc<InMemoryBlockstore<MAX_MULTIHASH_SIZE>> = Arc::new(InMemoryBlockstore::new());

                // Read the file as binary data
                let file_data = match fs::read(&file_path) {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Failed to read file from {:?}: {:?}", file_path, e);
                        let _ = sender.send(Err(anyhow::anyhow!("File read error: {:?}", e)));
                        return;
                    }
                };

                // Create the file block
                let block = FileBlock(file_data);

                // Generate the CID
                let cid = match block.cid() {
                    Ok(c) => c,
                    Err(e) => {
                        println!("Failed to generate CID: {:?}", e);
                        let _ = sender.send(Err(anyhow::anyhow!("CID generation error: {:?}", e)));
                        return;
                    }
                };

                println!("Uploading file with CID: {}", cid);
                if let Err(e) = blockstore.put_keyed(&cid, block.data()).await {
                    println!("Failed to store block: {:?}", e);
                    let _ = sender.send(Err(anyhow::anyhow!("Block storage error: {:?}", e)));
                    return;
                }

                let cid_key = RecordKey::new(&cid.to_bytes());
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(cid_key)
                    .expect("Failed to start providing the CID");

                // Send the CID as the result of the upload
                let _ = sender.send(Ok(cid));
            },
            Command::RequestFile { cid, save_path, sender } => {
                //let cid_key = RecordKey::new(&cid.to_bytes());
                let query_id = self.swarm.behaviour_mut().bitswap.get(&cid);
                println!("Bitswap query ID: {:?} for CID: {:?}", query_id, cid);

                self.queries.insert(query_id, cid);

                let _ = sender.send(Ok(()));
            },
            Command::StartListening { addr, sender } => {
                let peer_id = *self.swarm.local_peer_id();
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(peer_id.to_string())),
                    Err(e) => sender.send(Ok(e.to_string())),
                };
            },
            Command::FindPeers { target_peer_id } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_closest_peers(target_peer_id);
                println!("{:?}", query_id);
            },
            Command::GetActualListeningAddress { sender } => {
                let mut listeners = self.swarm.listeners();
                if let Some(listener) = listeners.next() {
                    let _ = sender.send(Ok(listener.to_string()));
                } else {
                    let _ = sender.send(Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "No listening address found",
                    ))));
                }
            },
            Command::GetPeers { sender } => {
                let peers: Vec<PeerId> = self.swarm.connected_peers().cloned().collect();
                let _ = sender.send(Ok(peers));
            }
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    None=>  return,
                },
            }
        }
    }

}