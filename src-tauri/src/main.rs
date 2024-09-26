mod content;
mod network;
mod node;
use crate::content::ContentManager;
use crate::node::{
    load_or_generate_keypair, load_peer_info, save_node_type, save_peer_info, NodeType,
};
use futures::future::FutureExt;
use tokio_stream::StreamExt;
use libp2p::{core::Multiaddr, multiaddr::Protocol, PeerId};
use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;
use base64::encode;
use tauri::{async_runtime::spawn, State};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    network_client: Arc<Mutex<network::Client>>,
    network_events: Arc<Mutex<Pin<Box<dyn futures::Stream<Item = network::Event> + Send>>>>,
    content_manager: Arc<Mutex<ContentManager>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // Initialize the network
    let (network_client, network_events, network_event_loop) = network::new(None).await?;

    // Spawn the network task to run in the background
    spawn(network_event_loop.run());

    let content_manager = ContentManager::new()
        .await
        .expect("Failed to initialize content manager");
    let app_state = AppState {
        network_client: Arc::new(Mutex::new(network_client)),
        network_events: Arc::new(Mutex::new(Box::pin(network_events))),
        content_manager: Arc::new(Mutex::new(content_manager)),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_listening,
            list_peers,
            dial_peer,
            provide_file,
            get_file,
            load_peer,
            save_peer,
            retrieve_available_peers,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Box Peer");

    Ok(())
}
#[tauri::command]
async fn retrieve_and_reassemble_file(
    state: State<'_, AppState>,
    content_hash: String,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file_chunks = Vec::new();
    let mut client = state.network_client.lock().await;

    let mut index = 0;

    loop {
        let chunk_index = format!("{}_chunk_{}", content_hash, index);

        let providers = client.get_providers(chunk_index.clone()).await;

        if providers.is_empty() {
            // If no providers are found for this chunk, assume we've reached the end
            break;
        }

        // Choose the first provider and request the file chunk
        let peer = providers.into_iter().next().unwrap();
        let chunk_data = client.request_file(peer, chunk_index).await.expect("Error requesting file");

        file_chunks.push(chunk_data);

        index += 1;
    }

    let reassembled_file = file_chunks.into_iter().flatten().collect::<Vec<u8>>();

    Ok(reassembled_file)
}

#[tauri::command]
async fn start_listening(state: State<'_, AppState>) -> Result<String, String> {
    let peer_info = load_peer_info();
    let addr = peer_info
        .as_ref()
        .and_then(|info| {
            if info.listening_addr.is_empty() {
                None
            } else {
                Some(info.listening_addr.clone())
            }
        })
        .unwrap_or("/ip4/0.0.0.0/tcp/0".to_string());

    let parsed_addr = addr.parse::<Multiaddr>().map_err(|e| e.to_string())?;

    let mut client = state.network_client.lock().await;

    let id = client
        .start_listening(parsed_addr.clone())
        .await
        .map_err(|e| e.to_string())?;

    let peer_id: PeerId = id.parse().expect("Error parsing");
    client.find_peers(peer_id).await;
    let actual_listening_addr = client
        .get_listening_addr()
        .await
        .expect("Error getting listening address");
    let existing_nodes = state.content_manager.lock().await.get_nodes().await?;
    if !existing_nodes.iter().any(|node| *node == peer_id) {
        state.content_manager.lock().await.add_node(peer_id.to_string()).await?;
    }

    if peer_info.is_none()
        || peer_info.as_ref().unwrap().peer_id.is_empty()
        || peer_info.as_ref().unwrap().listening_addr.is_empty()
    {
        save_peer_info(&id, &actual_listening_addr).expect("Error saving peer info");
    }

    let cached_contents = state
        .content_manager
        .lock()
        .await
        .read_cached_chunks()
        .await?;
    for content_hash in cached_contents {
        client.start_providing(content_hash).await;
    }
    Ok(id)
}

#[tauri::command]
async fn load_peer() -> Option<node::PeerInfo> {
    load_peer_info()
}
#[tauri::command]
async fn save_peer(node_type: NodeType) {
    save_node_type(node_type).expect("Error saving node type")
}

#[tauri::command]
async fn lock_content(
    state: State<'_, AppState>,
    content_hash: String,
    chunk_index: i64,
    chunk_size: i64,
) -> Result<(), String> {
    let mut client = state.network_client.lock().await;
    let peer_info = load_peer_info().expect("Peer Id not found");
    state
        .content_manager
        .lock()
        .await
        .lock_content(
            content_hash.clone(),
            chunk_index,
            chunk_size,
            peer_info.peer_id,
        )
        .await
        .expect("Error locking content");
    client.start_providing(content_hash.clone()).await;
    Ok(())
}

#[tauri::command]
async fn unlock_content(state: State<'_, AppState>, content_hash: String) -> Result<(), String> {
    let mut client = state.network_client.lock().await;
    let peer_info = load_peer_info().expect("Peer Id not found");
    state
        .content_manager
        .lock()
        .await
        .unlock_content(content_hash.clone(), peer_info.peer_id)
        .await
        .expect("Error locking content");

    client.start_providing(content_hash.clone()).await;
    Ok(())
}

