//! P2P Manager - WebSocket relay-based secure P2P file sharing
//!
//! Architecture:
//! 1. Sharer connects to relay, joins room with share_code
//! 2. Receiver connects to same relay with share_code  
//! 3. All data is AES-256-GCM encrypted before transmission
//! 4. Relay only sees encrypted blobs - cannot read file contents
//! 5. No direct peer connections = No IP exposure = No doxxing

use crate::p2p_libp2p::ShareInfo;
use crate::p2p_relay::{self, RelayClient, RelayMessage, encrypt_data, decrypt_data, derive_key};
use crate::p2p_sharing::{ShareableModPack, ShareSession, TransferProgress, TransferStatus, P2PError, P2PResult};
use log::{info, warn, error, debug};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use base64::Engine;

// ============================================================================
// CONSTANTS  
// ============================================================================

/// Relay server URL (shown in logs for debugging)
const RELAY_URL: &str = "wss://free.blr2.piesocket.com/v3/repak-p2p-v2";

// ============================================================================
// UNIFIED P2P MANAGER
// ============================================================================

/// Manages P2P sharing via WebSocket relay
/// All communication goes through the relay - no direct IP exposure
pub struct UnifiedP2PManager {
    /// Unique ID for this instance  
    instance_id: String,
    /// Active share sessions (share_code -> ActiveShare)
    pub active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
    /// Active downloads (share_code -> ActiveDownload)
    pub active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
    /// Background task handles
    task_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Active share session
pub struct ActiveShare {
    pub session: ShareSession,
    pub mod_pack: ShareableModPack,
    pub mod_paths: Vec<PathBuf>,
    pub encryption_key: [u8; 32],
}

/// Active download session  
pub struct ActiveDownload {
    pub share_info: ShareInfo,
    pub progress: TransferProgress,
    pub output_dir: PathBuf,
    pub encryption_key: [u8; 32],
    /// Received chunks per file: filename -> (chunk_index -> decrypted_data)
    pub received_chunks: HashMap<String, HashMap<u32, Vec<u8>>>,
    /// File metadata: filename -> (total_chunks, hash)
    pub file_metadata: HashMap<String, (u32, String)>,
}

impl UnifiedP2PManager {
    /// Create a new P2P manager
    pub async fn new() -> P2PResult<Self> {
        // Generate a unique instance ID
        let instance_id = format!("repak-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        
        info!("[P2P Manager] Initialized with instance ID: {}", instance_id);
        info!("[P2P Manager] Relay server: {}", RELAY_URL);
        
        Ok(Self {
            instance_id,
            active_shares: Arc::new(Mutex::new(HashMap::new())),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            task_handles: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Start sharing a mod pack
    /// Returns ShareInfo that can be encoded and shared with others
    pub async fn start_sharing(
        &self,
        name: String,
        description: String,
        mod_paths: Vec<PathBuf>,
        creator: Option<String>,
    ) -> P2PResult<ShareInfo> {
        info!("[P2P Share] Starting share session...");
        info!("[P2P Share] Pack name: {}", name);
        info!("[P2P Share] Files to share: {}", mod_paths.len());
        
        // Create the mod pack
        let mod_pack = crate::p2p_sharing::create_mod_pack(
            name.clone(),
            description.clone(),
            &mod_paths,
            creator.clone(),
        )?;

        // Generate share code and encryption key
        let share_code = crate::p2p_sharing::generate_share_code();
        let encryption_key = crate::p2p_sharing::generate_encryption_key();
        let encryption_key_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&encryption_key);

        info!("[P2P Share] Generated share code: {}", share_code);
        info!("[P2P Share] Mod pack contains {} files", mod_pack.mods.len());
        for (i, m) in mod_pack.mods.iter().enumerate() {
            info!("[P2P Share]   File {}: {} ({} bytes)", i + 1, m.filename, m.size);
        }

        // Derive 32-byte key
        let key_32 = derive_key(&encryption_key_b64)
            .map_err(|e| P2PError::ValidationError(e))?;

        // Create share info - using instance_id as peer_id (no real IP exposed)
        let share_info = ShareInfo {
            peer_id: self.instance_id.clone(),
            addresses: vec![RELAY_URL.to_string()],
            encryption_key: encryption_key_b64.clone(),
            share_code: share_code.clone(),
        };

        // Create session
        let encoded_share_info = share_info.encode()
            .map_err(|e| P2PError::ValidationError(format!("Failed to encode share info: {}", e)))?;
        
        let session = ShareSession {
            share_code: share_code.clone(),
            encryption_key: encryption_key_b64.clone(),
            local_ip: "relay".to_string(),
            obfuscated_ip: "[Relay Protected]".to_string(),
            port: 0,
            connection_string: encoded_share_info.clone(),
            obfuscated_connection_string: format!("Share Code: {}", share_code),
            active: true,
        };

        // Store active share
        self.active_shares.lock().insert(
            share_code.clone(),
            ActiveShare {
                session,
                mod_pack: mod_pack.clone(),
                mod_paths: mod_paths.clone(),
                encryption_key: key_32,
            },
        );

        info!("[P2P Share] Share session created successfully");
        info!("[P2P Share] Connection string length: {} chars", encoded_share_info.len());

        // Start the sharer relay task
        let share_code_clone = share_code.clone();
        let instance_id = self.instance_id.clone();
        let active_shares = Arc::clone(&self.active_shares);
        
        let handle = tokio::spawn(async move {
            if let Err(e) = run_sharer_task(share_code_clone, instance_id, active_shares).await {
                error!("[P2P Share] Sharer task error: {}", e);
            }
        });
        
        self.task_handles.lock().push(handle);

        Ok(share_info)
    }

    /// Stop sharing a mod pack
    pub fn stop_sharing(&self, share_code: &str) -> P2PResult<()> {
        info!("[P2P Share] Stopping share: {}", share_code);
        self.active_shares.lock().remove(share_code);
        info!("[P2P Share] Share stopped");
        Ok(())
    }

    /// Start downloading a mod pack from a share code
    pub async fn start_receiving(
        &self,
        connection_string: &str,
        output_dir: PathBuf,
        _client_name: Option<String>,
    ) -> P2PResult<()> {
        info!("[P2P Receive] Starting download...");
        info!("[P2P Receive] Output directory: {}", output_dir.display());
        
        // Decode share info
        let share_info = ShareInfo::decode(connection_string)
            .map_err(|e| P2PError::ValidationError(format!("Invalid connection string: {}", e)))?;

        info!("[P2P Receive] Decoded share info:");
        info!("[P2P Receive]   Peer ID: {}", share_info.peer_id);
        info!("[P2P Receive]   Share code: {}", share_info.share_code);
        info!("[P2P Receive]   Addresses: {:?}", share_info.addresses);

        // Derive encryption key
        let key_32 = derive_key(&share_info.encryption_key)
            .map_err(|e| P2PError::ValidationError(e))?;
        
        info!("[P2P Receive] Encryption key derived successfully");

        // Store download info
        let share_code = share_info.share_code.clone();
        self.active_downloads.lock().insert(
            share_code.clone(),
            ActiveDownload {
                share_info: share_info.clone(),
                progress: TransferProgress {
                    current_file: String::new(),
                    files_completed: 0,
                    total_files: 0,
                    bytes_transferred: 0,
                    total_bytes: 0,
                    status: TransferStatus::Connecting,
                },
                output_dir: output_dir.clone(),
                encryption_key: key_32,
                received_chunks: HashMap::new(),
                file_metadata: HashMap::new(),
            },
        );

        info!("[P2P Receive] Download session created");
        info!("[P2P Receive] Connecting to relay server...");

        // Start the receiver relay task
        let instance_id = self.instance_id.clone();
        let active_downloads = Arc::clone(&self.active_downloads);
        let sharer_id = share_info.peer_id.clone();
        
        let handle = tokio::spawn(async move {
            if let Err(e) = run_receiver_task(share_code, instance_id, sharer_id, active_downloads).await {
                error!("[P2P Receive] Receiver task error: {}", e);
            }
        });
        
        self.task_handles.lock().push(handle);

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

    /// Get local instance ID (replaces peer_id)
    pub fn local_peer_id(&self) -> String {
        self.instance_id.clone()
    }

    /// Get relay addresses
    pub fn listening_addresses(&self) -> Vec<String> {
        vec![RELAY_URL.to_string()]
    }
}

// ============================================================================
// RELAY TASK FUNCTIONS
// ============================================================================

/// Run the sharer task - connects to relay and serves file requests
async fn run_sharer_task(
    share_code: String,
    instance_id: String,
    active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
) -> Result<(), String> {
    info!("[P2P Sharer] Starting sharer task for {}", share_code);
    
    // Connect to relay room (each share code gets its own channel)
    let room_name = format!("/repak-{}", share_code.replace("-", "").to_lowercase());
    let (client, mut rx) = RelayClient::connect_to_room(instance_id.clone(), &room_name).await?;
    
    // Announce presence as sharer
    client.join_room(&share_code, "sharer")?;
    info!("[P2P Sharer] Joined room: {}", share_code);
    
    // Main event loop
    loop {
        // Check if share is still active
        {
            let shares = active_shares.lock();
            if !shares.contains_key(&share_code) {
                info!("[P2P Sharer] Share no longer active, exiting");
                break;
            }
        }
        
        // Wait for messages with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx.recv()).await {
            Ok(Some(msg)) => {
                debug!("[P2P Sharer] Received message: {:?}", msg);
                
                match msg {
                    RelayMessage::RequestPackInfo { room, from } if room == share_code => {
                        info!("[P2P Sharer] Pack info requested by {}", from);
                        
                        // Get pack info and encrypt it
                        let shares = active_shares.lock();
                        if let Some(share) = shares.get(&share_code) {
                            let pack_json = serde_json::to_string(&share.mod_pack)
                                .map_err(|e| format!("Serialize error: {}", e))?;
                            
                            // Encrypt pack info
                            let encrypted = encrypt_data(pack_json.as_bytes(), &share.encryption_key)?;
                            let encrypted_b64 = base64::engine::general_purpose::STANDARD.encode(&encrypted);
                            
                            drop(shares);
                            
                            // Send encrypted pack info
                            client.send(RelayMessage::PackInfo {
                                room: share_code.clone(),
                                from: instance_id.clone(),
                                pack_json: encrypted_b64,
                            })?;
                            
                            info!("[P2P Sharer] Sent encrypted pack info");
                        }
                    }
                    RelayMessage::ChunkRequest { room, from, to, filename, chunk_index } 
                        if room == share_code && to == instance_id => {
                        info!("[P2P Sharer] Chunk {} of '{}' requested by {}", chunk_index, filename, from);
                        
                        // Find the file and read the chunk
                        let shares = active_shares.lock();
                        if let Some(share) = shares.get(&share_code) {
                            // Find the file path
                            let file_path = share.mod_pack.mods.iter()
                                .zip(share.mod_paths.iter())
                                .find(|(m, _)| m.filename == filename)
                                .map(|(_, p)| p.clone());
                            
                            let key = share.encryption_key;
                            let file_hash = share.mod_pack.mods.iter()
                                .find(|m| m.filename == filename)
                                .map(|m| m.hash.clone())
                                .unwrap_or_default();
                            
                            drop(shares);
                            
                            if let Some(path) = file_path {
                                // Read file chunks
                                match p2p_relay::read_file_chunks(&path) {
                                    Ok((chunks, hash)) => {
                                        let total_chunks = chunks.len() as u32;
                                        
                                        if (chunk_index as usize) < chunks.len() {
                                            let chunk_data = &chunks[chunk_index as usize];
                                            
                                            // Encrypt chunk
                                            let encrypted = encrypt_data(chunk_data, &key)?;
                                            let encrypted_b64 = base64::engine::general_purpose::STANDARD.encode(&encrypted);
                                            
                                            client.send(RelayMessage::ChunkData {
                                                room: share_code.clone(),
                                                from: instance_id.clone(),
                                                to: from.clone(),
                                                filename: filename.clone(),
                                                chunk_index,
                                                data: encrypted_b64,
                                                total_chunks,
                                                file_hash: hash,
                                            })?;
                                            
                                            info!("[P2P Sharer] Sent chunk {}/{} of '{}'", 
                                                chunk_index + 1, total_chunks, filename);
                                        } else {
                                            warn!("[P2P Sharer] Invalid chunk index: {}", chunk_index);
                                        }
                                    }
                                    Err(e) => {
                                        error!("[P2P Sharer] Failed to read file: {}", e);
                                        client.send(RelayMessage::Error {
                                            room: share_code.clone(),
                                            message: format!("File read error: {}", e),
                                        })?;
                                    }
                                }
                            } else {
                                warn!("[P2P Sharer] File not found: {}", filename);
                            }
                        }
                    }
                    RelayMessage::Ping { room, from } if room == share_code => {
                        debug!("[P2P Sharer] Ping from {}", from);
                    }
                    _ => {
                        debug!("[P2P Sharer] Ignoring message");
                    }
                }
            }
            Ok(None) => {
                info!("[P2P Sharer] Channel closed");
                break;
            }
            Err(_) => {
                // Timeout - send keepalive ping
                debug!("[P2P Sharer] Sending keepalive ping");
                let _ = client.send(RelayMessage::Ping {
                    room: share_code.clone(),
                    from: instance_id.clone(),
                });
            }
        }
    }
    
    info!("[P2P Sharer] Task ended for {}", share_code);
    Ok(())
}

/// Run the receiver task - connects to relay and downloads files
async fn run_receiver_task(
    share_code: String,
    instance_id: String,
    sharer_id: String,
    active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
) -> Result<(), String> {
    info!("[P2P Receiver] Starting receiver task for {}", share_code);
    info!("[P2P Receiver] Looking for sharer: {}", sharer_id);
    
    // Connect to relay room (must match sharer's room)
    let room_name = format!("/repak-{}", share_code.replace("-", "").to_lowercase());
    let (client, mut rx) = RelayClient::connect_to_room(instance_id.clone(), &room_name).await?;
    
    // Announce presence as receiver
    client.join_room(&share_code, "receiver")?;
    info!("[P2P Receiver] Joined room: {}", share_code);
    
    // Request pack info
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    client.send(RelayMessage::RequestPackInfo {
        room: share_code.clone(),
        from: instance_id.clone(),
    })?;
    info!("[P2P Receiver] Requested pack info");
    
    let mut mod_pack: Option<ShareableModPack> = None;
    let mut current_file_index = 0usize;
    let mut current_chunk_index = 0u32;
    
    // Main event loop
    loop {
        // Check if download is still active
        {
            let downloads = active_downloads.lock();
            if !downloads.contains_key(&share_code) {
                info!("[P2P Receiver] Download cancelled, exiting");
                break;
            }
        }
        
        // Wait for messages
        match tokio::time::timeout(std::time::Duration::from_secs(60), rx.recv()).await {
            Ok(Some(msg)) => {
                debug!("[P2P Receiver] Received message: {:?}", msg);
                
                match msg {
                    RelayMessage::PackInfo { room, from, pack_json } if room == share_code => {
                        info!("[P2P Receiver] Received pack info from {}", from);
                        
                        // Decrypt pack info
                        let key = {
                            let downloads = active_downloads.lock();
                            downloads.get(&share_code).map(|d| d.encryption_key)
                        };
                        
                        if let Some(key) = key {
                            let encrypted = base64::engine::general_purpose::STANDARD
                                .decode(&pack_json)
                                .map_err(|e| format!("Base64 decode error: {}", e))?;
                            
                            let decrypted = decrypt_data(&encrypted, &key)?;
                            let pack: ShareableModPack = serde_json::from_slice(&decrypted)
                                .map_err(|e| format!("JSON parse error: {}", e))?;
                            
                            info!("[P2P Receiver] Pack contains {} files:", pack.mods.len());
                            for (i, m) in pack.mods.iter().enumerate() {
                                info!("[P2P Receiver]   {}: {} ({} bytes)", i + 1, m.filename, m.size);
                            }
                            
                            // Update progress
                            {
                                let mut downloads = active_downloads.lock();
                                if let Some(download) = downloads.get_mut(&share_code) {
                                    download.progress.total_files = pack.mods.len();
                                    download.progress.total_bytes = pack.mods.iter().map(|m| m.size).sum();
                                    download.progress.status = TransferStatus::Transferring;
                                }
                            }
                            
                            mod_pack = Some(pack);
                            
                            // Start requesting first file
                            if let Some(ref pack) = mod_pack {
                                if !pack.mods.is_empty() {
                                    let filename = &pack.mods[0].filename;
                                    info!("[P2P Receiver] Requesting first file: {}", filename);
                                    client.send(RelayMessage::ChunkRequest {
                                        room: share_code.clone(),
                                        from: instance_id.clone(),
                                        to: sharer_id.clone(),
                                        filename: filename.clone(),
                                        chunk_index: 0,
                                    })?;
                                }
                            }
                        }
                    }
                    RelayMessage::ChunkData { room, from, to, filename, chunk_index, data, total_chunks, file_hash }
                        if room == share_code && to == instance_id => {
                        info!("[P2P Receiver] Received chunk {}/{} of '{}'", 
                            chunk_index + 1, total_chunks, filename);
                        
                        // Decrypt chunk
                        let (key, output_dir) = {
                            let downloads = active_downloads.lock();
                            downloads.get(&share_code)
                                .map(|d| (d.encryption_key, d.output_dir.clone()))
                                .ok_or("Download not found")?
                        };
                        
                        let encrypted = base64::engine::general_purpose::STANDARD
                            .decode(&data)
                            .map_err(|e| format!("Base64 decode error: {}", e))?;
                        
                        let decrypted = decrypt_data(&encrypted, &key)?;
                        
                        // Store chunk
                        {
                            let mut downloads = active_downloads.lock();
                            if let Some(download) = downloads.get_mut(&share_code) {
                                download.received_chunks
                                    .entry(filename.clone())
                                    .or_insert_with(HashMap::new)
                                    .insert(chunk_index, decrypted);
                                
                                download.file_metadata
                                    .entry(filename.clone())
                                    .or_insert((total_chunks, file_hash.clone()));
                                
                                // Update progress
                                let total_received: usize = download.received_chunks.values()
                                    .map(|chunks| chunks.values().map(|c| c.len()).sum::<usize>())
                                    .sum();
                                download.progress.bytes_transferred = total_received as u64;
                                download.progress.current_file = filename.clone();
                            }
                        }
                        
                        // Check if file is complete
                        let file_complete = {
                            let downloads = active_downloads.lock();
                            downloads.get(&share_code)
                                .and_then(|d| d.received_chunks.get(&filename))
                                .map(|chunks| chunks.len() as u32 >= total_chunks)
                                .unwrap_or(false)
                        };
                        
                        if file_complete {
                            info!("[P2P Receiver] File complete: {}", filename);
                            
                            // Write file to disk
                            let (chunks, metadata) = {
                                let downloads = active_downloads.lock();
                                let download = downloads.get(&share_code).ok_or("Download not found")?;
                                let chunks = download.received_chunks.get(&filename).cloned().unwrap_or_default();
                                let metadata = download.file_metadata.get(&filename).cloned().unwrap_or((0, String::new()));
                                (chunks, metadata)
                            };
                            
                            let file_path = output_dir.join(&filename);
                            p2p_relay::write_file_from_chunks(&file_path, &chunks, metadata.0, &metadata.1)?;
                            
                            info!("[P2P Receiver] File saved: {}", file_path.display());
                            
                            // Update progress
                            {
                                let mut downloads = active_downloads.lock();
                                if let Some(download) = downloads.get_mut(&share_code) {
                                    download.progress.files_completed += 1;
                                    
                                    // Clear chunks to free memory
                                    download.received_chunks.remove(&filename);
                                }
                            }
                            
                            // Request next file
                            current_file_index += 1;
                            current_chunk_index = 0;
                            
                            if let Some(ref pack) = mod_pack {
                                if current_file_index < pack.mods.len() {
                                    let next_filename = &pack.mods[current_file_index].filename;
                                    info!("[P2P Receiver] Requesting next file: {}", next_filename);
                                    client.send(RelayMessage::ChunkRequest {
                                        room: share_code.clone(),
                                        from: instance_id.clone(),
                                        to: sharer_id.clone(),
                                        filename: next_filename.clone(),
                                        chunk_index: 0,
                                    })?;
                                } else {
                                    info!("[P2P Receiver] All files received!");
                                    
                                    // Mark as complete
                                    {
                                        let mut downloads = active_downloads.lock();
                                        if let Some(download) = downloads.get_mut(&share_code) {
                                            download.progress.status = TransferStatus::Completed;
                                        }
                                    }
                                    
                                    client.send(RelayMessage::TransferComplete {
                                        room: share_code.clone(),
                                        from: instance_id.clone(),
                                    })?;
                                    
                                    break;
                                }
                            }
                        } else {
                            // Request next chunk
                            current_chunk_index = chunk_index + 1;
                            if current_chunk_index < total_chunks {
                                client.send(RelayMessage::ChunkRequest {
                                    room: share_code.clone(),
                                    from: instance_id.clone(),
                                    to: sharer_id.clone(),
                                    filename: filename.clone(),
                                    chunk_index: current_chunk_index,
                                })?;
                            }
                        }
                    }
                    RelayMessage::Error { room, message } if room == share_code => {
                        error!("[P2P Receiver] Error from sharer: {}", message);
                        
                        // Update status
                        {
                            let mut downloads = active_downloads.lock();
                            if let Some(download) = downloads.get_mut(&share_code) {
                                download.progress.status = TransferStatus::Failed(message.clone());
                            }
                        }
                        
                        return Err(message);
                    }
                    _ => {
                        debug!("[P2P Receiver] Ignoring message");
                    }
                }
            }
            Ok(None) => {
                info!("[P2P Receiver] Channel closed");
                break;
            }
            Err(_) => {
                // Timeout - request pack info again if we don't have it
                if mod_pack.is_none() {
                    info!("[P2P Receiver] Timeout waiting for pack info, retrying...");
                    client.send(RelayMessage::RequestPackInfo {
                        room: share_code.clone(),
                        from: instance_id.clone(),
                    })?;
                } else {
                    warn!("[P2P Receiver] Timeout waiting for chunk");
                }
            }
        }
    }
    
    info!("[P2P Receiver] Task ended for {}", share_code);
    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Validate a connection string
pub fn validate_connection_string(conn_str: &str) -> P2PResult<bool> {
    match ShareInfo::decode(conn_str) {
        Ok(_) => Ok(true),
        Err(e) => Err(P2PError::ValidationError(format!("Invalid connection string: {}", e))),
    }
}
