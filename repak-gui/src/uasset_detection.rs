//! UAsset detection using UAssetAPI (via UAssetTool)
//! 
//! All detection is done via UAssetAPI - no heuristic fallbacks.
//! If UAssetAPI fails (e.g., missing USMAP), detection returns false.
//!
//! Uses the global UAssetToolkit singleton for optimal performance -
//! the UAssetTool process is started once and reused for all operations.

use log::info;
use uasset_toolkit::get_global_toolkit;

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
    
    // Use global UAssetToolkit singleton for batch detection
    match get_global_toolkit() {
        Ok(toolkit) => {
            info!("[Detection] Using global UAssetToolkit singleton");
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
            info!("[Detection] ✗ Failed to get global UAssetToolkit: {}", e);
            info!("[Detection] UAssetAPI unavailable - cannot detect SkeletalMesh");
        }
    }

    false
}

/// Detects texture files that need the texture fix (Texture2D with .ubulk companion)
/// Uses UAssetAPI to find Texture2D assets, then checks if they have a matching .ubulk file
/// Async version for use in Tauri commands
pub async fn detect_texture_files_async(mod_contents: &[String]) -> bool {
    info!("[Detection] Texture detection received {} files to check", mod_contents.len());
    
    // Collect all .ubulk file stems (without extension) for quick lookup
    let ubulk_stems: std::collections::HashSet<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".ubulk"))
        .filter_map(|f| {
            std::path::Path::new(f)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
        })
        .collect();
    
    info!("[Detection] Found {} .ubulk files in input", ubulk_stems.len());
    
    if ubulk_stems.is_empty() {
        info!("[Detection] No .ubulk files found - texture fix NOT needed");
        return false;
    }
    
    // Get all .uasset files
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    
    info!("[Detection] Scanning {} uasset files for Texture2D with matching .ubulk", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Use global UAssetToolkit singleton to detect which files are textures
    match get_global_toolkit() {
        Ok(toolkit) => {
            info!("[Detection] Using global UAssetToolkit singleton");
            
            // Check each uasset file - if it's a texture AND has a matching .ubulk, we need the fix
            for uasset_path in &uasset_files {
                // Get the file stem to check against .ubulk files
                let file_stem = std::path::Path::new(uasset_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_lowercase());
                
                if let Some(stem) = file_stem {
                    // Check if there's a matching .ubulk file
                    if ubulk_stems.contains(&stem) {
                        // This .uasset has a matching .ubulk - check if it's a texture
                        match toolkit.is_texture_uasset(uasset_path).await {
                            Ok(true) => {
                                info!("[Detection] FOUND Texture2D with matching .ubulk: {} - texture fix ENABLED", uasset_path);
                                return true;
                            }
                            Ok(false) => {
                                // Not a texture, continue checking
                            }
                            Err(e) => {
                                info!("[Detection] Error checking {}: {}", uasset_path, e);
                            }
                        }
                    }
                }
            }
            
            info!("[Detection] No Texture2D with matching .ubulk found");
            return false;
        }
        Err(e) => {
            info!("[Detection] Failed to get global UAssetToolkit: {}", e);
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
    
    // Use global UAssetToolkit singleton for batch detection
    match get_global_toolkit() {
        Ok(toolkit) => {
            info!("[Detection] Using global UAssetToolkit singleton");
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
            info!("[Detection] Failed to get global UAssetToolkit: {}", e);
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
    
    // Use global UAssetToolkit singleton for batch detection
    if let Ok(toolkit) = get_global_toolkit() {
        info!("[Detection] Using global UAssetToolkit singleton for Blueprint");
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
    
    // Use global singleton for detection
    for file in uasset_files {
        if let Ok(true) = uasset_toolkit::is_skeletal_mesh_uasset(file) {
            return true;
        }
    }

    // UAssetAPI unavailable or no matches found
    false
}

/// Detects texture files that need the texture fix (Texture2D with .ubulk companion)
/// Uses UAssetAPI to find Texture2D assets, then checks if they have a matching .ubulk file
/// Sync version for use in install_mod.rs
pub fn detect_texture_files(mod_contents: &[String]) -> bool {
    // Collect all .ubulk file stems (without extension) for quick lookup
    let ubulk_stems: std::collections::HashSet<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".ubulk"))
        .filter_map(|f| {
            std::path::Path::new(f)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
        })
        .collect();
    
    if ubulk_stems.is_empty() {
        return false;
    }
    
    let uasset_files: Vec<&String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .collect();
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Check each uasset file - if it's a texture AND has a matching .ubulk, we need the fix
    for uasset_path in uasset_files {
        // Get the file stem to check against .ubulk files
        let file_stem = std::path::Path::new(uasset_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        
        if let Some(stem) = file_stem {
            // Check if there's a matching .ubulk file
            if ubulk_stems.contains(&stem) {
                // This .uasset has a matching .ubulk - check if it's a texture
                if let Ok(true) = uasset_toolkit::is_texture_uasset(uasset_path) {
                    return true;
                }
            }
        }
    }

    // No texture with matching .ubulk found
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
    
    // Use global singleton for detection
    for file in uasset_files {
        if let Ok(true) = uasset_toolkit::is_static_mesh_uasset(file) {
            return true;
        }
    }

    // UAssetAPI unavailable or no matches found
    false
}

