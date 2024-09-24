// use libp2p::{
//     gossipsub::{self, Gossipsub, GossipsubConfig, MessageAuthenticity, Topic},
//     kad::{self, store::MemoryStore},
//     mdns,
//     swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
//     tcp,
//     yamux, noise,
//     Multiaddr, PeerId, identity,
// };
// use libp2p::Swarm;
// use libp2p::tcp::TcpConfig;
// use libp2p::core::upgrade::Version;
// use std::{collections::{HashMap, HashSet}, time::Duration};
// use std::error::Error;
// use std::fs::{File, write};
// use std::io::{self, Read, Seek, SeekFrom};
// use std::path::Path;
// use futures::Stream;
// use sha2::{Sha256, Digest};
// use tokio::sync::{mpsc, oneshot};
//
// const CHUNK_SIZE: usize = 1024 * 1024; // 1024KB
//
// // Utility function to split a file into chunks and return the chunks along with their hashes.
// fn split_file_into_chunks_with_hashes(file_path: &str) -> io::Result<Vec<(Vec<u8>, String)>> {
//     let mut file = File::open(file_path)?;
//     let mut chunks = Vec::new();
//     let mut buffer = vec![0; CHUNK_SIZE];
//
//     loop {
//         let bytes_read = file.read(&mut buffer)?;
//         if bytes_read == 0 {
//             break; // No more data to read
//         }
//
//         let chunk_data = buffer[..bytes_read].to_vec();
//         let mut hasher = Sha256::new();
//         hasher.update(&chunk_data);
//         let chunk_hash = format!("{:x}", hasher.finalize());
//
//         chunks.push((chunk_data, chunk_hash));  // Store both chunk and hash
//     }
//
//     Ok(chunks)
// }
//
// // Utility function to cache a chunk locally.
// fn cache_chunk(content_hash: &str, chunk_index: usize, chunk_data: &[u8]) -> io::Result<()> {
//     let cache_dir = format!("cache/{}", content_hash);
//     if !Path::new(&cache_dir).exists() {
//         std::fs::create_dir_all(&cache_dir)?;
//     }
//
//     let chunk_path = format!("{}/chunk_{}", cache_dir, chunk_index);
//     write(chunk_path, chunk_data)
// }
//
// // Utility function to verify a chunk by comparing its calculated hash with the expected hash.
// fn verify_chunk(chunk_data: &[u8], expected_hash: &str) -> bool {
//     let mut hasher = Sha256::new();
//     hasher.update(chunk_data);
//     let chunk_hash = format!("{:x}", hasher.finalize());
//
//     chunk_hash == expected_hash
// }
//
// #[derive(NetworkBehaviour)]
// struct Behaviour {
//     gossipsub: Gossipsub,
//     kademlia: kad::Behaviour<MemoryStore>,
//     mdns: mdns::tokio::Behaviour,
// }
//
// #[derive(Debug)]
// pub(crate) enum Command {
//     StartListening {
//         addr: Multiaddr,
//         sender: oneshot::Sender<Result<String, Box<dyn Error + Send>>>,
//     },
//     GetPeers {
//         sender: oneshot::Sender<Result<Vec<PeerId>, Box<dyn Error + Send>>>,
//     },
//     GetAvailablePeers {
//         sender: oneshot::Sender<Result<Vec<String>, Box<dyn Error + Send>>>,
//     },
//     Dial {
//         peer_id: PeerId,
//         peer_addr: Multiaddr,
//         sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
//     },
//     StartProvidingChunk {
//         content_hash: String,
//         chunk_index: i64,
//         sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
//     },
//     GetProviders {
//         content_hash: String,
//         sender: oneshot::Sender<HashSet<PeerId>>,
//     },
//     SubscribeChunkAnnouncements {
//         topic: Topic,
//     },
//     AnnounceChunk {
//         topic: Topic,
//         message: String,
//     },
//     RequestChunk {
//         content_hash: String,
//         chunk_index: i64,
//         peer_id: PeerId,
//         sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
//     },
// }
//
// pub(crate) struct Client {
//     sender: mpsc::Sender<Command>,
// }
//
// impl Client {
//     /// Listen for incoming connections on the given address.
//     pub(crate) async fn start_listening(
//         &mut self,
//         addr: Multiaddr,
//     ) -> Result<String, Box<dyn Error + Send>> {
//         let (sender, receiver) = oneshot::channel();
//         self.sender
//             .send(Command::StartListening { addr, sender })
//             .await
//             .expect("Command receiver not to be dropped.");
//         receiver.await.expect("Sender not to be dropped.")
//     }
//
//     pub(crate) async fn get_available_peers(&mut self) -> Result<Vec<String>, Box<dyn Error + Send>> {
//         let (sender, receiver) = oneshot::channel();
//         self.sender
//             .send(Command::GetAvailablePeers { sender })
//             .await
//             .expect("Command receiver not to be dropped.");
//         receiver.await.expect("Sender not to be dropped.")
//     }
//
//     pub(crate) async fn start_providing_chunk(&mut self, content_hash: String, chunk_index: usize) {
//         let (sender, receiver) = oneshot::channel();
//         self.sender
//             .send(Command::StartProvidingChunk {
//                 content_hash: content_hash.clone(),
//                 chunk_index: chunk_index as i64,
//                 sender,
//             })
//             .await
//             .expect("Command receiver not to be dropped.");
//
//         match receiver.await {
//             Ok(_) => println!("Successfully started providing chunk: {}_chunk_{}", content_hash, chunk_index),
//             Err(e) => eprintln!("Failed to start providing chunk: {}", e),
//         }
//     }
//
//     /// Announce a chunk along with its hash.
//     pub(crate) async fn announce_chunk_with_hash(
//         &mut self,
//         content_hash: String,
//         chunk_index: usize,
//         chunk_hash: String,
//     ) {
//         let topic = Topic::new("chunk-announcement");
//         let message = format!("{}_{}_{}", content_hash, chunk_index, chunk_hash);  // Include chunk hash
//         self.sender
//             .send(Command::AnnounceChunk { topic, message })
//             .await
//             .expect("Command receiver not to be dropped.");
//     }
//
//     /// Call this function after caching a chunk to announce it.
//     pub(crate) async fn cache_and_announce_chunk(
//         &mut self,
//         content_hash: String,
//         chunk_index: usize,
//         chunk_data: Vec<u8>,
//         chunk_hash: String,
//     ) -> io::Result<()> {
//         // Cache the chunk locally
//         cache_chunk(&content_hash, chunk_index, &chunk_data)?;
//
//         // Announce the cached chunk to the network with its hash
//         self.announce_chunk_with_hash(content_hash, chunk_index, chunk_hash).await;
//
//         Ok(())
//     }
//
//     /// Retrieve and verify all chunks of a file by checking their hashes.
//     pub(crate) async fn retrieve_and_verify_chunks(
//         &mut self,
//         content_hash: String,
//         total_chunks: usize,
//         chunk_hashes: Vec<String>,  // Expected chunk hashes
//     ) -> io::Result<Vec<u8>> {
//         let mut file_data = Vec::new();
//
//         for chunk_index in 0..total_chunks {
//             let providers = self.get_providers_for_chunk(content_hash.clone(), chunk_index).await;
//             if let Some(peer) = providers.iter().next() {
//                 // Request the chunk from the provider
//                 let chunk = self.request_chunk(peer.clone(), content_hash.clone(), chunk_index as i64).await?;
//
//                 // Verify the chunk
//                 let expected_hash = &chunk_hashes[chunk_index];
//                 if verify_chunk(&chunk, expected_hash) {
//                     println!("Chunk {} verified successfully!", chunk_index);
//                     file_data.extend(chunk);
//                 } else {
//                     eprintln!("Chunk {} failed verification!", chunk_index);
//                     return Err(io::Error::new(io::ErrorKind::InvalidData, "Chunk verification failed"));
//                 }
//             } else {
//                 eprintln!("No providers found for chunk {} of file {}", chunk_index, content_hash);
//             }
//         }
//
//         Ok(file_data)
//     }
//
//     pub(crate) async fn get_providers_for_chunk(
//         &mut self,
//         content_hash: String,
//         chunk_index: usize,
//     ) -> HashSet<PeerId> {
//         let chunk_id = format!("{}_chunk_{}", content_hash, chunk_index);
//         let (sender, receiver) = oneshot::channel();
//
//         self.sender
//             .send(Command::GetProviders {
//                 content_hash: chunk_id.clone(),
//                 sender,
//             })
//             .await
//             .expect("Command receiver not to be dropped.");
//
//         receiver.await.expect("Sender not to be dropped.")
//     }
//
//     /// Ensure that each chunk is cached by at least `n` peers for redundancy.
//     pub(crate) async fn ensure_redundancy_for_chunk(
//         &mut self,
//         content_hash: String,
//         chunk_index: usize,
//         chunk_data: Vec<u8>,
//         min_redundancy: usize,
//     ) -> io::Result<()> {
//         let providers = self.get_providers_for_chunk(content_hash.clone(), chunk_index).await;
//
//         if providers.len() < min_redundancy {
//             // Broadcast the chunk to additional peers or request them to cache it
//             for _ in 0..(min_redundancy - providers.len()) {
//                 // Send the chunk to new peers (via gossipsub or direct transfer)
//                 self.cache_and_announce_chunk(content_hash.clone(), chunk_index, chunk_data.clone(), String::new())
//                     .await?;
//             }
//         }
//
//         Ok(())
//     }
//
//     /// Subscribe to chunk announcements.
//     pub(crate) async fn subscribe_to_chunk_announcements(&mut self, topic_name: &str) {
//         let topic = Topic::new(topic_name.to_string());
//         self.sender
//             .send(Command::SubscribeChunkAnnouncements { topic })
//             .await
//             .expect("Command receiver not to be dropped.");
//     }
//
//     /// Request the chunk from a peer.
//     pub(crate) async fn request_chunk(
//         &mut self,
//         peer_id: PeerId,
//         content_hash: String,
//         chunk_index: i64,
//     ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
//         let (sender, receiver) = oneshot::channel();
//         self.sender
//             .send(Command::RequestChunk {
//                 content_hash,
//                 chunk_index,
//                 peer_id,
//                 sender,
//             })
//             .await
//             .expect("Command receiver not to be dropped.");
//         receiver.await.expect("Sender not be dropped.")
//     }
// }
//
// pub(crate) struct EventLoop {
//     swarm: Swarm<Behaviour>,
//     command_receiver: mpsc::Receiver<Command>,
//     pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
//     pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
//     pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
// }
//
// impl EventLoop {
//     fn new(
//         swarm: Swarm<Behaviour>,
//         command_receiver: mpsc::Receiver<Command>,
//     ) -> Self {
//         Self {
//             swarm,
//             command_receiver,
//             pending_dial: Default::default(),
//             pending_start_providing: Default::default(),
//             pending_get_providers: Default::default(),
//         }
//     }
//
//     pub(crate) async fn run(mut self) {
//         loop {
//             tokio::select! {
//                 event = self.swarm.select_next_some() => self.handle_event(event).await,
//                 command = self.command_receiver.recv() => match command {
//                     Some(c) => self.handle_command(c).await,
//                     None => return,  // Command channel closed
//                 },
//             }
//         }
//     }
//
//     async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
//         match event {
//             SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
//                                                                 propagation_source: peer_id,
//                                                                 message_id: _,
//                                                                 message,
//                                                             })) => {
//                 let chunk_announcement = String::from_utf8_lossy(&message.data);
//                 println!("Received chunk announcement from peer {}: {}", peer_id, chunk_announcement);
//
//                 // Parse content_hash and chunk_index from message
//                 let parts: Vec<&str> = chunk_announcement.split('_').collect();
//                 if parts.len() == 3 {
//                     let content_hash = parts[0].to_string();
//                     let chunk_index: i64 = parts[1].parse().expect("Chunk index to be an integer");
//                     let chunk_hash = parts[2].to_string();
//
//                     // Handle received chunk announcement (e.g., save info or request chunk)
//                     println!("Content Hash: {}, Chunk Index: {}, Chunk Hash: {}", content_hash, chunk_index, chunk_hash);
//                 }
//             }
//             SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
//                                                                id,
//                                                                result: kad::QueryResult::StartProviding(_),
//                                                                ..
//                                                            })) => {
//                 let sender = self.pending_start_providing.remove(&id).expect("Completed query to be previously pending.");
//                 let _ = sender.send(());
//             }
//             SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
//                 for (peer_id, multiaddr) in list {
//                     self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
//                 }
//             }
//             SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
//                                                                id,
//                                                                result: kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { providers, .. })),
//                                                                ..
//                                                            })) => {
//                 if let Some(sender) = self.pending_get_providers.remove(&id) {
//                     let _ = sender.send(providers);
//                 }
//             }
//             _ => {}
//         }
//     }
//
//     async fn handle_command(&mut self, command: Command) {
//         match command {
//             Command::StartListening { addr, sender } => {
//                 let peer_id = *self.swarm.local_peer_id();
//                 self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
//                 let _ = match self.swarm.listen_on(addr) {
//                     Ok(_) => sender.send(Ok(peer_id.to_string())),
//                     Err(e) => sender.send(Err(Box::new(e))),
//                 };
//             }
//             Command::GetAvailablePeers { sender } => {
//                 let peers: Vec<String> = self.swarm.connected_peers().map(|peer_id| peer_id.to_string()).collect();
//                 let _ = sender.send(Ok(peers));
//             }
//             Command::Dial { peer_id, peer_addr, sender } => {
//                 if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
//                     self.swarm.behaviour_mut().kademlia.add_address(&peer_id, peer_addr.clone());
//                     match self.swarm.dial(peer_addr.with(libp2p::multiaddr::Protocol::P2p(peer_id.into()))) {
//                         Ok(()) => { e.insert(sender); }
//                         Err(e) => { let _ = sender.send(Err(Box::new(e))); }
//                     }
//                 }
//             }
//             Command::StartProvidingChunk { content_hash, chunk_index, sender } => {
//                 let chunk_id = format!("{}_chunk_{}", content_hash, chunk_index);  // Unique chunk ID
//                 let query_id = self.swarm.behaviour_mut().kademlia.start_providing(chunk_id.into_bytes().into())
//                     .expect("No store error.");
//                 self.pending_start_providing.insert(query_id, sender);
//             }
//             Command::GetProviders { content_hash, sender } => {
//                 let query_id = self.swarm.behaviour_mut().kademlia.get_providers(content_hash.into_bytes().into());
//                 self.pending_get_providers.insert(query_id, sender);
//             }
//             Command::SubscribeChunkAnnouncements { topic } => {
//                 let _ = self.swarm.behaviour_mut().gossipsub.subscribe(&topic);
//             }
//             Command::AnnounceChunk { topic, message } => {
//                 if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, message.as_bytes()) {
//                     eprintln!("Error publishing chunk announcement: {:?}", e);
//                 }
//             }
//             Command::RequestChunk { content_hash, chunk_index, peer_id, sender } => {
//                 // In a real-world scenario, you'd request the chunk from the peer
//                 // Simulate chunk retrieval
//                 let chunk_id = format!("{}_chunk_{}", content_hash, chunk_index);
//                 let data = vec![0; 1024];  // Simulated chunk data
//                 let _ = sender.send(Ok(data));
//             }
//         }
//     }
// }
//
