use std::path::{Path, PathBuf};
use std::fs;
use log::info;

/// Heuristic detection for SKELETAL mesh UAsset files (NOT Static Meshes)
/// This is for "Fix Mesh" which applies to SK_* files
/// Static Meshes (SM_*) are handled separately by detect_static_mesh_files()
pub fn is_mesh_uasset_heuristic(path: &Path) -> bool {
    let file_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let path_str = path.to_string_lossy().to_lowercase();
    
    // ONLY Skeletal Mesh indicators (SK_* prefix)
    // Exclude Static Meshes (SM_* prefix) - they have their own fixer
    let skeletal_mesh_indicators = [
        "sk_", "skeletal", "_sk"
    ];
    
    // Pattern matching for Skeletal Meshes only
    let has_skeletal_pattern = skeletal_mesh_indicators.iter().any(|indicator| {
        file_name.contains(indicator) || path_str.contains(indicator)
    });
    
    // Explicitly exclude Static Meshes (SM_* prefix)
    let is_static_mesh = file_name.starts_with("sm_");
    
    has_skeletal_pattern && !is_static_mesh
}

/// Heuristic detection for texture UAsset files
pub fn is_texture_uasset_heuristic(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    
    // Path-based indicators
    let path_indicates_texture = 
        path_str.contains("texture") ||
        path_str.contains("/textures/") ||
        path_str.contains("\\textures\\") ||
        path_str.contains("_t.") ||      // UE naming convention
        path_str.contains("_diffuse") ||
        path_str.contains("_normal") ||
        path_str.contains("_specular") ||
        path_str.contains("_albedo") ||
        path_str.contains("_roughness") ||
        path_str.contains("_metallic");
    
    if path_indicates_texture {
        return true;
    }
    
    // Binary analysis fallback
    analyze_uasset_for_texture_class(path).unwrap_or(false)
}

/// Analyzes UAsset binary for texture class references
fn analyze_uasset_for_texture_class(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let data_str = String::from_utf8_lossy(&data);
    
    // Look for UTexture2D class references
    let has_texture_class = 
        data_str.contains("UTexture2D") ||
        data_str.contains("TextureSource") ||
        data_str.contains("MipGenSettings") ||
        data_str.contains("TMGS_");
    
    Ok(has_texture_class)
}

use uasset_toolkit::{UAssetToolkit, UAssetToolkitSync};

/// Detects SKELETAL mesh files in a list of mod contents using UAssetAPI (persistent process)
/// Async version for use in Tauri commands
pub async fn detect_mesh_files_async(mod_contents: &[String]) -> bool {
    use log::info;
    
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for SkeletalMesh", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Try batch detection first (much faster - single request for all files)
    if let Ok(toolkit) = UAssetToolkit::new(None) {
        info!("[Detection] Using batch detection for SkeletalMesh");
        match toolkit.batch_detect_skeletal_mesh(&uasset_files).await {
            Ok(true) => {
                info!("[Detection] FOUND SkeletalMesh (batch UAssetAPI)");
                return true;
            }
            Ok(false) => {
                info!("[Detection] No SkeletalMesh found via batch UAssetAPI");
                return false;
            }
            Err(e) => {
                let err_str = e.to_string();
                if !err_str.contains("File not found") {
                    info!("[Detection] Batch detection error: {}", e);
                }
            }
        }
    }

    // Fallback to heuristic detection (for PAK files where paths are virtual)
    info!("[Detection] Using heuristic fallback for SkeletalMesh detection");
    let result = mod_contents.iter().any(|file| {
        let path = PathBuf::from(file);
        if path.extension().and_then(|e| e.to_str()) == Some("uasset") {
            let is_mesh = is_mesh_uasset_heuristic(&path);
            if is_mesh {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                info!("[Detection] FOUND SkeletalMesh (heuristic): {}", filename);
            }
            return is_mesh;
        }
        false
    });
    
    if !result {
        info!("[Detection] No SkeletalMesh found via heuristic in {} files", uasset_files.len());
    }
    result
}

/// Detects texture files in a list of mod contents using UAssetAPI (persistent process)
/// Async version for use in Tauri commands
pub async fn detect_texture_files_async(mod_contents: &[String]) -> bool {
    use log::info;
    
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for Texture", uasset_files.len());
    
    // Check for raw image files first (always works, even for PAK paths)
    for file in mod_contents {
        let lower_file = file.to_lowercase();
        if lower_file.ends_with(".png") ||
           lower_file.ends_with(".jpg") ||
           lower_file.ends_with(".jpeg") ||
           lower_file.ends_with(".dds") ||
           lower_file.ends_with(".tga") {
            let path = PathBuf::from(file);
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
            info!("[Detection] FOUND raw image file: {}", filename);
            return true;
        }
    }
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Try batch detection first (much faster - single request for all files)
    if let Ok(toolkit) = UAssetToolkit::new(None) {
        info!("[Detection] Using batch detection for Texture");
        match toolkit.batch_detect_texture(&uasset_files).await {
            Ok(true) => {
                info!("[Detection] FOUND Texture (batch UAssetAPI)");
                return true;
            }
            Ok(false) => {
                info!("[Detection] No Texture found via batch UAssetAPI");
                return false;
            }
            Err(e) => {
                let err_str = e.to_string();
                if !err_str.contains("File not found") {
                    info!("[Detection] Batch texture detection error: {}", e);
                }
            }
        }
    }

    // Fallback to heuristic detection (for PAK files where paths are virtual)
    info!("[Detection] Using heuristic fallback for Texture detection");
    let result = mod_contents.iter().any(|file| {
        let path = PathBuf::from(file);
        if path.extension().and_then(|e| e.to_str()) == Some("uasset") {
            let is_texture = is_texture_uasset_heuristic(&path);
            if is_texture {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                info!("[Detection] FOUND Texture (heuristic): {}", filename);
            }
            return is_texture;
        }
        false
    });
    
    if !result {
        info!("[Detection] No Texture found via heuristic in {} files", uasset_files.len());
    }
    result
}

