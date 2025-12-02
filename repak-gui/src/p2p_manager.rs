//! P2P Manager - Integration layer between libp2p and file sharing
//!
//! This module bridges the libp2p network layer with the existing
//! P2P file sharing implementation, enabling internet-wide transfers.

use crate::p2p_libp2p::{P2PNetwork, P2PNetworkEvent, ShareInfo};
use crate::p2p_protocol::{FileTransferRequest, FileTransferResponse};
use crate::p2p_sharing::{ShareableModPack, ShareSession, TransferProgress, P2PError, P2PResult};
use crate::p2p_security::MerkleTree;
use libp2p::{Multiaddr, PeerId};
use libp2p::request_response as req_resp;
use log::{info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Seek};
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
    pub active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
    /// Active downloads
    pub active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
    /// Event sender for network events
    event_tx: mpsc::UnboundedSender<P2PManagerEvent>,
    /// Event receiver for network events
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<P2PManagerEvent>>>,
}

/// Active share session
pub struct ActiveShare {
    pub session: ShareSession,
    pub mod_pack: ShareableModPack,
    pub mod_paths: Vec<PathBuf>,
    pub peer_id: PeerId,
}

/// Active download session
pub struct ActiveDownload {
    pub share_info: ShareInfo,
    pub progress: TransferProgress,
    pub output_dir: PathBuf,
}

