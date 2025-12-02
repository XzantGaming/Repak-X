//! P2P Manager - File Hosting Implementation
//! Uses free file hosting (gofile.io) for reliable mod sharing

use crate::p2p_libp2p::ShareInfo;
use crate::p2p_sharing::{ShareableModPack, ShareSession, TransferProgress, TransferStatus, P2PError, P2PResult};
use log::{info, error};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use std::io::Write;

pub struct UnifiedP2PManager {
    instance_id: String,
    pub active_shares: Arc<Mutex<HashMap<String, ActiveShare>>>,
    pub active_downloads: Arc<Mutex<HashMap<String, ActiveDownload>>>,
}

pub struct ActiveShare {
    pub session: ShareSession,
    pub mod_pack: ShareableModPack,
    pub mod_paths: Vec<PathBuf>,
    pub download_url: String,
}

pub struct ActiveDownload {
    pub share_info: ShareInfo,
    pub progress: TransferProgress,
    pub output_dir: PathBuf,
}

impl UnifiedP2PManager {
    pub async fn new() -> P2PResult<Self> {
        let id = format!("repak-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        info!("[Share] Manager initialized: {}", id);
        Ok(Self {
            instance_id: id,
            active_shares: Arc::new(Mutex::new(HashMap::new())),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn start_sharing(&self, name: String, desc: String, paths: Vec<PathBuf>, creator: Option<String>) -> P2PResult<ShareInfo> {
        info!("[Share] Starting share: {} ({} files)", name, paths.len());
        
        let pack = crate::p2p_sharing::create_mod_pack(name.clone(), desc, &paths, creator)?;
        let code = crate::p2p_sharing::generate_share_code();

        // Create temp zip of all files
        let temp_dir = std::env::temp_dir().join(format!("repak_share_{}", code));
        std::fs::create_dir_all(&temp_dir).map_err(|e| P2PError::FileError(e.to_string()))?;
        let zip_path = temp_dir.join("mods.zip");
        
        info!("[Share] Creating zip at: {}", zip_path.display());
        create_zip(&paths, &zip_path)?;
        
        // Upload to file.io (provides direct download links)
        info!("[Share] Uploading to file.io...");
        let download_url = upload_to_fileio(&zip_path).await?;
        info!("[Share] Upload complete: {}", download_url);
        
        // Clean up temp zip
        let _ = std::fs::remove_file(&zip_path);
        let _ = std::fs::remove_dir(&temp_dir);

        let share_info = ShareInfo {
            peer_id: self.instance_id.clone(),
            addresses: vec![download_url.clone()],
            encryption_key: String::new(),
            share_code: code.clone(),
        };

        let conn = share_info.encode().map_err(|e| P2PError::ValidationError(format!("{}", e)))?;
        
        let sess = ShareSession {
            share_code: code.clone(),
            encryption_key: String::new(),
            local_ip: "cloud".into(),
            obfuscated_ip: "[Cloud Hosted]".into(),
            port: 0,
            connection_string: conn.clone(),
            obfuscated_connection_string: format!("Share Code: {}", code),
            active: true,
        };

        self.active_shares.lock().insert(code.clone(), ActiveShare {
            session: sess,
            mod_pack: pack,
            mod_paths: paths,
            download_url,
        });

        info!("[Share] Share ready! Code length: {} chars", conn.len());
        Ok(share_info)
    }

    pub fn stop_sharing(&self, code: &str) -> P2PResult<()> {
        info!("[Share] Stopping share: {}", code);
        self.active_shares.lock().remove(code);
        Ok(())
    }

    pub async fn start_receiving(&self, conn: &str, out: PathBuf, _name: Option<String>) -> P2PResult<()> {
        info!("[Share] Starting download to: {}", out.display());
        
        let share_info = ShareInfo::decode(conn).map_err(|e| P2PError::ValidationError(format!("{}", e)))?;
        let download_url = share_info.addresses.first().ok_or_else(|| P2PError::ValidationError("No download URL".into()))?.clone();
        let code = share_info.share_code.clone();

        info!("[Share] Download URL: {}", download_url);

        self.active_downloads.lock().insert(code.clone(), ActiveDownload {
            share_info: share_info.clone(),
            progress: TransferProgress {
                current_file: "Connecting...".into(),
                files_completed: 0,
                total_files: 1,
                bytes_transferred: 0,
                total_bytes: 0,
                status: TransferStatus::Connecting,
            },
            output_dir: out.clone(),
        });

        let dl = self.active_downloads.clone();
        let c = code.clone();
        
        tokio::spawn(async move {
            match download_and_extract(&download_url, &out, dl.clone(), &c).await {
                Ok(()) => {
                    info!("[Share] Download complete!");
                    if let Some(d) = dl.lock().get_mut(&c) {
                        d.progress.status = TransferStatus::Completed;
                        d.progress.files_completed = d.progress.total_files;
                    }
                }
                Err(e) => {
                    error!("[Share] Download failed: {}", e);
                    if let Some(d) = dl.lock().get_mut(&c) {
                        d.progress.status = TransferStatus::Failed(e.to_string());
                    }
                }
            }
        });

        Ok(())
    }

    pub fn get_share_session(&self, code: &str) -> Option<ShareSession> {
        self.active_shares.lock().get(code).map(|s| s.session.clone())
    }

    pub fn get_transfer_progress(&self, code: &str) -> Option<TransferProgress> {
        self.active_downloads.lock().get(code).map(|d| d.progress.clone())
    }

    pub fn is_sharing(&self, code: &str) -> bool { self.active_shares.lock().contains_key(code) }
    pub fn is_receiving(&self, code: &str) -> bool { self.active_downloads.lock().contains_key(code) }
    pub fn local_peer_id(&self) -> String { self.instance_id.clone() }
    pub fn listening_addresses(&self) -> Vec<String> { vec!["cloud://gofile.io".into()] }
}

fn create_zip(paths: &[PathBuf], zip_path: &PathBuf) -> P2PResult<()> {
    let file = std::fs::File::create(zip_path).map_err(|e| P2PError::FileError(e.to_string()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for path in paths {
        if path.is_file() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            info!("[Share] Adding to zip: {}", name);
            zip.start_file(name.to_string(), options).map_err(|e| P2PError::FileError(e.to_string()))?;
            let data = std::fs::read(path).map_err(|e| P2PError::FileError(e.to_string()))?;
            zip.write_all(&data).map_err(|e| P2PError::FileError(e.to_string()))?;
        }
    }
    zip.finish().map_err(|e| P2PError::FileError(e.to_string()))?;
    Ok(())
}

async fn upload_to_fileio(path: &PathBuf) -> P2PResult<String> {
    // file.io provides direct download links without authentication
    let client = reqwest::Client::new();
    
    let file_data = std::fs::read(path).map_err(|e| P2PError::FileError(e.to_string()))?;
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let file_size = file_data.len();
    
    info!("[Share] Uploading {} ({} bytes) to file.io...", file_name, file_size);
    
    let part = reqwest::multipart::Part::bytes(file_data)
        .file_name(file_name)
        .mime_str("application/zip").map_err(|e| P2PError::ConnectionError(e.to_string()))?;
    let form = reqwest::multipart::Form::new().part("file", part);

    let resp: serde_json::Value = client.post("https://file.io")
        .multipart(form)
        .send().await.map_err(|e| P2PError::ConnectionError(e.to_string()))?
        .json().await.map_err(|e| P2PError::ConnectionError(e.to_string()))?;

    info!("[Share] file.io response: {:?}", resp);

    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(P2PError::ConnectionError(format!("Upload failed: {:?}", resp)));
    }

    // file.io returns a direct download link
    let download_url = resp["link"].as_str()
        .ok_or_else(|| P2PError::ConnectionError("No download URL in response".into()))?;
    
    info!("[Share] Upload complete! URL: {}", download_url);
    Ok(download_url.to_string())
}

async fn download_and_extract(url: &str, out_dir: &PathBuf, dl: Arc<Mutex<HashMap<String, ActiveDownload>>>, code: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    
    // Update status
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.status = TransferStatus::Transferring;
        d.progress.current_file = "Downloading...".into();
    }

    // Download
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let total = resp.content_length().unwrap_or(0);
    
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.total_bytes = total;
    }

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.bytes_transferred = bytes.len() as u64;
        d.progress.current_file = "Extracting...".into();
    }

    // Save and extract
    let temp_zip = out_dir.join("_temp_download.zip");
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    std::fs::write(&temp_zip, &bytes).map_err(|e| e.to_string())?;

    // Extract zip
    let file = std::fs::File::open(&temp_zip).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    let total_files = archive.len();
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.total_files = total_files;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        let out_path = out_dir.join(&name);
        
        if let Some(d) = dl.lock().get_mut(code) {
            d.progress.current_file = name.clone();
            d.progress.files_completed = i;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut file, &mut out_file).map_err(|e| e.to_string())?;
        info!("[Share] Extracted: {}", name);
    }

    // Cleanup
    let _ = std::fs::remove_file(&temp_zip);
    
    Ok(())
}

pub fn validate_connection_string(s: &str) -> P2PResult<bool> {
    // Helper: try to decode a ShareInfo and ensure it has at least one address
    fn try_decode(input: &str) -> P2PResult<bool> {
        ShareInfo::decode(input)
            .map(|info| !info.addresses.is_empty())
            .map_err(|e| P2PError::ValidationError(format!("{}", e)))
    }

    // 1) First try the raw string as-is
    if let Ok(valid) = try_decode(s) {
        return Ok(valid);
    }

    // 2) Fallback: strip any non-base64 prefix (e.g. ":" or "Share Code: ")
    let trimmed = s.trim();
    let start_idx = trimmed
        .char_indices()
        .find(|&(_, c)| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        .map(|(i, _)| i);

    if let Some(i) = start_idx {
        let candidate = &trimmed[i..];
        return try_decode(candidate);
    }

    Err(P2PError::ValidationError("Invalid connection string".to_string()))
}