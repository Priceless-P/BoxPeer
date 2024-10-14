use libp2p::identity;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tauri::api::path::cache_dir;

#[derive(Serialize, Deserialize, Clone)]
pub enum NodeType {
    Provider,
    Distributor,
    Consumer,
}

#[derive(Serialize, Deserialize, Clone)]
struct NodeInfo {
    node_type: NodeType,
}

#[derive(Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub listening_addr: String,
    pub node_type: Option<NodeType>,
}

pub(crate) fn load_or_generate_keypair() -> identity::Keypair {
    let cache_path = cache_dir().unwrap();
    let mut file_path = PathBuf::from(cache_path);
    file_path.push("Boxpeer");
    file_path.push("peer_keypair.bin".to_string());

    // if let Ok(mut file) = File::open(&file_path) {
    //     let mut contents = Vec::new();
    //     let _ = file.read_to_end(&mut contents);
    //     println!("In If");
    //     identity::Keypair::from_protobuf_encoding(&contents).unwrap()
    // } else {
    // Generate a new keypair if no file is found
    let keypair = identity::Keypair::generate_ed25519();
    println!("In else");
    // Save the keypair to disk
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_path)
        .unwrap();
    let encoded = keypair.to_protobuf_encoding().unwrap();
    let _ = file.write_all(&encoded);
    keypair
    //}
}

pub async fn boxpeer_dir() -> Result<String, String> {
    match cache_dir() {
        Some(cache_path) => {
            let mut dir = PathBuf::from(cache_path);
            dir.push("Boxpeer");
            dir.to_str()
                .map(|s| s.to_string())
                .ok_or("Failed to convert PathBuf to String".to_string())
        }
        None => Err("No cache directory found".to_string()),
    }
}