/// Events from the P2P manager
#[derive(Debug)]
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
    /// File transfer response received
    FileTransferResponse(PeerId, FileTransferResponse),
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
            let mut network = self.network.lock();
            network.start_listening()
                .map_err(|e| P2PError::NetworkError(format!("Failed to start listening: {}", e)))?;
        }

        // Start network event loop
        let network = Arc::clone(&self.network);
        let event_tx = self.event_tx.clone();
        let active_shares = Arc::clone(&self.active_shares);
        let active_downloads = Arc::clone(&self.active_downloads);
        
        self.network_task = Some(tokio::spawn(async move {
            Self::network_event_loop(network, event_tx, active_shares, active_downloads).await;
        }));

        // Bootstrap the DHT
        {
            let mut network = self.network.lock();
            network.bootstrap()
                .map_err(|e| P2PError::NetworkError(format!("Failed to bootstrap: {}", e)))?;
            
            // Connect to default relays
            let relays = network.relay_addresses();
            for relay in relays {
                info!("Connecting to default relay: {}", relay);
                let _ = network.connect_to_relay(relay);
            }
        }

        info!("P2P network started");
        Ok(())
    }

    /// Network event loop
    async fn network_event_loop(
        network: Arc<Mutex<P2PNetwork>>,
        event_tx: mpsc::UnboundedSender<P2PManagerEvent>,
        active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
        active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
    ) {
        loop {
            let event = {
                let mut net = network.lock();
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
                        info!("Peer connected: {}", peer_id);
                        let _ = event_tx.send(P2PManagerEvent::PeerConnected(peer_id));
                        
                        // Check if we're waiting to download from this peer
                        let downloads = active_downloads.lock();
                        let should_request = downloads.iter().any(|(_, d)| {
                            d.share_info.peer_id.parse::<PeerId>().ok() == Some(peer_id)
                        });
                        drop(downloads);
                        
                        if should_request {
                            info!("Requesting pack info from newly connected peer {}", peer_id);
                            let mut net = network.lock();
                            net.request_pack_info(peer_id);
                        }
                    }
                    P2PNetworkEvent::SharePeerFound(peer_id) => {
                        info!("Found peer for share: {}", peer_id);
                        
                        // Check if we are waiting for this peer in any active download
                        let downloads = active_downloads.lock();
                        let should_connect = downloads.iter().any(|(_, d)| {
                            d.share_info.peer_id.parse::<PeerId>().ok() == Some(peer_id)
                        });
                        drop(downloads);

                        if should_connect {
                            info!("Found target peer {}, initiating connection", peer_id);
                            let mut net = network.lock();
                            if let Err(e) = net.dial_peer_id(peer_id) {
                                warn!("Failed to dial peer {}: {}", peer_id, e);
                            }
                        }
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
                    P2PNetworkEvent::FileTransferRequest { peer, request, channel } => {
                        info!("File transfer request from {}: {:?}", peer, request);
                        // Handle request in a separate task to avoid blocking
                        let network_clone = Arc::clone(&network);
                        let shares_clone = Arc::clone(&active_shares);
                        tokio::spawn(async move {
                            Self::handle_file_transfer_request(network_clone, peer, request, channel, shares_clone).await;
                        });
                    }
                    P2PNetworkEvent::FileTransferResponse { peer, response } => {
                        info!("File transfer response from {}: {:?}", peer, response);
                        // Handle response inline
                        let network_clone = Arc::clone(&network);
                        let downloads_clone = Arc::clone(&active_downloads);
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_file_response_static(network_clone, downloads_clone, peer, response).await {
                                warn!("Failed to handle file response: {}", e);
                            }
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handle incoming file transfer requests
    async fn handle_file_transfer_request(
        network: Arc<Mutex<P2PNetwork>>,
        peer: PeerId,
        request: FileTransferRequest,
        channel: req_resp::ResponseChannel<FileTransferResponse>,
        active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
    ) {
        let response = match request {
            FileTransferRequest::GetPackInfo => {
                // Find the share for this peer
                let shares = active_shares.lock();
                if let Some((_share_code, share)) = shares.iter().find(|(_, s)| s.peer_id == peer) {
                    // Serialize the mod pack
                    match bincode::serialize(&share.mod_pack) {
                        Ok(pack_data) => {
                            info!("Sending pack info to {}: {} files", peer, share.mod_pack.mods.len());
                            FileTransferResponse::PackInfo { pack_data }
                        }
                        Err(e) => {
                            warn!("Failed to serialize pack info: {}", e);
                            FileTransferResponse::Error {
                                message: format!("Serialization error: {}", e),
                            }
                        }
                    }
                } else {
                    FileTransferResponse::Error {
                        message: "No active share found for this peer".to_string(),
                    }
                }
            }
            FileTransferRequest::GetChunk { filename, offset, size } => {
                // Find the share and file
                let shares = active_shares.lock();
                if let Some((_share_code, share)) = shares.iter().find(|(_, s)| s.peer_id == peer) {
                    // Find the file in the mod pack
                    if let Some((mod_info, mod_path)) = share.mod_pack.mods.iter()
                        .zip(share.mod_paths.iter())
                        .find(|(info, _)| info.filename == filename)
                    {
                        // Read the chunk from file
                        match Self::read_file_chunk(mod_path, offset, size) {
                            Ok((data, is_last)) => {
                                // Calculate chunk hash
                                let mut hasher = Sha256::new();
                                hasher.update(&data);
                                let hash = hex::encode(hasher.finalize());
                                
                                info!("Sending chunk of {} to {}: offset={}, size={}, is_last={}", 
                                    filename, peer, offset, data.len(), is_last);
                                
                                FileTransferResponse::FileChunk {
                                    filename: filename.clone(),
                                    offset,
                                    data,
                                    is_last,
                                    hash,
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read file chunk: {}", e);
                                FileTransferResponse::Error {
                                    message: format!("File read error: {}", e),
                                }
                            }
                        }
                    } else {
                        FileTransferResponse::Error {
                            message: format!("File not found: {}", filename),
                        }
                    }
                } else {
                    FileTransferResponse::Error {
                        message: "No active share found for this peer".to_string(),
                    }
                }
            }
            FileTransferRequest::Ping => {
                FileTransferResponse::Pong
            }
            _ => {
                FileTransferResponse::Error {
                    message: "Unknown request".to_string(),
                }
            }
        };

        let mut net = network.lock();
        if let Err(e) = net.send_response(channel, response) {
            warn!("Failed to send response to {}: {:?}", peer, e);
        }
    }

    /// Read a chunk from a file
    fn read_file_chunk(path: &Path, offset: u64, size: usize) -> std::io::Result<(Vec<u8>, bool)> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        
        let mut reader = BufReader::new(file);
        reader.seek(std::io::SeekFrom::Start(offset))?;
        
        let mut buffer = vec![0u8; size];
        let bytes_read = reader.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        
        let is_last = offset + bytes_read as u64 >= file_size;
        
        Ok((buffer, is_last))
    }

    /// Handle file transfer response (receiving files) - static version for event loop
    async fn handle_file_response_static(
        network: Arc<Mutex<P2PNetwork>>,
        active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
        peer: PeerId,
        response: FileTransferResponse,
    ) -> P2PResult<()> {
        match response {
            FileTransferResponse::PackInfo { pack_data } => {
                // Deserialize the mod pack
                let mod_pack: ShareableModPack = bincode::deserialize(&pack_data)
                    .map_err(|e| P2PError::ValidationError(format!("Failed to deserialize pack: {}", e)))?;
                
                let total_size: u64 = mod_pack.mods.iter().map(|m| m.size).sum();
                info!("Received pack info from {}: {} files, total size: {} bytes", 
                    peer, mod_pack.mods.len(), total_size);
                
                // Find the download session
                let downloads = active_downloads.lock();
                if let Some((_share_code, download)) = downloads.iter().find(|(_, d)| {
                    d.share_info.peer_id.parse::<PeerId>().ok() == Some(peer)
                }) {
                    // Start requesting files
                    let output_dir = download.output_dir.clone();
                    drop(downloads);
                    
                    // Request first file
                    if let Some(first_file) = mod_pack.mods.first() {
                        let mut net = network.lock();
                        net.request_file_chunk(peer, first_file.filename.clone(), 0, 1024 * 1024); // 1MB chunks
                        info!("Requesting first chunk of {}", first_file.filename);
                    }
                }
                
                Ok(())
            }
            FileTransferResponse::FileChunk { filename, offset, data, is_last, hash } => {
                // Verify chunk hash
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let computed_hash = hex::encode(hasher.finalize());
                
                if computed_hash != hash {
                    return Err(P2PError::ValidationError(format!(
                        "Chunk hash mismatch for {}: expected {}, got {}",
                        filename, hash, computed_hash
                    )));
                }
                
                info!("Received chunk of {}: offset={}, size={}, is_last={}", 
                    filename, offset, data.len(), is_last);
                
                // Find the download session
                let downloads = active_downloads.lock();
                if let Some((_share_code, download)) = downloads.iter().find(|(_, d)| {
                    d.share_info.peer_id.parse::<PeerId>().ok() == Some(peer)
                }) {
                    let output_dir = download.output_dir.clone();
                    drop(downloads);
                    
                    // Write chunk to file
                    let file_path = output_dir.join(&filename);
                    let file = if offset == 0 {
                        File::create(&file_path)
                    } else {
                        std::fs::OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(&file_path)
                    }.map_err(|e| P2PError::FileError(format!("Failed to open file: {}", e)))?;
                    
                    let mut writer = BufWriter::new(file);
                    writer.write_all(&data)
                        .map_err(|e| P2PError::FileError(format!("Failed to write chunk: {}", e)))?;
                    writer.flush()
                        .map_err(|e| P2PError::FileError(format!("Failed to flush: {}", e)))?;
                    
                    // Request next chunk if not last
                    if !is_last {
                        let next_offset = offset + data.len() as u64;
                        let mut net = network.lock();
                        net.request_file_chunk(peer, filename.clone(), next_offset, 1024 * 1024);
                        info!("Requesting next chunk of {} at offset {}", filename, next_offset);
                    } else {
                        info!("File complete: {}", filename);
                        // TODO: Request next file in the pack
                    }
                }
                
                Ok(())
            }
            FileTransferResponse::Error { message } => {
                warn!("File transfer error from {}: {}", peer, message);
                Err(P2PError::NetworkError(message))
            }
            FileTransferResponse::Pong => {
                info!("Pong from {}", peer);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Start sharing a mod pack (internet-wide)
    pub async fn start_sharing(
        &self,
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
            let network = self.network.lock();
            let peer_id = network.local_peer_id();
            let listening = network.listening_addresses();
            let external = network.external_addresses();
            let relay = network.relay_addresses();
            info!("Listening addresses: {:?}", listening);
            info!("External addresses: {:?}", external);
            info!("Relay addresses: {:?}", relay);
            
            // Use all_addresses() to get listening + external + relay circuit addresses
            let addrs = network.all_addresses();
            (peer_id, addrs)
        };

        info!("Total addresses for share: {}", addresses.len());
        for (i, addr) in addresses.iter().enumerate() {
            info!("  Address {}: {}", i+1, addr);
        }
        
        if addresses.is_empty() {
            warn!("No addresses available! Network may not be fully initialized.");
            warn!("Consider waiting a few seconds after starting the network before creating shares.");
            warn!("Peers will still be able to connect via DHT discovery.");
        }

        // Create share info
        let share_info = ShareInfo {
            peer_id: peer_id.to_string(),
            addresses: addresses.iter().map(|a| a.to_string()).collect(),
            encryption_key: encryption_key_b64.clone(),
            share_code: share_code.clone(),
        };
        
        info!("Created ShareInfo with {} addresses", share_info.addresses.len());

        // Advertise in DHT
        {
            let mut network = self.network.lock();
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

        self.active_shares.lock().insert(
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
    pub fn stop_sharing(&self, share_code: &str) -> P2PResult<()> {
        self.active_shares.lock().remove(share_code);
        info!("Stopped sharing: {}", share_code);
        Ok(())
    }

    /// Start downloading a mod pack from a share code
    pub async fn start_receiving(
        &self,
        connection_string: &str,
        output_dir: PathBuf,
        client_name: Option<String>,
    ) -> P2PResult<()> {
        // Decode share info
        let share_info = ShareInfo::decode(connection_string)
            .map_err(|e| P2PError::ValidationError(format!("Invalid connection string: {}", e)))?;

        info!("Decoded share info: peer_id={}, addresses={:?}, share_code={}", 
            share_info.peer_id, share_info.addresses, share_info.share_code);

        // Search for peer in DHT
        {
            let mut network = self.network.lock();
            network.find_peer_by_share_code(&share_info.share_code);
        }

        // Store download info
        self.active_downloads.lock().insert(
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

        // Try to connect to peer directly if we have addresses
        info!("Attempting to connect to peer...");
        if let Ok(peer_id) = share_info.peer_id.parse::<PeerId>() {
            info!("Parsed peer ID: {}", peer_id);
            
            if share_info.addresses.is_empty() {
                warn!("No addresses provided in connection string!");
                info!("Will rely on DHT discovery to find peer");
                // The DHT search initiated above (line 536) will help discover the peer
                // When the peer is found via DHT, the PeerDiscovered event will trigger connection
            } else {
                info!("Trying {} addresses", share_info.addresses.len());
                
                // Attempt to dial the peer
                let mut connected = false;
                for (i, addr_str) in share_info.addresses.iter().enumerate() {
                    info!("Trying address {}/{}: {}", i+1, share_info.addresses.len(), addr_str);
                    if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                        let mut network = self.network.lock();
                        if let Err(e) = network.dial_peer(peer_id, addr.clone()) {
                            warn!("Failed to dial peer {} at {}: {}", peer_id, addr, e);
                        } else {
                            info!("Successfully initiated dial to peer {} at {}", peer_id, addr);
                            connected = true;
                            // Don't request pack info yet - wait for PeerConnected event
                            break;
                        }
                    } else {
                        warn!("Failed to parse address: {}", addr_str);
                    }
                }
                
                if !connected {
                    info!("Failed to connect via provided addresses, falling back to DHT discovery");
                }
            }
        } else {
            warn!("Failed to parse peer ID: {}", share_info.peer_id);
        }

        info!("Started receiving: {}", share_info.share_code);
        Ok(())
    }

    /// Get share session
    pub fn get_share_session(&self, share_code: &str) -> Option<ShareSession> {
        self.active_shares
            .lock()
            .get(share_code)
            .map(|s| s.session.clone())
    }

    /// Get transfer progress
    pub fn get_transfer_progress(&self, share_code: &str) -> Option<TransferProgress> {
        self.active_downloads
            .lock()
            .get(share_code)
            .map(|d| d.progress.clone())
    }

    /// Check if currently sharing
    pub fn is_sharing(&self, share_code: &str) -> bool {
        self.active_shares.lock().contains_key(share_code)
    }

    /// Check if currently receiving
    pub fn is_receiving(&self, share_code: &str) -> bool {
        self.active_downloads.lock().contains_key(share_code)
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> String {
        self.network.lock().local_peer_id().to_string()
    }

    /// Get listening addresses
    pub fn listening_addresses(&self) -> Vec<String> {
        self.network
            .lock()
            .listening_addresses()
            .iter()
            .map(|a| a.to_string())
            .collect()
    }

    /// Connect to a relay server
    pub fn connect_to_relay(&mut self, relay_addr: &str) -> P2PResult<()> {
        let addr: Multiaddr = relay_addr.parse()
            .map_err(|e| P2PError::ValidationError(format!("Invalid relay address: {}", e)))?;
        
        self.network.lock().connect_to_relay(addr)
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
            let shares = self.active_shares.lock();
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