#[tauri::command]
async fn get_peerid_locked_content(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let content_manager = state.content_manager.lock().await;
    let peer_info = load_peer_info().expect("Peer Id not found");
    // Fetch locked content hashes for the given peer_id
    match content_manager
        .get_peerid_locked_content(peer_info.peer_id)
        .await
    {
        Ok(content_list) => Ok(content_list),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn get_peerid_provided_content(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let content_manager = state.content_manager.lock().await;

    let peer_info = load_peer_info().expect("Peer Id not found");
    match content_manager
        .get_peerid_provided_content(peer_info.peer_id)
        .await
    {
        Ok(content_list) => Ok(content_list),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn get_all_locked_content(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let content_manager = state.content_manager.lock().await;

    match content_manager.get_locked_content().await {
        Ok(content_list) => Ok(content_list),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn list_peers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut client = state.network_client.lock().await;
    match client.get_peers_count().await {
        Ok(peers) => {

            let peer_strings: Vec<String> =
                peers.into_iter().map(|peer| peer.to_string()).collect();
            println!("{:?}", peer_strings.clone());
            Ok(peer_strings)
        }
        Err(e) => {
            println!("{e}");
            Err(format!("Failed to get peers: {}", e))
        }
    }
}
#[tauri::command]
async fn dial_peer(state: State<'_, AppState>, addr: String) -> Result<(), String> {
    let parsed_addr = addr.parse::<Multiaddr>().map_err(|e| e.to_string())?;
    println!("{:?}", parsed_addr);
    let Some(Protocol::P2p(peer_id)) = parsed_addr.iter().last() else {
        return Err("Expect peer multiaddr to contain peer ID.".to_string());
    };

    let mut client = state.network_client.lock().await;
    client
        .dial(peer_id, parsed_addr)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn retrieve_available_peers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut client = state.network_client.lock().await;

    let peers = client.get_available_peers().await.map_err(|e| e.to_string())?;
    Ok(peers.into_iter().map(|peer| peer.to_string()).collect())
}


#[tauri::command]
async fn provide_file(
    state: State<'_, AppState>,
    path: String,
    content_hash: String,
    file_name: String,
) -> Result<(), String> {
    let peer_info = load_peer_info().ok_or("Peer Id not found".to_string())?;

    let file_size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
    let mut client = state.network_client.lock().await;

    state
        .content_manager
        .lock()
        .await
        .add_provided_content(
            content_hash.clone(),
            peer_info.peer_id.to_string(),
            file_size,
            file_name,
        )
        .await
        .map_err(|_| "Error providing content".to_string())?;

    // Define the chunk size (4 MB)
    let chunk_size = 4 * 1024 * 1024; // 4 MB chunk size

    // Split the file into chunks
    let file_chunks = state
        .content_manager
        .lock()
        .await
        .split_file_into_chunks(&path, chunk_size)
        .await
        .map_err(|_| "Error splitting file".to_string())?;

    // Get the available peers
    let peers = client
        .get_available_peers()
        .await
        .map_err(|_| "Error getting peers".to_string())?;

    // Distribute file chunks among peers
    let peer_map = state
        .content_manager
        .lock()
        .await
        .distribute_file_chunks(content_hash.clone(), peers, file_chunks.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Send file chunks to each assigned peer
    let mut index = 1;
    for (peer, chunk_index) in peer_map {
        // Prepare the content_chunk_index for this peer
        let content_chunk_index = format!("{}_chunk_{}", content_hash.clone(), chunk_index);
    let content_chunk_hash = encode(file_chunks[index - 1].clone());
        // Send the corresponding chunk to the peer
        client
            .send_chunk_to_peer(peer, content_chunk_index.clone(), content_chunk_hash)
            .await
            .map_err(|e| e.to_string())?;
        index += 1;
    }

    // Notify peers to provide the chunks
    // for (peer, content_chunk_index) in peer_chunk_map {
    //     client.notify_peer_to_provide_chunk(peer, content_chunk_index).await?;
    // }

    client.start_providing(content_hash.clone()).await;
    println!("Uploader Providing...");

    // Handle incoming requests
    let mut network_events = state.network_events.lock().await;
    while let Some(network::Event::InboundRequest { request, channel }) =
        network_events.next().await
    {
        if request == content_hash {
            let file_data = std::fs::read(&path).map_err(|e| e.to_string())?;
            client.respond_file(file_data, channel).await;
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_file(state: State<'_, AppState>, content_hash: String) -> Result<Vec<u8>, String> {
    let mut client = state.network_client.lock().await;
    let providers = client.get_providers(content_hash.clone()).await;

    if providers.is_empty() {
        return Err(format!("Could not find provider for file {content_hash}."));
    }

    let requests = providers.into_iter().map(|p| {
        let mut client = client.clone();
        let content_hash = content_hash.clone();
        async move { client.request_file(p, content_hash).await }.boxed()
    });

    let file_content = futures::future::select_ok(requests)
        .await
        .map_err(|_| "None of the providers returned file.".to_string())?
        .0;
    println!("{:?}", file_content);
    Ok(file_content)
}
