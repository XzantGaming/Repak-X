#![allow(dead_code)]
//! P2P Manager - File Hosting Implementation
//! Uses free file hosting (0x0.st) for reliable mod sharing

use crate::p2p_libp2p::ShareInfo;
use crate::p2p_sharing::{ShareableModPack, ShareSession, TransferProgress, TransferStatus, P2PError, P2PResult};
use log::{info, error};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::io::Write;
use parking_lot::Mutex;
use tauri::{Emitter, Window};

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
        
        // Upload to 0x0.st (provides direct download links)
        info!("[Share] Uploading to 0x0.st...");
        let download_url = upload_to_fileio(&zip_path).await?;
        info!("[Share] Upload complete: {}", download_url);
        
        // Clean up temp zip
        let _ = std::fs::remove_file(&zip_path);
        let _ = std::fs::remove_dir(&temp_dir);

        let share_info = ShareInfo {
            peer_id: self.instance_id.clone(),
            addresses: vec![download_url.clone()],
            encryption_key: "cloud-hosted".to_string(), // Placeholder for cloud hosting (no encryption needed)
            share_code: code.clone(),
        };

        let conn = share_info.encode().map_err(|e| P2PError::ValidationError(format!("{}", e)))?;
        
        let sess = ShareSession {
            share_code: code.clone(),
            encryption_key: share_info.encryption_key.clone(),
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

    pub async fn start_receiving(&self, conn: &str, out: PathBuf, _name: Option<String>, window: Window) -> P2PResult<()> {
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
                    // Emit event to refresh mod list
                    info!("[Share] Emitting mods_dir_changed event to refresh UI");
                    let _ = window.emit("mods_dir_changed", ());
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
    pub fn listening_addresses(&self) -> Vec<String> { vec!["cloud://0x0.st".into()] }
}

fn create_zip(paths: &[PathBuf], zip_path: &PathBuf) -> P2PResult<()> {
    let file = std::fs::File::create(zip_path).map_err(|e| P2PError::FileError(e.to_string()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Track files we've already added to avoid duplicates
    let mut added_files = std::collections::HashSet::new();

    for path in paths {
        if path.is_file() {
            let original_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            
            // Check if this is a disabled mod (.bak_repak) - rename to .pak for enabled state
            let (zip_name, is_disabled_mod) = if original_name.ends_with(".bak_repak") {
                let enabled_name = original_name.replace(".bak_repak", ".pak");
                info!("[Share] Converting disabled mod to enabled: {} -> {}", original_name, enabled_name);
                (enabled_name, true)
            } else {
                (original_name.clone(), false)
            };
            
            // Add the main file with the appropriate name
            if added_files.insert(zip_name.clone()) {
                info!("[Share] Adding to zip: {}", zip_name);
                zip.start_file(zip_name.clone(), options).map_err(|e| P2PError::FileError(e.to_string()))?;
                let data = std::fs::read(path).map_err(|e| P2PError::FileError(e.to_string()))?;
                zip.write_all(&data).map_err(|e| P2PError::FileError(e.to_string()))?;
            }

            // Check for accompanying .ucas and .utoc files
            // Works for both .pak and .bak_repak files
            let should_check_companions = zip_name.ends_with(".pak");
            
            if should_check_companions {
                let base_name = &zip_name[..zip_name.len() - 4]; // Remove ".pak"
                let parent_dir = path.parent();

                if let Some(dir) = parent_dir {
                    // Determine the base name on disk (for disabled mods, companions also have .bak_repak)
                    let disk_base = if is_disabled_mod {
                        original_name[..original_name.len() - 10].to_string() // Remove ".bak_repak"
                    } else {
                        base_name.to_string()
                    };

                    // Check for .ucas file
                    let ucas_disk_name = if is_disabled_mod {
                        format!("{}.ucas.bak_repak", disk_base)
                    } else {
                        format!("{}.ucas", disk_base)
                    };
                    let ucas_zip_name = format!("{}.ucas", base_name);
                    let ucas_path = dir.join(&ucas_disk_name);
                    
                    if ucas_path.exists() && added_files.insert(ucas_zip_name.clone()) {
                        info!("[Share] Adding accompanying file: {} (from {})", ucas_zip_name, ucas_disk_name);
                        zip.start_file(ucas_zip_name, options).map_err(|e| P2PError::FileError(e.to_string()))?;
                        let data = std::fs::read(&ucas_path).map_err(|e| P2PError::FileError(e.to_string()))?;
                        zip.write_all(&data).map_err(|e| P2PError::FileError(e.to_string()))?;
                    }

                    // Check for .utoc file
                    let utoc_disk_name = if is_disabled_mod {
                        format!("{}.utoc.bak_repak", disk_base)
                    } else {
                        format!("{}.utoc", disk_base)
                    };
                    let utoc_zip_name = format!("{}.utoc", base_name);
                    let utoc_path = dir.join(&utoc_disk_name);
                    
                    if utoc_path.exists() && added_files.insert(utoc_zip_name.clone()) {
                        info!("[Share] Adding accompanying file: {} (from {})", utoc_zip_name, utoc_disk_name);
                        zip.start_file(utoc_zip_name, options).map_err(|e| P2PError::FileError(e.to_string()))?;
                        let data = std::fs::read(&utoc_path).map_err(|e| P2PError::FileError(e.to_string()))?;
                        zip.write_all(&data).map_err(|e| P2PError::FileError(e.to_string()))?;
                    }
                }
            }
        }
    }
    zip.finish().map_err(|e| P2PError::FileError(e.to_string()))?;
    Ok(())
}

async fn upload_to_fileio(path: &PathBuf) -> P2PResult<String> {
    // 0x0.st provides simple, reliable file hosting (512MB limit, 365 day retention)
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for large files
        .build()
        .map_err(|e| P2PError::ConnectionError(format!("Failed to create HTTP client: {}", e)))?;
    
    let file_data = std::fs::read(path).map_err(|e| P2PError::FileError(e.to_string()))?;
    let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let file_size = file_data.len();
    
    info!("[Share] Uploading {} ({} bytes / {:.2} MB) to 0x0.st...", file_name, file_size, file_size as f64 / 1024.0 / 1024.0);
    
    // Check file size (0x0.st has 512MB limit)
    if file_size > 512 * 1024 * 1024 {
        error!("[Share] File too large: {} bytes (max 512MB)", file_size);
        return Err(P2PError::FileError("File exceeds 512MB limit".into()));
    }
    
    let part = reqwest::multipart::Part::bytes(file_data)
        .file_name(file_name.clone())
        .mime_str("application/zip").map_err(|e| P2PError::ConnectionError(e.to_string()))?;
    let form = reqwest::multipart::Form::new().part("file", part);

    info!("[Share] Sending upload request to 0x0.st...");
    let response = client.post("https://0x0.st")
        .header("User-Agent", "RepakX/1.0")
        .multipart(form)
        .send().await.map_err(|e| P2PError::ConnectionError(format!("Failed to send request: {}", e)))?;

    // Check status code first
    let status = response.status();
    info!("[Share] 0x0.st response status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        error!("[Share] Upload failed with status {}: {}", status, error_text);
        return Err(P2PError::ConnectionError(format!("Upload failed with status {}: {}", status, error_text)));
    }

    // 0x0.st returns the download URL as plain text
    let download_url = response.text().await
        .map_err(|e| P2PError::ConnectionError(format!("Failed to read response: {}", e)))?
        .trim()
        .to_string();
    
    info!("[Share] 0x0.st raw response: {}", download_url);

    // Validate URL format
    if !download_url.starts_with("https://0x0.st/") {
        error!("[Share] Invalid response from 0x0.st: {}", download_url);
        return Err(P2PError::ConnectionError(format!("Invalid response from server: {}", download_url)));
    }
    
    info!("[Share] Upload complete! URL: {} (expires in 365 days)", download_url);
    Ok(download_url)
}

async fn download_and_extract(url: &str, out_dir: &PathBuf, dl: Arc<Mutex<HashMap<String, ActiveDownload>>>, code: &str) -> Result<(), String> {
    info!("[Share] Starting download from: {}", url);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 minute timeout for downloads
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Update status
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.status = TransferStatus::Transferring;
        d.progress.current_file = "Downloading...".into();
    }

    // Download with better error handling
    info!("[Share] Sending download request...");
    let resp = client.get(url).send().await.map_err(|e| {
        error!("[Share] Download request failed: {}", e);
        format!("Failed to download: {}", e)
    })?;
    
    let status = resp.status();
    info!("[Share] Download response status: {}", status);
    
    if !status.is_success() {
        let error_msg = format!("Download failed with status: {}", status);
        error!("[Share] {}", error_msg);
        return Err(error_msg);
    }
    
    let total = resp.content_length().unwrap_or(0);
    info!("[Share] Download size: {} bytes ({:.2} MB)", total, total as f64 / 1024.0 / 1024.0);
    
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.total_bytes = total;
    }

    info!("[Share] Downloading file...");
    let bytes = resp.bytes().await.map_err(|e| {
        error!("[Share] Failed to read download bytes: {}", e);
        format!("Failed to read download: {}", e)
    })?;
    
    info!("[Share] Downloaded {} bytes", bytes.len());
    
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.bytes_transferred = bytes.len() as u64;
        d.progress.current_file = "Extracting...".into();
        d.progress.status = TransferStatus::Verifying;
    }

    // Save and extract
    let temp_zip = out_dir.join("_temp_download.zip");
    std::fs::create_dir_all(out_dir).map_err(|e| {
        error!("[Share] Failed to create output directory: {}", e);
        e.to_string()
    })?;
    
    info!("[Share] Saving zip to: {}", temp_zip.display());
    std::fs::write(&temp_zip, &bytes).map_err(|e| {
        error!("[Share] Failed to write zip file: {}", e);
        e.to_string()
    })?;

    // Extract zip
    info!("[Share] Opening zip archive...");
    let file = std::fs::File::open(&temp_zip).map_err(|e| {
        error!("[Share] Failed to open zip: {}", e);
        e.to_string()
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        error!("[Share] Failed to read zip archive: {}", e);
        format!("Invalid or corrupted zip file: {}", e)
    })?;
    
    let total_files = archive.len();
    info!("[Share] Extracting {} files...", total_files);
    
    if let Some(d) = dl.lock().get_mut(code) {
        d.progress.total_files = total_files;
        d.progress.status = TransferStatus::Transferring;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            error!("[Share] Failed to access file {} in zip: {}", i, e);
            e.to_string()
        })?;
        let name = file.name().to_string();
        let out_path = out_dir.join(&name);
        
        info!("[Share] Extracting: {} ({}/{})", name, i + 1, total_files);
        
        if let Some(d) = dl.lock().get_mut(code) {
            d.progress.current_file = name.clone();
            d.progress.files_completed = i;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                error!("[Share] Failed to create directory: {}", e);
                e.to_string()
            })?;
        }
        
        let mut out_file = std::fs::File::create(&out_path).map_err(|e| {
            error!("[Share] Failed to create output file {}: {}", name, e);
            e.to_string()
        })?;
        std::io::copy(&mut file, &mut out_file).map_err(|e| {
            error!("[Share] Failed to extract {}: {}", name, e);
            e.to_string()
        })?;
        info!("[Share] ✓ Extracted: {}", name);
    }

    // Cleanup
    info!("[Share] Cleaning up temporary files...");
    let _ = std::fs::remove_file(&temp_zip);
    
    info!("[Share] Download and extraction complete!");
    Ok(())
}

pub fn validate_connection_string(s: &str) -> P2PResult<bool> {
    info!("[Share] Validating connection string: {} chars", s.len());
    
    // Helper: try to decode a ShareInfo and ensure it has at least one address
    fn try_decode(input: &str) -> P2PResult<bool> {
        match ShareInfo::decode(input) {
            Ok(info) => {
                info!("[Share] Decoded ShareInfo - peer_id: {}, addresses: {:?}, encryption_key: {}, share_code: {}", 
                    info.peer_id, info.addresses, info.encryption_key, info.share_code);
                Ok(!info.addresses.is_empty())
            },
            Err(e) => {
                error!("[Share] Failed to decode: {}", e);
                Err(P2PError::ValidationError(format!("{}", e)))
            }
        }
    }

    // 1) First try the raw string as-is
    if let Ok(valid) = try_decode(s) {
        info!("[Share] Validation result: {}", valid);
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
        info!("[Share] Trying trimmed candidate: {} chars", candidate.len());
        return try_decode(candidate);
    }

    error!("[Share] Validation failed - invalid format");
    Err(P2PError::ValidationError("Invalid connection string".to_string()))
}