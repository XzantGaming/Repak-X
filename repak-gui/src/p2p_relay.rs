//! P2P Relay Client - WebSocket-based secure file transfer
//!
//! All communication goes through a WebSocket relay server.
//! No direct peer connections = No IP exposure = No doxxing.
//!
//! Security:
//! - All file data is AES-256-GCM encrypted before transmission
//! - Encryption key is part of the share code (not sent over relay)
//! - Relay only sees encrypted blobs, cannot read file contents

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Chunk size for file transfers (64KB - small for relay efficiency)
const CHUNK_SIZE: usize = 64 * 1024;

/// Relay server base URL (room/channel is appended)
const RELAY_BASE_URL: &str = "wss://free.blr2.piesocket.com/v3";
const RELAY_API_KEY: &str = "VCXCEuvhGcBDP7XhiJJUDvR1e1D3eiVjgZ9VRiaV";

// ============================================================================
// RELAY MESSAGE PROTOCOL
// ============================================================================

/// Messages sent over the relay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelayMessage {
    /// Join a share room
    Join {
        room: String,
        instance_id: String,
        role: String, // "sharer" or "receiver"
    },
    /// Sharer announces pack info
    PackInfo {
        room: String,
        from: String,
        pack_json: String, // Encrypted
    },
    /// Receiver requests pack info
    RequestPackInfo {
        room: String,
        from: String,
    },
    /// Request a file chunk
    ChunkRequest {
        room: String,
        from: String,
        to: String,
        filename: String,
        chunk_index: u32,
    },
    /// File chunk data
    ChunkData {
        room: String,
        from: String,
        to: String,
        filename: String,
        chunk_index: u32,
        data: String, // Base64 encoded encrypted data
        total_chunks: u32,
        file_hash: String,
    },
    /// Transfer complete
    TransferComplete {
        room: String,
        from: String,
    },
    /// Error
    Error {
        room: String,
        message: String,
    },
    /// Ping to keep connection alive
    Ping {
        room: String,
        from: String,
    },
}

// ============================================================================
// ENCRYPTION HELPERS
// ============================================================================

