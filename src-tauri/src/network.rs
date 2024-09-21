use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use futures::StreamExt;
use std::result::Result;

use libp2p::multiaddr::Protocol;
use libp2p::{
    core::Multiaddr,
    identity, kad, noise,
    request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol,
};

use crate::node::load_or_generate_keypair;
use libp2p::mdns;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

pub(crate) async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop), Box<dyn Error>> {
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => load_or_generate_keypair(),
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
                    StreamProtocol::new("/file-exchange/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                .expect("Error"),
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
    ) -> Result<(String), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Get all connected peers
    pub(crate) async fn get_peers_count(&mut self) -> Result<Vec<PeerId>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetPeers { sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    pub(crate) async fn get_available_peers(
        &mut self,
    ) -> Result<Vec<PeerId>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetAvailablePeers { sender })
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

    /// Advertise the local node as the provider of the given file on the DHT.
    pub(crate) async fn start_providing(&mut self, content_chunk_index: String) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding {
                content_chunk_index: content_chunk_index.clone(),
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        // Wait for the result of the operation
        match receiver.await {
            Ok(_) => {
                println!("Successfully started providing content: {}", content_chunk_index);
            }
            Err(e) => {
                eprintln!("Failed to start providing content: {}", e);
            }
        }
    }

    /// Find the providers for the given file on the DHT.
    pub(crate) async fn get_providers(&mut self, content_chunk_index: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetProviders {
                content_chunk_index,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Request the content of the given file from the given peer.
    pub(crate) async fn request_file(
        &mut self,
        peer: PeerId,
        content_chunk_index: String,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestFile {
                content_chunk_index,
                peer,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond with the provided file content to the given request.
    pub(crate) async fn respond_file(
        &mut self,
        file: Vec<u8>,
        channel: ResponseChannel<FileResponse>,
    ) {
        self.sender
            .send(Command::RespondFile { file, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }
    pub(crate) async fn find_peers(&mut self, target_peer_id: PeerId) {
        // let (sender, _receiver) = oneshot::channel();
        self.sender
            .send(Command::FindPeers { target_peer_id })
            .await
            .expect("Command receiver not to be dropped.");
    }

    pub async fn notify_peer_to_provide_chunk(
        &mut self,
        peer: PeerId,
        content_chunk_index: String,
    ) -> Result<(), String> {
        // Send a command to the peer telling them to provide the chunk
        self.sender
            .send(Command::StartProvidingRequest {
                peer,
                content_chunk_index,
            })
            .await
            .map_err(|e| format!("Failed to send StartProvidingRequest command: {}", e))?;

        Ok(())
    }


    pub(crate) async fn get_listening_addr(&mut self) -> Result<String, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Command::GetActualListeningAddress { sender })
            .await
            .expect("No listening address found");
        receiver.await.expect("Failed to receive listening address")
    }
}

pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    pending_request_file:
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
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            pending_request_file: Default::default(),
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
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result: kad::QueryResult::StartProviding(_),
                    ..
                },
            )) => {
                let sender: oneshot::Sender<()> = self
                    .pending_start_providing
                    .remove(&id)
                    .expect("Completed query to be previously pending.");
                let _ = sender.send(());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::RoutingUpdated {
                peer,
                addresses,
                ..
            })) => {
                // Peer discovered, dial it
                if let address = addresses.first() {
                    println!("Discovered peer via Kademlia: {:?} at {:?}", peer, address);

                    // Clone the address before passing it to dial
                    if self.swarm.dial(address.clone()).is_ok() {
                        println!("Dialing discovered peer: {:?}", peer);
                    } else {
                        println!("Failed to dial peer: {:?}", peer);
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multiaddr);
                }
            }

            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    id,
                    result:
                        kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                            providers,
                            ..
                        })),
                    ..
                },
            )) => {
                if let Some(sender) = self.pending_get_providers.remove(&id) {
                    sender.send(providers).expect("Receiver not to be dropped");

                    // Finish the query. We are only interested in the first result.
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .query_mut(&id)
                        .unwrap()
                        .finish();
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result:
                        kad::QueryResult::GetProviders(Ok(
                            kad::GetProvidersOk::FinishedWithNoAdditionalRecord { .. },
                        )),
                    ..
                },
            )) => {}
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(_)) => {}
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.event_sender
                        .send(Event::InboundRequest {
                            request: request.0,
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
                        .pending_request_file
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
                    .pending_request_file
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(Box::new(error)));
            }
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {}
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                eprintln!(
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
                        println!("Connection established with peer in If: {:?}", peer_id);
                    }
                }
                println!("Connection established with peer: {:?}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Connection closed with peer: {:?}", peer_id);
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                eprintln!(
                    "Failed to connect to peer: {:?}, error: {:?}",
                    peer_id, error
                );
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
                let peer_id = *self.swarm.local_peer_id();
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(peer_id.to_string())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }
            Command::GetPeers { sender } => {
                let peers: Vec<PeerId> = self.swarm.connected_peers().cloned().collect(); // Clone the PeerIds
                let _ = sender.send(Ok(peers)); // Send the list of peers as Ok
            }
            Command::GetAvailablePeers { sender } => {
                let mut available_peers = Vec::new();
                let peers: Vec<PeerId> = self.swarm.connected_peers().cloned().collect();
                for peer_id in peers {
                    // Convert the peer_id to a string (or another format as needed)
                    available_peers.push(peer_id);
                }
                let _ = sender.send(Ok(available_peers));
            }
            Command::StartProvidingRequest {peer, content_chunk_index} => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, FileRequest(content_chunk_index));

                println!(
                    "Sent request to peer {} to start providing",
                    peer,
                );
            }
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                // println!("Peer Id {:?}", peer_id);
                // println!("Peer Address{:?}", peer_addr);
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
                    println!("Already dialing peer.");
                }
            }
            Command::StartProviding {
                content_chunk_index,
                sender,
            } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(content_chunk_index.into_bytes().into())
                    .expect("No store error.");
                self.pending_start_providing.insert(query_id, sender);
            }
            Command::GetProviders {
                content_chunk_index,
                sender,
            } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(content_chunk_index.into_bytes().into());
                self.pending_get_providers.insert(query_id, sender);
            }
            Command::RequestFile {
                content_chunk_index,
                peer,
                sender,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, FileRequest(content_chunk_index));
                self.pending_request_file.insert(request_id, sender);
            }
            Command::RespondFile { file, channel } => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, FileResponse(file))
                    .expect("Connection to peer to be still open.");
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
                    let _ = sender.send(Ok(listener.to_string())); // Send the result through the channel
                } else {
                    let _ = sender.send(Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "No listening address found",
                    ))));
                }
            }

        }
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    request_response: request_response::cbor::Behaviour<FileRequest, FileResponse>,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<String, Box<dyn Error + Send>>>,
    },
    GetPeers {
        sender: oneshot::Sender<Result<Vec<PeerId>, Box<dyn Error + Send>>>,
    },
    GetAvailablePeers {
        sender: oneshot::Sender<Result<Vec<PeerId>, Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    StartProviding {
        content_chunk_index: String,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        content_chunk_index: String,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    RequestFile {
        content_chunk_index: String,
        peer: PeerId,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
    RespondFile {
        file: Vec<u8>,
        channel: ResponseChannel<FileResponse>,
    },
    FindPeers {
        target_peer_id: PeerId,
        // sender: oneshot::Sender<HashSet<PeerId>>,
    },
    GetActualListeningAddress {
        sender: oneshot::Sender<Result<String, Box<dyn Error + Send>>>,
    },
    StartProvidingRequest {
        peer: PeerId,
        content_chunk_index: String,

    },
}

#[derive(Debug)]
pub(crate) enum Event {
    InboundRequest {
        request: String,
        channel: ResponseChannel<FileResponse>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);


