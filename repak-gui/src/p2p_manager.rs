//! P2P Manager - Integration layer between libp2p and file sharing
//!
//! This module bridges the libp2p network layer with the existing
//! P2P file sharing implementation, enabling internet-wide transfers.

use crate::p2p_libp2p::{P2PNetwork, P2PNetworkEvent, ShareInfo};
use crate::p2p_sharing::{ShareableModPack, ShareSession, TransferProgress, P2PError, P2PResult};
use crate::p2p_security::MerkleTree;
use libp2p::{Multiaddr, PeerId};
use log::info;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use sha2::{Digest, Sha256};
use base64::Engine;

// ============================================================================
// UNIFIED P2P MANAGER
// ============================================================================

/// Manages both local and internet-wide P2P sharing
pub struct UnifiedP2PManager {
    /// libp2p network for internet-wide connectivity
    network: Arc<Mutex<P2PNetwork>>,
    /// Network event loop handle
    network_task: Option<JoinHandle<()>>,
    /// Active share sessions
    active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
    /// Active downloads
    active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
    /// Event sender for network events
    event_tx: mpsc::UnboundedSender<P2PManagerEvent>,
    /// Event receiver for network events
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<P2PManagerEvent>>>,
}

/// Active share session
struct ActiveShare {
    session: ShareSession,
    mod_pack: ShareableModPack,
    mod_paths: Vec<PathBuf>,
    peer_id: PeerId,
}

/// Active download session
struct ActiveDownload {
    share_info: ShareInfo,
    progress: TransferProgress,
    output_dir: PathBuf,
}

/// Events from the P2P manager
#[derive(Debug, Clone)]
pub enum P2PManagerEvent {
    /// Network is ready
    NetworkReady,
    /// Share is now advertised and discoverable
    ShareAdvertised(String),
    /// Found a peer for a share code
    SharePeerDiscovered { share_code: String, peer_id: PeerId },
    /// Connected to a peer
    PeerConnected(PeerId),
    /// Transfer progress update
    TransferProgress { share_code: String, progress: TransferProgress },
    /// Transfer completed
    TransferComplete { share_code: String },
    /// Error occurred
    Error(String),
}