/// Encrypt data with AES-256-GCM
pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err(format!("Invalid key length: {} (expected 32)", key.len()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    // Generate random nonce
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

/// Decrypt data with AES-256-GCM
pub fn decrypt_data(encrypted: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err(format!("Invalid key length: {} (expected 32)", key.len()));
    }
    if encrypted.len() < 12 {
        return Err("Encrypted data too short".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let nonce = Nonce::from_slice(&encrypted[..12]);
    let ciphertext = &encrypted[12..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Derive 32-byte key from base64 encryption key
pub fn derive_key(key_b64: &str) -> Result<[u8; 32], String> {
    let key_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(key_b64)
        .map_err(|e| format!("Invalid key encoding: {}", e))?;

    if key_bytes.len() >= 32 {
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes[..32]);
        Ok(key)
    } else {
        // Pad with SHA256 hash if key is too short
        let mut hasher = Sha256::new();
        hasher.update(&key_bytes);
        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        Ok(key)
    }
}

// ============================================================================
// RELAY CLIENT
// ============================================================================

/// WebSocket relay client for P2P file transfer
pub struct RelayClient {
    instance_id: String,
    tx: mpsc::UnboundedSender<RelayMessage>,
    connected: Arc<RwLock<bool>>,
}

impl RelayClient {
    /// Connect to the relay server for a specific room
    pub async fn connect(instance_id: String) -> Result<(Self, mpsc::UnboundedReceiver<RelayMessage>), String> {
        // Use a default channel - will be overridden by connect_to_room
        Self::connect_to_room(instance_id, "lobby").await
    }
    
    /// Connect to a specific room/channel on the relay
    pub async fn connect_to_room(instance_id: String, room: &str) -> Result<(Self, mpsc::UnboundedReceiver<RelayMessage>), String> {
        // Each room gets its own WebSocket channel - this ensures messages are only
        // broadcast to clients in the same room
        let url = format!("{}{}?api_key={}&notify_self=1", RELAY_BASE_URL, room, RELAY_API_KEY);
        
        info!("[P2P Relay] Connecting to relay server...");
        info!("[P2P Relay] Room: {}", room);
        info!("[P2P Relay] URL: {}", url);

        let (ws_stream, response) = connect_async(&url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        info!("[P2P Relay] Connected! Response status: {}", response.status());

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut internal_rx) = mpsc::unbounded_channel::<RelayMessage>();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<RelayMessage>();
        let connected = Arc::new(RwLock::new(true));
        let connected_clone = connected.clone();
        let instance_id_clone = instance_id.clone();

        // Spawn writer task
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = internal_rx.recv().await {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("[P2P Relay] Failed to serialize message: {}", e);
                        continue;
                    }
                };
                info!("[P2P Relay] >>> SENDING: {}", &json[..json.len().min(500)]);
                if let Err(e) = write.send(Message::Text(json.clone())).await {
                    error!("[P2P Relay] Send error: {}", e);
                    break;
                }
            }
            info!("[P2P Relay] Writer task ended");
        });

        // Spawn reader task
        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        info!("[P2P Relay] <<< RECEIVED: {}", &text[..text.len().min(500)]);
                        match serde_json::from_str::<RelayMessage>(&text) {
                            Ok(relay_msg) => {
                                info!("[P2P Relay] Parsed as: {:?}", std::mem::discriminant(&relay_msg));
                                if let Err(e) = event_tx.send(relay_msg) {
                                    error!("[P2P Relay] Failed to forward message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                // Might be a system message from the relay
                                warn!("[P2P Relay] Non-protocol message (parse error: {}): {}", e, text);
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        debug!("[P2P Relay] Received ping");
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("[P2P Relay] Received pong");
                    }
                    Ok(Message::Close(frame)) => {
                        info!("[P2P Relay] Connection closed: {:?}", frame);
                        *connected_clone.write().await = false;
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("[P2P Relay] Read error: {}", e);
                        *connected_clone.write().await = false;
                        break;
                    }
                }
            }
            info!("[P2P Relay] Reader task ended");
        });

        Ok((
            Self {
                instance_id,
                tx,
                connected,
            },
            event_rx,
        ))
    }

    /// Send a message to the relay
    pub fn send(&self, msg: RelayMessage) -> Result<(), String> {
        self.tx
            .send(msg)
            .map_err(|e| format!("Failed to send: {}", e))
    }

    /// Announce presence in the room
    pub fn join_room(&self, room: &str, role: &str) -> Result<(), String> {
        info!("[P2P Relay] Announcing presence in room '{}' as {}", room, role);
        // Send a join message so other clients know we're here
        self.send(RelayMessage::Join {
            room: room.to_string(),
            instance_id: self.instance_id.clone(),
            role: role.to_string(),
        })
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Get instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

// ============================================================================
// FILE TRANSFER HELPERS
// ============================================================================

/// Read a file and split into chunks
pub fn read_file_chunks(path: &Path) -> Result<(Vec<Vec<u8>>, String), String> {
    use std::fs::File;
    use std::io::Read;

    info!("[P2P Transfer] Reading file: {}", path.display());

    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let hash = hex::encode(hasher.finalize());

    info!(
        "[P2P Transfer] File size: {} bytes, hash: {}",
        contents.len(),
        &hash[..16]
    );

    // Split into chunks
    let chunks: Vec<Vec<u8>> = contents.chunks(CHUNK_SIZE).map(|c| c.to_vec()).collect();

    info!("[P2P Transfer] Split into {} chunks", chunks.len());

    Ok((chunks, hash))
}

/// Write chunks to a file
pub fn write_file_from_chunks(
    path: &Path,
    chunks: &HashMap<u32, Vec<u8>>,
    total_chunks: u32,
    expected_hash: &str,
) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;

    info!(
        "[P2P Transfer] Writing file: {} ({} chunks)",
        path.display(),
        total_chunks
    );

    // Ensure we have all chunks
    for i in 0..total_chunks {
        if !chunks.contains_key(&i) {
            return Err(format!("Missing chunk {}", i));
        }
    }

    // Combine chunks
    let mut contents = Vec::new();
    for i in 0..total_chunks {
        contents.extend(&chunks[&i]);
    }

    // Verify hash
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash != expected_hash {
        return Err(format!(
            "Hash mismatch: expected {}, got {}",
            expected_hash, actual_hash
        ));
    }

    info!("[P2P Transfer] Hash verified: {}", &actual_hash[..16]);

    // Create parent directories
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    // Write file
    let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&contents)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    info!(
        "[P2P Transfer] File written successfully: {} bytes",
        contents.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = [0u8; 32];
        let data = b"Hello, World!";

        let encrypted = encrypt_data(data, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }
}
