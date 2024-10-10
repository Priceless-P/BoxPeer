use std::path::Path;
use libp2p::identity;
use libp2p_webrtc::tokio::Certificate;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use tauri::api::dir;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
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

    if let Ok(mut file) = File::open(&file_path) {
        let mut contents = Vec::new();
        let _ = file.read_to_end(&mut contents);
        println!("In If");
        identity::Keypair::from_protobuf_encoding(&contents).unwrap()
    } else {
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
    }
}

pub async fn boxpeer_dir() -> Result<String, String> {
    match cache_dir() {
        Some(cache_path) => {
            let mut dir = PathBuf::from(cache_path);
            dir.push("Boxpeer");
            // Convert PathBuf to String and return it
            dir.to_str()
                .map(|s| s.to_string())
                .ok_or("Failed to convert PathBuf to String".to_string())
        }
        None => Err("No cache directory found".to_string()),
    }
}

async fn generate_and_save_certificate() -> std::io::Result<Certificate> {
 let dir = boxpeer_dir().await.expect("Error");
 let mut cert_path = PathBuf::from(dir);
    cert_path.push("cert.pem");
    let cert = Certificate::generate(&mut thread_rng()).unwrap();
    let pem_str = cert.serialize_pem();
    let mut file = File::create(cert_path)?;
    file.write_all(&pem_str.as_bytes())?;

    Ok(cert)
}

pub async fn load_or_generate_certificate() -> std::io::Result<Certificate> {
    let dir = boxpeer_dir().await.expect("Error");
    let mut cert_path = PathBuf::from(dir);
    cert_path.push("cert.pem");

    if cert_path.exists() {
        let mut file = File::open(cert_path)?;
        let mut pem_str = String::new();
        file.read_to_string(&mut pem_str)?;

        // Recreate the certificate from the DER format
        Certificate::from_pem(&pem_str).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    } else {
        generate_and_save_certificate().await
    }
}