impl UnifiedP2PManager {
    /// Create a new unified P2P manager
    pub async fn new() -> P2PResult<Self> {
        let network = P2PNetwork::new()
            .await
            .map_err(|e| P2PError::NetworkError(format!("Failed to create network: {}", e)))?;

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            network: Arc::new(Mutex::new(network)),
            network_task: None,
            active_shares: Arc::new(Mutex::new(HashMap::new())),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
        })
    }

    /// Start the P2P network
    pub async fn start(&mut self) -> P2PResult<()> {
        // Start listening
        {
            let mut network = self.network.lock().unwrap();
            network.start_listening()
                .map_err(|e| P2PError::NetworkError(format!("Failed to start listening: {}", e)))?;
        }

        // Start network event loop
        let network = Arc::clone(&self.network);
        let event_tx = self.event_tx.clone();
        
        self.network_task = Some(tokio::spawn(async move {
            Self::network_event_loop(network, event_tx).await;
        }));

        // Bootstrap the DHT
        {
            let mut network = self.network.lock().unwrap();
            network.bootstrap()
                .map_err(|e| P2PError::NetworkError(format!("Failed to bootstrap: {}", e)))?;
        }

        info!("P2P network started");
        Ok(())
    }

    /// Network event loop
    async fn network_event_loop(
        network: Arc<Mutex<P2PNetwork>>,
        event_tx: mpsc::UnboundedSender<P2PManagerEvent>,
    ) {
        loop {
            let event = {
                let mut net = network.lock().unwrap();
                // This is a blocking operation, so we need to be careful
                // In a real implementation, you'd want to use proper async handling
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(net.next_event())
                })
            };

            if let Some(event) = event {
                match event {
                    P2PNetworkEvent::ListeningOn(addr) => {
                        info!("Listening on: {}", addr);
                    }
                    P2PNetworkEvent::PeerConnected(peer_id) => {
                        let _ = event_tx.send(P2PManagerEvent::PeerConnected(peer_id));
                    }
                    P2PNetworkEvent::SharePeerFound(peer_id) => {
                        info!("Found peer for share: {}", peer_id);
                        // This would need the share code context
                    }
                    P2PNetworkEvent::HolePunchingSuccess(peer_id) => {
                        info!("Hole punching successful with: {}", peer_id);
                    }
                    P2PNetworkEvent::NatStatusChanged(status) => {
                        info!("NAT status: {:?}", status);
                    }
                    P2PNetworkEvent::RelayReservationSuccess(relay_peer_id) => {
                        info!("Relay reservation with: {}", relay_peer_id);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Start sharing a mod pack (internet-wide)
    pub async fn start_sharing(
        &mut self,
        name: String,
        description: String,
        mod_paths: Vec<PathBuf>,
        creator: Option<String>,
    ) -> P2PResult<ShareInfo> {
        // Create the mod pack
        let mod_pack = crate::p2p_sharing::create_mod_pack(
            name,
            description,
            &mod_paths,
            creator,
        )?;

        // Generate share code
        let share_code = crate::p2p_sharing::generate_share_code();
        let encryption_key = crate::p2p_sharing::generate_encryption_key();
        let encryption_key_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&encryption_key);

        // Get our peer ID and addresses
        let (peer_id, addresses) = {
            let network = self.network.lock().unwrap();
            let peer_id = network.local_peer_id();
            let mut addrs = network.listening_addresses();
            addrs.extend(network.external_addresses());
            (peer_id, addrs)
        };

        // Create share info
        let share_info = ShareInfo {
            peer_id: peer_id.to_string(),
            addresses: addresses.iter().map(|a| a.to_string()).collect(),
            encryption_key: encryption_key_b64.clone(),
            share_code: share_code.clone(),
        };

        // Advertise in DHT
        {
            let mut network = self.network.lock().unwrap();
            network.advertise_share(&share_code)
                .map_err(|e| P2PError::NetworkError(format!("Failed to advertise: {}", e)))?;
        }

        // Store active share
        let peer_id_str = peer_id.to_string();
        let encoded_share_info = share_info.encode()
            .map_err(|e| P2PError::ValidationError(format!("Failed to encode: {}", e)))?;
        let session = ShareSession {
            share_code: share_code.clone(),
            encryption_key: encryption_key_b64.clone(),
            local_ip: peer_id_str.clone(), // Use peer ID instead of IP
            obfuscated_ip: format!("Peer [{}]", &peer_id_str[..8]), // Show first 8 chars of peer ID
            port: 0, // Not used in libp2p
            connection_string: encoded_share_info.clone(),
            obfuscated_connection_string: format!("{}:{}:Peer[{}]:libp2p", 
                share_code, 
                encryption_key_b64,
                &peer_id_str[..8]
            ),
            active: true,
        };

        self.active_shares.lock().unwrap().insert(
            share_code.clone(),
            ActiveShare {
                session: session.clone(),
                mod_pack: mod_pack.clone(),
                mod_paths,
                peer_id,
            },
        );

        info!("Started sharing: {} (peer: {})", share_code, peer_id);
        let _ = self.event_tx.send(P2PManagerEvent::ShareAdvertised(share_code));

        Ok(share_info)
    }

    /// Stop sharing a mod pack
    pub fn stop_sharing(&mut self, share_code: &str) -> P2PResult<()> {
        self.active_shares.lock().unwrap().remove(share_code);
        info!("Stopped sharing: {}", share_code);
        Ok(())
    }

    /// Start downloading a mod pack from a share code
    pub async fn start_receiving(
        &mut self,
        connection_string: &str,
        output_dir: PathBuf,
        client_name: Option<String>,
    ) -> P2PResult<()> {
        // Decode share info
        let share_info = ShareInfo::decode(connection_string)
            .map_err(|e| P2PError::ValidationError(format!("Invalid connection string: {}", e)))?;

        // Search for peer in DHT
        {
            let mut network = self.network.lock().unwrap();
            network.find_peer_by_share_code(&share_info.share_code);
        }

        // Store download info
        self.active_downloads.lock().unwrap().insert(
            share_info.share_code.clone(),
            ActiveDownload {
                share_info: share_info.clone(),
                progress: TransferProgress {
                    current_file: String::new(),
                    files_completed: 0,
                    total_files: 0,
                    bytes_transferred: 0,
                    total_bytes: 0,
                    status: crate::p2p_sharing::TransferStatus::Connecting,
                },
                output_dir,
            },
        );

        info!("Started receiving: {}", share_info.share_code);
        Ok(())
    }

    /// Get current share session
    pub fn get_share_session(&self, share_code: &str) -> Option<ShareSession> {
        self.active_shares
            .lock()
            .unwrap()
            .get(share_code)
            .map(|s| s.session.clone())
    }

    /// Get transfer progress
    pub fn get_transfer_progress(&self, share_code: &str) -> Option<TransferProgress> {
        self.active_downloads
            .lock()
            .unwrap()
            .get(share_code)
            .map(|d| d.progress.clone())
    }

    /// Check if currently sharing
    pub fn is_sharing(&self, share_code: &str) -> bool {
        self.active_shares.lock().unwrap().contains_key(share_code)
    }

    /// Check if currently receiving
    pub fn is_receiving(&self, share_code: &str) -> bool {
        self.active_downloads.lock().unwrap().contains_key(share_code)
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> String {
        self.network.lock().unwrap().local_peer_id().to_string()
    }

    /// Get listening addresses
    pub fn listening_addresses(&self) -> Vec<String> {
        self.network
            .lock()
            .unwrap()
            .listening_addresses()
            .iter()
            .map(|a| a.to_string())
            .collect()
    }

    /// Connect to a relay server
    pub fn connect_to_relay(&mut self, relay_addr: &str) -> P2PResult<()> {
        let addr: Multiaddr = relay_addr.parse()
            .map_err(|e| P2PError::ValidationError(format!("Invalid relay address: {}", e)))?;
        
        self.network.lock().unwrap().connect_to_relay(addr)
            .map_err(|e| P2PError::NetworkError(format!("Failed to connect to relay: {}", e)))?;
        
        Ok(())
    }

    // ========================================================================
    // FILE TRANSFER IMPLEMENTATION
    // ========================================================================

    /// Handle file transfer to a peer
    pub async fn handle_file_transfer(
        &self,
        peer_id: PeerId,
        share_code: &str,
    ) -> P2PResult<()> {
        // Get share info
        let (mod_pack, mod_paths) = {
            let shares = self.active_shares.lock().unwrap();
            let share = shares.get(share_code)
                .ok_or_else(|| P2PError::ValidationError("Share not found".to_string()))?;
            (share.mod_pack.clone(), share.mod_paths.clone())
        };

        info!("Starting file transfer to peer: {}", peer_id);

        // Send each file
        for (mod_info, mod_path) in mod_pack.mods.iter().zip(mod_paths.iter()) {
            self.send_file_to_peer(peer_id, &mod_info.filename, mod_path, &mod_info.hash).await?;
        }

        info!("File transfer complete to peer: {}", peer_id);
        Ok(())
    }

    /// Send a single file to a peer
    async fn send_file_to_peer(
        &self,
        peer_id: PeerId,
        filename: &str,
        path: &Path,
        expected_hash: &str,
    ) -> P2PResult<()> {
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

        // Open file
        let file = File::open(path)
            .map_err(|e| P2PError::FileError(format!("Failed to open file: {}", e)))?;
        let file_size = file.metadata()
            .map_err(|e| P2PError::FileError(format!("Failed to get metadata: {}", e)))?
            .len();

        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut offset = 0u64;
        let mut hasher = Sha256::new();

        info!("Sending file: {} ({} bytes) to {}", filename, file_size, peer_id);

        // Send chunks
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| P2PError::FileError(format!("Failed to read file: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            hasher.update(chunk);

            // Calculate chunk hash for Merkle tree
            let mut chunk_hasher = Sha256::new();
            chunk_hasher.update(chunk);
            let chunk_hash = hex::encode(chunk_hasher.finalize());

            let is_last = offset + bytes_read as u64 >= file_size;

            // Send chunk via libp2p
            // TODO: Implement actual libp2p request/response
            // For now, this is a placeholder
            info!("Sent chunk at offset {} ({} bytes)", offset, bytes_read);

            offset += bytes_read as u64;

            if is_last {
                break;
            }
        }

        // Verify hash
        let computed_hash = hex::encode(hasher.finalize());
        if computed_hash != expected_hash {
            return Err(P2PError::ValidationError(format!(
                "Hash mismatch for {}: expected {}, got {}",
                filename, expected_hash, computed_hash
            )));
        }

        info!("File sent successfully: {}", filename);
        Ok(())
    }

    /// Receive a file from a peer
    pub async fn receive_file_from_peer(
        &self,
        peer_id: PeerId,
        filename: &str,
        expected_hash: &str,
        output_dir: &Path,
    ) -> P2PResult<u64> {
        info!("Receiving file: {} from {}", filename, peer_id);

        let output_path = output_dir.join(filename);
        let file = File::create(&output_path)
            .map_err(|e| P2PError::FileError(format!("Failed to create file: {}", e)))?;
        let mut writer = BufWriter::new(file);
        let mut hasher = Sha256::new();
        let mut total_received = 0u64;

        // Receive chunks
        // TODO: Implement actual libp2p request/response
        // For now, this is a placeholder
        loop {
            // Request next chunk
            // Receive chunk
            // Verify chunk hash
            // Write to file
            
            // Placeholder: break after first iteration
            break;
        }

        writer.flush()
            .map_err(|e| P2PError::FileError(format!("Failed to flush: {}", e)))?;

        // Verify final hash
        let computed_hash = hex::encode(hasher.finalize());
        if computed_hash != expected_hash {
            // Delete corrupted file
            let _ = std::fs::remove_file(&output_path);
            return Err(P2PError::ValidationError(format!(
                "Hash mismatch for {}: expected {}, got {}",
                filename, expected_hash, computed_hash
            )));
        }

        info!("File received successfully: {} ({} bytes)", filename, total_received);
        Ok(total_received)
    }

    /// Create Merkle tree for a file
    pub fn create_merkle_tree(&self, path: &Path) -> P2PResult<MerkleTree> {
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
        MerkleTree::from_file(path, CHUNK_SIZE)
            .map_err(|e| P2PError::FileError(format!("Failed to create Merkle tree: {}", e)))
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Validate a libp2p-based connection string
pub fn validate_libp2p_connection_string(conn_str: &str) -> P2PResult<bool> {
    match ShareInfo::decode(conn_str) {
        Ok(_) => Ok(true),
        Err(e) => Err(P2PError::ValidationError(format!("Invalid connection string: {}", e))),
    }
}
