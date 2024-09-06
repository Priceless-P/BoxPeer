mod network;
use futures::prelude::*;
use futures::StreamExt;
use libp2p::{core::Multiaddr, multiaddr::Protocol};
use std::path::PathBuf;
use tokio::task::spawn;
use tracing_subscriber::EnvFilter;
use tauri::command;
use std::sync::Mutex;

// NetworkManager to maintain network state
struct NetworkManager {
    client: Option<network::Client>,
    event_loop_handle: Option<tokio::task::JoinHandle<()>>,
}

impl NetworkManager {
    fn new() -> Self {
        Self {
            client: None,
            event_loop_handle: None,
            network_events: None,
        }
    }

    async fn initialize(&mut self, secret_key_seed: Option<u8>) -> Result<(), String> {
        if self.client.is_none() {
            let (mut client, mut network_events, event_loop) = network::new(secret_key_seed)
                .await
                .map_err(|e| e.to_string())?;
            self.event_loop_handle = Some(tokio::spawn(event_loop.run()));
            self.client = Some(client);
            self.network_events = Some(network_events);
        }
        Ok(())
    }

    fn get_client(&self) -> Result<&network::Client, String> {
        self.client.as_ref().ok_or("Network client not initialized".to_string())
    }
}

// Use a static to hold the network manager instance
lazy_static::lazy_static! {
    static ref NETWORK_MANAGER: Mutex<NetworkManager> = Mutex::new(NetworkManager::new());
}

// Tauri command to start the network
#[command]
fn start(secret_key_seed: Option<u8>, listen_address: Option<String>, peer: Option<String>) -> Result<(), String> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // Spawn async task to handle network startup
    tokio::spawn(async move {
        let mut network_manager = NETWORK_MANAGER.lock().unwrap();

        // Initialize network
        network_manager.initialize(secret_key_seed).await?;

        let network_client = network_manager.network_events()?;

        // Listen on provided or default address
        match listen_address {
            Some(addr_str) => {
                let addr: Multiaddr = addr_str.parse();

                network_client.start_listening(addr).await.map_err(|e| e.to_string())?;
            }
            None => {
                network_client.start_listening("/ip4/0.0.0.0/tcp/0".parse().map_err(|e| e.to_string())?)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        // If peer address is provided, dial it
        if let Some(peer_str) = peer {
            let addr: Multiaddr = peer_str.parse().map_err(|e| e.to_string())?;
            let Some(Protocol::P2p(peer_id)) = addr.iter().last() else {
                return Err("Peer multiaddr must contain peer ID".to_string());
            };
            network_client.dial(peer_id, addr).await.map_err(|e| e.to_string())?;
        }

        Ok::<(), String>(())
    });

    Ok(())
}

#[tauri::command]
async fn provide_file(name: String, path: PathBuf) -> Result<(), String> {
    let mut network_manager = NETWORK_MANAGER.lock().unwrap();

    let network_client = network_manager.get_client()?;
    network_client.start_providing(name.clone()).await;

    // Spawn task to listen to network events and provide file
    tokio::spawn(async move {
        let mut network_events = network_client.clone().events();

        while let Some(event) = network_events.next().await {
            match event {
                network::Event::InboundRequest { request, channel } if request == name => {
                    let file_content = std::fs::read(&path).map_err(|e| e.to_string())?;
                    network_client.respond_file(file_content, channel).await.map_err(|e| e.to_string())?;
                }
                _ => (),
            }
        }
    });

    Ok::<(), String>(())
}

#[tauri::command]
async fn get_file(name: String) -> Result<Vec<u8>, String> {
    let mut network_manager = NETWORK_MANAGER.lock().unwrap();

    let network_client = network_manager.get_client()?;

    // Locate providers for the file
    let providers = network_client.clone().get_providers(name.clone()).await;
    if providers.is_empty() {
        return Err(format!("No provider found for file: {name}"));
    }

    // Request file from providers
    let requests = providers.into_iter().map(|p| {
        let mut client_clone = network_client.clone();
        let name_clone = name.clone();
        Box::pin(async move { client_clone.request_file(p, name_clone).await })
    });

    // Get the first successful response
    let file_content = futures::future::select_ok(requests)
        .await
        .map_err(|_| "No provider returned the file.".to_string())?
        .0;

    Ok(file_content)
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![start, provide_file, get_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