/// Heuristic detection for static mesh UAsset files
pub fn is_static_mesh_uasset_heuristic(path: &Path) -> bool {
    let file_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // SM_* prefix is the standard UE naming convention for Static Meshes
    file_name.starts_with("sm_")
}

/// Detects Static Mesh files in a list of mod contents using UAssetAPI (persistent process)
/// Async version for use in Tauri commands
pub async fn detect_static_mesh_files_async(mod_contents: &[String]) -> bool {
    use log::info;
    
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for StaticMesh", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Try batch detection first (much faster - single request for all files)
    if let Ok(toolkit) = UAssetToolkit::new(None) {
        info!("[Detection] Using batch detection for StaticMesh");
        match toolkit.batch_detect_static_mesh(&uasset_files).await {
            Ok(true) => {
                info!("[Detection] FOUND StaticMesh (batch UAssetAPI)");
                return true;
            }
            Ok(false) => {
                info!("[Detection] No StaticMesh found via batch UAssetAPI");
                return false;
            }
            Err(e) => {
                let err_str = e.to_string();
                if !err_str.contains("File not found") {
                    info!("[Detection] Batch static mesh detection error: {}", e);
                }
            }
        }
    }

    // Fallback to heuristic detection (for PAK files where paths are virtual)
    info!("[Detection] Using heuristic fallback for StaticMesh detection");
    let result = mod_contents.iter().any(|file| {
        let path = PathBuf::from(file);
        if path.extension().and_then(|e| e.to_str()) == Some("uasset") {
            let is_static = is_static_mesh_uasset_heuristic(&path);
            if is_static {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                info!("[Detection] FOUND StaticMesh (heuristic): {}", filename);
            }
            return is_static;
        }
        false
    });
    
    if !result {
        info!("[Detection] No StaticMesh found via heuristic in {} files", uasset_files.len());
    }
    result
}

/// Detects Blueprint files in a list of mod contents using UAssetAPI (persistent process)
/// Async version for use in Tauri commands
pub async fn detect_blueprint_files_async(mod_contents: &[String]) -> bool {
    use log::info;
    
    let uasset_files: Vec<String> = mod_contents.iter()
        .filter(|f| f.to_lowercase().ends_with(".uasset"))
        .cloned()
        .collect();
    info!("[Detection] Scanning {} uasset files for Blueprint", uasset_files.len());
    
    if uasset_files.is_empty() {
        return false;
    }
    
    // Try batch detection first (much faster - single request for all files)
    if let Ok(toolkit) = UAssetToolkit::new(None) {
        info!("[Detection] Using batch detection for Blueprint");
        match toolkit.batch_detect_blueprint(&uasset_files).await {
            Ok(true) => {
                info!("[Detection] FOUND Blueprint (batch UAssetAPI)");
                return true;
            }
            Ok(false) => {
                info!("[Detection] No Blueprint found via batch UAssetAPI");
                return false;
            }
            Err(e) => {
                let err_str = e.to_string();
                if !err_str.contains("File not found") {
                    info!("[Detection] Batch blueprint detection error: {}", e);
                }
            }
        }
    }

    false
}

/// Detects SKELETAL mesh files in a list of mod contents using UAssetAPI (persistent process)
/// This is for "Fix Mesh" auto-enable which applies to SK_* files only
pub fn detect_mesh_files(mod_contents: &[String]) -> bool {
    // Try to use UAssetToolkit with persistent process for batch scanning
    if let Ok(toolkit) = UAssetToolkitSync::new(None) {
        for file in mod_contents {
            let path = PathBuf::from(file);
            if path.extension().and_then(|e| e.to_str()) == Some("uasset") {
                if let Ok(true) = toolkit.is_skeletal_mesh_uasset(file) {
                    return true;
                }
            }
        }
        return false;
    }

    // Fallback to heuristic if toolkit unavailable
    /*
    mod_contents.iter().any(|file| {
        let path = PathBuf::from(file);
        if path.extension().and_then(|e| e.to_str()) == Some("uasset") {
            return is_mesh_uasset_heuristic(&path);
        }
        false
    })
    */
    false
}

/// Detects texture files in a list of mod contents using UAssetAPI (persistent process)
pub fn detect_texture_files(mod_contents: &[String]) -> bool {
    // Try to use UAssetToolkit with persistent process for batch scanning
    if let Ok(toolkit) = UAssetToolkitSync::new(None) {
        for file in mod_contents {
            let lower_file = file.to_lowercase();
            // Image file extensions
            if lower_file.ends_with(".png") ||
               lower_file.ends_with(".jpg") ||
               lower_file.ends_with(".jpeg") ||
               lower_file.ends_with(".dds") ||
               lower_file.ends_with(".tga") {
                return true;
            }
            
            if lower_file.ends_with(".uasset") {
                if let Ok(true) = toolkit.is_texture_uasset(file) {
                    return true;
                }
            }
        }
        return false;
    }

    // Fallback to heuristic
    /*
    mod_contents.iter().any(|file| {
        let lower_file = file.to_lowercase();
        
        // Image file extensions
        if lower_file.ends_with(".png") ||
           lower_file.ends_with(".jpg") ||
           lower_file.ends_with(".jpeg") ||
           lower_file.ends_with(".dds") ||
           lower_file.ends_with(".tga") {
            return true;
        }
        
        // UAsset files
        if lower_file.ends_with(".uasset") {
            let path = PathBuf::from(file);
            return is_texture_uasset_heuristic(&path);
        }
        
        false
    })
    */
    false
}

