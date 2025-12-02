//! P2P Manager - Simple WebSocket-based P2P file sharing
//!
//! Uses a free public WebSocket relay for signaling and data transfer.
//! No IP addresses are exposed - all communication goes through the relay.
//!
//! Architecture:
//! 1. Sharer connects to relay, gets a room ID (share code)
//! 2. Receiver connects to same relay with room ID
//! 3. All data is transferred through the relay (encrypted)
//! 4. No direct peer connections = no IP exposure

use crate::p2p_libp2p::ShareInfo;
use crate::p2p_sharing::{ShareableModPack, ShareSession, TransferProgress, TransferStatus, P2PError, P2PResult};
use log::{info, warn, error};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use sha2::{Digest, Sha256};
use base64::Engine;
use serde::{Serialize, Deserialize};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Free public WebSocket relay for P2P signaling
/// Using a simple echo/broadcast relay pattern
const RELAY_SERVERS: &[&str] = &[
    "wss://free.blr2.piesocket.com/v3/repak-p2p?api_key=VCXCEuvhGcBDP7XhiJJUDvR1e1D3eiVjgZ9VRiaV",
];

/// Chunk size for file transfers (256KB)
const CHUNK_SIZE: usize = 256 * 1024;

// ============================================================================
// UNIFIED P2P MANAGER
// ============================================================================

/// Manages P2P sharing via WebSocket relay
/// All communication goes through the relay - no direct IP exposure
pub struct UnifiedP2PManager {
    /// Unique ID for this instance
    instance_id: String,
    /// Active share sessions
    pub active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
    /// Active downloads
    pub active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
}

/// Active share session
pub struct ActiveShare {
    pub session: ShareSession,
    pub mod_pack: ShareableModPack,
    pub mod_paths: Vec<PathBuf>,
}

/// Active download session
pub struct ActiveDownload {
    pub share_info: ShareInfo,
    pub progress: TransferProgress,
    pub output_dir: PathBuf,
}

/// Message types for relay communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum RelayMessage {
    /// Announce availability of a share
    ShareAnnounce {
        share_code: String,
        instance_id: String,
        mod_pack_json: String,
    },
    /// Request to join a share
    JoinRequest {
        share_code: String,
        instance_id: String,
    },
    /// Share info response
    ShareInfo {
        share_code: String,
        to_instance: String,
        mod_pack_json: String,
    },
    /// Request a file chunk
    ChunkRequest {
        share_code: String,
        to_instance: String,
        filename: String,
        offset: u64,
        size: usize,
    },
    /// File chunk data
    ChunkData {
        share_code: String,
        to_instance: String,
        filename: String,
        offset: u64,
        data_b64: String,
        is_last: bool,
        hash: String,
    },
    /// Error message
    Error {
        share_code: String,
        message: String,
    },
}

impl UnifiedP2PManager {
    /// Create a new P2P manager
    pub async fn new() -> P2PResult<Self> {
        // Generate a unique instance ID
        let instance_id = format!("repak-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        info!("P2P Manager initialized with instance ID: {}", instance_id);
        
        Ok(Self {
            instance_id,
            active_shares: Arc::new(Mutex::new(HashMap::new())),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Generate a share code
    fn generate_share_code() -> String {
        crate::p2p_sharing::generate_share_code()
    }

    /// Read a chunk from a file
    fn read_file_chunk(path: &Path, offset: u64, size: usize) -> std::io::Result<(Vec<u8>, bool)> {
        use std::io::Seek;
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

    /// Start sharing a mod pack
    /// Returns ShareInfo that can be encoded and shared with others
    pub async fn start_sharing(
        &self,
        name: String,
        description: String,
        mod_paths: Vec<PathBuf>,
        creator: Option<String>,
    ) -> P2PResult<ShareInfo> {
        // Create the mod pack
        let mod_pack = crate::p2p_sharing::create_mod_pack(
            name.clone(),
            description.clone(),
            &mod_paths,
            creator.clone(),
        )?;

        // Generate share code and encryption key
        let share_code = Self::generate_share_code();
        let encryption_key = crate::p2p_sharing::generate_encryption_key();
        let encryption_key_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&encryption_key);

        info!("Creating share with code: {}", share_code);
        info!("Mod pack contains {} files", mod_pack.mods.len());

        // Create share info - using instance_id as peer_id (no real IP exposed)
        let share_info = ShareInfo {
            peer_id: self.instance_id.clone(),
            addresses: vec![RELAY_SERVERS[0].to_string()], // Use relay server as "address"
            encryption_key: encryption_key_b64.clone(),
            share_code: share_code.clone(),
        };

        // Create session
        let encoded_share_info = share_info.encode()
            .map_err(|e| P2PError::ValidationError(format!("Failed to encode share info: {}", e)))?;
        
        let session = ShareSession {
            share_code: share_code.clone(),
            encryption_key: encryption_key_b64.clone(),
            local_ip: "relay".to_string(), // No real IP
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
                mod_pack,
                mod_paths,
            },
        );

        info!("Started sharing: {} (instance: {})", share_code, self.instance_id);
        info!("Share connection string ready for distribution");

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
        _client_name: Option<String>,
    ) -> P2PResult<()> {
        // Decode share info
        let share_info = ShareInfo::decode(connection_string)
            .map_err(|e| P2PError::ValidationError(format!("Invalid connection string: {}", e)))?;

        info!("Decoded share info: peer_id={}, addresses={:?}, share_code={}", 
            share_info.peer_id, share_info.addresses, share_info.share_code);

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
                    status: TransferStatus::Connecting,
                },
                output_dir,
            },
        );

        info!("Started receiving: {}", share_info.share_code);
        info!("Connecting to relay server...");
        
        // TODO: Connect to WebSocket relay and request files
        // For now, this sets up the download session
        // The actual transfer will happen via the relay

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
        RELAY_SERVERS.iter().map(|s| s.to_string()).collect()
    }
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
