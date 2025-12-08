//! UAsset detection using UAssetAPI (via UAssetBridge)
//! 
//! All detection is done via UAssetAPI - no heuristic fallbacks.
//! If UAssetAPI fails (e.g., missing USMAP), detection returns false.

use log::info;
use uasset_toolkit::{UAssetToolkit, UAssetToolkitSync};

/// Detects SKELETAL mesh files using UAssetAPI batch detection
/// Async version for use in Tauri commands
pub async fn detect_mesh_files_async(mod_contents: &[String]) -> bool {
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for SkeletalMesh", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI batch detection
    match UAssetToolkit::new(None) {
        Ok(toolkit) => {
            info!("[Detection] UAssetToolkit initialized successfully");
            info!("[Detection] Passing {} files to UAssetAPI batch_detect_skeletal_mesh", uasset_files.len());
            
            // Log first few files being checked
            for (i, file) in uasset_files.iter().take(3).enumerate() {
                info!("[Detection] UAssetAPI checking file {}: {}", i + 1, file);
            }
            
            info!("[Detection] Calling batch_detect_skeletal_mesh...");
            match toolkit.batch_detect_skeletal_mesh(&uasset_files).await {
                Ok(true) => {
                    info!("[Detection] ✓ FOUND SkeletalMesh (UAssetAPI returned true)");
                    return true;
                }
                Ok(false) => {
                    info!("[Detection] ✗ No SkeletalMesh found (UAssetAPI returned false)");
                    info!("[Detection] This means UAssetAPI checked the files but didn't identify any as SkeletalMesh");
                    return false;
                }
                Err(e) => {
                    info!("[Detection] ✗ UAssetAPI batch detection error: {}", e);
                    info!("[Detection] This may indicate USMAP issues or file read errors");
                }
            }
        }
        Err(e) => {
            info!("[Detection] ✗ Failed to initialize UAssetToolkit: {}", e);
            info!("[Detection] UAssetAPI unavailable - cannot detect SkeletalMesh");
        }
    }

    false
}

/// Detects texture files that need the texture fix (Texture2D with .ubulk companion)
/// Uses UAssetAPI batch detection to find Texture2D assets that need MipGen fixing,
/// but only returns true if there's also a .ubulk file (bulk texture data)
/// Async version for use in Tauri commands
pub async fn detect_texture_files_async(mod_contents: &[String]) -> bool {
    // First check: do we have any .ubulk files at all?
    // If not, no texture fix is needed regardless of Texture2D presence
    let has_ubulk = mod_contents.iter().any(|f| f.to_lowercase().ends_with(".ubulk"));
    if !has_ubulk {
        info!("[Detection] No .ubulk files found - texture fix NOT needed");
        return false;
    }
    
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for Texture2D needing MipGen fix (has .ubulk)", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI batch detection - checks if texture needs MipGen fix
    // (is Texture2D AND MipGenSettings != NoMipmaps)
    match UAssetToolkit::new(None) {
        Ok(toolkit) => {
            info!("[Detection] UAssetToolkit initialized successfully");
            info!("[Detection] Using UAssetAPI batch detection for Texture2D");
            match toolkit.batch_detect_texture(&uasset_files).await {
                Ok(true) => {
                    info!("[Detection] FOUND Texture2D needing MipGen fix with .ubulk - texture fix ENABLED");
                    return true;
                }
                Ok(false) => {
                    info!("[Detection] No Texture2D needing MipGen fix found (UAssetAPI)");
                    return false;
                }
                Err(e) => {
                    info!("[Detection] UAssetAPI batch detection error: {}", e);
                    info!("[Detection] This may indicate USMAP issues or file read errors");
                }
            }
        }
        Err(e) => {
            info!("[Detection] Failed to initialize UAssetToolkit: {}", e);
            info!("[Detection] UAssetAPI unavailable - cannot detect Texture2D");
        }
    }

    false
}

