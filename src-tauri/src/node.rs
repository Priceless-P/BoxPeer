use libp2p::{identity, Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};

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

impl NodeType {
    pub fn can_provide(&self) -> bool {
        matches!(self, NodeType::Provider | NodeType::Distributor)
    }
    pub fn can_distribute(&self) -> bool {
        matches!(self, NodeType::Distributor)
    }
    pub fn can_consume(&self) -> bool {
        true
    }
    fn to_string(&self) -> String {
        match self {
            NodeType::Provider => "Provider".to_string(),
            NodeType::Distributor => "Distributor".to_string(),
            NodeType::Consumer => "Consumer".to_string(),
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        match s {
            "Provider" => Some(NodeType::Provider),
            "Distributor" => Some(NodeType::Distributor),
            "Consumer" => Some(NodeType::Consumer),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub listening_addr: String,
    pub node_type: Option<NodeType>,
}

pub(crate) fn load_or_generate_keypair() -> identity::Keypair {
    if let Ok(mut file) = File::open("peer_keypair.bin") {
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
            .open("peer_keypair.bin")
            .unwrap();
        let encoded = keypair.to_protobuf_encoding().unwrap();
        let _ = file.write_all(&encoded);
        keypair
    }
}
pub(crate) fn save_node_type(node_type: NodeType) -> io::Result<()> {
    let mut peer_info = load_peer_info().unwrap_or_else(|| PeerInfo {
        peer_id: String::new(),
        listening_addr: String::new(),
        node_type: None,
    });

    // Update the node_type with the new value
    peer_info.node_type = Some(node_type);

    let file_content = serde_json::to_string(&peer_info)?;
    let mut file = File::create("peer_info.json")?;
    file.write_all(file_content.as_bytes())?;

    Ok(())
}
pub(crate) fn save_peer_info(peer_id: &str, listening_addr: &str) -> io::Result<()> {
    let mut peer_info = load_peer_info().unwrap_or(PeerInfo {
        peer_id: peer_id.to_string(),
        listening_addr: listening_addr.to_string(),
        node_type: None, // Default value
    });

    peer_info.peer_id = peer_id.to_string();
    peer_info.listening_addr = listening_addr.to_string();

    let file_content = serde_json::to_string(&peer_info)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("peer_info.json")?;
    file.write_all(file_content.as_bytes())?;

    Ok(())
}

pub(crate) fn load_peer_info() -> Option<PeerInfo> {
    if let Ok(mut file) = File::open("peer_info.json") {
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        serde_json::from_str(&contents).ok()
    } else {
        None
    }
}

async fn set_node_type(node_type: NodeType) -> Result<(), String> {
    let mut file = File::open("peer_info.json").map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    let mut peer_info: PeerInfo = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    peer_info.node_type = Some(node_type);

    let updated_content = serde_json::to_string(&peer_info).map_err(|e| e.to_string())?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("peer_info.json")
        .map_err(|e| e.to_string())?;
    file.write_all(updated_content.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn get_node_type() -> Result<NodeType, String> {
    let mut file = File::open("peer_info.json").map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    let peer_info: PeerInfo = serde_json::from_str(&contents).map_err(|e| e.to_string())?;

    match peer_info.node_type {
        Some(node_type) => Ok(node_type),
        None => Err("Node type not set".to_string()),
    }
}
