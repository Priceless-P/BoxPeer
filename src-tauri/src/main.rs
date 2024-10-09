#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod net;
mod node;
use std::net::Ipv4Addr;
use tauri::{async_runtime::spawn, Manager, State};
use libp2p::multiaddr::{Multiaddr, Protocol};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use crate::net::P2PCDNClient;
use anyhow::Result;
use cid::Cid;
use libp2p_core::PeerId;
use tokio::sync::Mutex as AsyncMutex;

struct AppState {
    client: Arc<AsyncMutex<P2PCDNClient>>,
    network_events: Arc<AsyncMutex<Pin<Box<dyn futures::Stream<Item = net::Event> + Send>>>>,
}

#[tauri::command]
async fn start_listening(state: State<'_, AppState>) -> Result<String, String> {
    let address_webrtc = Multiaddr::from(Ipv4Addr::new(127, 0, 0, 1))
        .with(Protocol::Udp(9090))
        .with(Protocol::WebRTCDirect);
    println!("{:?}", address_webrtc);

    let mut client = state.client.lock().await;

    let id = client
        .start_listening(address_webrtc.clone())
        .await
        .map_err(|e| e.to_string())?;

    let peer_id: PeerId = id.parse().expect("Error parsing");
    Ok(id)
}

#[tauri::command]
async fn list_peers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut client = state.client.lock().await;
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

// Tauri command to upload a file.
#[tauri::command]
async fn upload_file(state: State<'_, AppState>, file_path: String) -> Result<String, String> {
    let path = PathBuf::from(file_path);
    let mut client = state.client.lock().await;
    client
        .upload_file(path)
        .await
        .map(|cid| cid.to_string())
        .map_err(|e| e.to_string())
}

// Tauri command to request a file by CID and save it to the given path.
#[tauri::command]
async fn request_file(
    state: State<'_, AppState>,
    cid: String,
    save_path: String,
) -> Result<Vec<u8>, String> {
    let cid = cid
        .parse()
        .map_err(|e| format!("Request file error: {}", e))?;
    let path = PathBuf::from(save_path);
    let mut client = state.client.lock().await;
    client.request_file(cid).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn request_files(
    state: State<'_, AppState>,
    cid_strings: Vec<String>,
) -> Result<Vec<Vec<u8>>, String> {
    let cids: Result<Vec<Cid>, _> = cid_strings.into_iter().map(|s| Cid::try_from(s)).collect();
    let cids = cids.map_err(|e| format!("Invalid CID: {}", e))?;
    let mut client = state.client.lock().await;
    client.get_all_files(cids).await.map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, network_events, network_event_loop) = P2PCDNClient::new(None, None).await?;
    spawn(network_event_loop.run());
    let app_state = AppState {
        client: Arc::new(AsyncMutex::new(client)),
        network_events: Arc::new(AsyncMutex::new(Box::pin(network_events))),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_listening,
            upload_file,
            list_peers,
            request_file,
            request_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running BoxPeer application");

    Ok(())
}