/// Detects Static Mesh files using UAssetAPI batch detection
/// Async version for use in Tauri commands
pub async fn detect_static_mesh_files_async(mod_contents: &[String]) -> bool {
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for StaticMesh", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI batch detection
    match UAssetToolkit::new(None) {
        Ok(toolkit) => {
            info!("[Detection] UAssetToolkit initialized successfully");
            info!("[Detection] Using UAssetAPI batch detection for StaticMesh");
            match toolkit.batch_detect_static_mesh(&uasset_files).await {
                Ok(true) => {
                    info!("[Detection] FOUND StaticMesh (UAssetAPI)");
                    return true;
                }
                Ok(false) => {
                    info!("[Detection] No StaticMesh found (UAssetAPI)");
                    return false;
                }
                Err(e) => {
                    info!("[Detection] UAssetAPI batch detection error: {}", e);
                    info!("[Detection] This may indicate USMAP issues or file read errors");
                }
            }
        }
        Err(e) => {
            info!("[Detection] Failed to initialize UAssetToolkit: {}", e);
            info!("[Detection] UAssetAPI unavailable - cannot detect StaticMesh");
        }
    }

    false
}

/// Detects Blueprint files using UAssetAPI batch detection
/// Async version for use in Tauri commands
#[allow(dead_code)]
pub async fn detect_blueprint_files_async(mod_contents: &[String]) -> bool {
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for Blueprint", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI batch detection
    if let Ok(toolkit) = UAssetToolkit::new(None) {
        info!("[Detection] Using UAssetAPI batch detection for Blueprint");
        match toolkit.batch_detect_blueprint(&uasset_files).await {
            Ok(true) => {
                info!("[Detection] FOUND Blueprint (UAssetAPI)");
                return true;
            }
            Ok(false) => {
                info!("[Detection] No Blueprint found (UAssetAPI)");
                return false;
            }
            Err(e) => {
                info!("[Detection] UAssetAPI error (check USMAP config): {}", e);
            }
        }
    } else {
        info!("[Detection] UAssetAPI unavailable - cannot detect Blueprint");
    }

    false
}

/// Detects SKELETAL mesh files using UAssetAPI
/// Sync version for use in install_mod.rs
pub fn detect_mesh_files(mod_contents: &[String]) -> bool {
    let uasset_files: Vec<&String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .collect();
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI for detection
    if let Ok(toolkit) = UAssetToolkitSync::new(None) {
        for file in uasset_files {
            if let Ok(true) = toolkit.is_skeletal_mesh_uasset(file) {
                return true;
            }
        }
    }

    // UAssetAPI unavailable or no matches found
    false
}

/// Detects texture files that need the texture fix (Texture2D with .ubulk companion)
/// Uses UAssetAPI to find Texture2D assets that need MipGen fixing,
/// but only returns true if there's also a .ubulk file (bulk texture data)
/// Sync version for use in install_mod.rs
pub fn detect_texture_files(mod_contents: &[String]) -> bool {
    // First check: do we have any .ubulk files at all?
    // If not, no texture fix is needed regardless of Texture2D presence
    let has_ubulk = mod_contents.iter().any(|f| f.to_lowercase().ends_with(".ubulk"));
    
    if !has_ubulk {
        return false;
    }
    
    let uasset_files: Vec<&String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .collect();
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI to detect Texture2D assets needing MipGen fix
    if let Ok(toolkit) = UAssetToolkitSync::new(None) {
        for file in uasset_files {
            if let Ok(true) = toolkit.is_texture_uasset(file) {
                // UAssetAPI found a texture needing fix, and we already confirmed .ubulk exists
                return true;
            }
        }
    }

    // UAssetAPI unavailable or no matches found
    false
}

/// Detects Static Mesh files using UAssetAPI
/// Sync version for use in install_mod.rs
pub fn detect_static_mesh_files(mod_contents: &[String]) -> bool {
    let uasset_files: Vec<&String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .collect();
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use UAssetAPI for detection
    if let Ok(toolkit) = UAssetToolkitSync::new(None) {
        for file in uasset_files {
            if let Ok(true) = toolkit.is_static_mesh_uasset(file) {
                return true;
            }
        }
    }

    // UAssetAPI unavailable or no matches found
    false
}

