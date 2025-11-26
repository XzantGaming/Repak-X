use crate::install_mod::install_mod_logic::pak_files::repak_dir;
use crate::install_mod::install_mod_logic::patch_meshes;
use crate::install_mod::{InstallableMod, AES_KEY};
use crate::uasset_detection::{modify_texture_mipmaps, patch_mesh_files};
use crate::uasset_api_integration::process_texture_with_uasset_api;
use crate::utils::collect_files;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use repak::Version;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::AtomicI32;
use retoc::*;
use std::sync::Arc;
use log::{debug, error, warn, info};
use std::fs::File;
use std::process::Command;
use path_slash::PathExt;
use serde::{Deserialize, Serialize};

// Windows-specific: Hide CMD windows when spawning processes
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn convert_to_iostore_directory(
    pak: &InstallableMod,
    mod_dir: PathBuf,
    to_pak_dir: PathBuf,
    packed_files_count: &AtomicI32,
) -> Result<(), repak::Error> {
    let mod_type = pak.mod_type.clone();
    if mod_type == "Audio" || mod_type == "Movies" {
        debug!("{} mod detected. Not creating iostore packages",mod_type);
        repak_dir(pak, to_pak_dir, mod_dir, packed_files_count)?;
        return Ok(());
    }


    let mut pak_name = pak.mod_name.clone();
    pak_name.push_str(".pak");

    let mut utoc_name = pak.mod_name.clone();
    utoc_name.push_str(".utoc");

    let mut paths = vec![];
    collect_files(&mut paths, &to_pak_dir)?;

    // CRITICAL: Fix Static Mesh SerializeSize and SKIP IoStore conversion
    // IoStore conversion (action_to_zen) rebuilds assets and overwrites SerializeSize fixes!
    // Solution: Use regular .pak format to preserve fixes
    if pak.fix_serialsize_header {
        info!("╔══════════════════════════════════════════════════════════╗");
        info!("║  STATIC MESH SERIALIZESIZE FIX - STARTING                ║");
        info!("╚══════════════════════════════════════════════════════════╝");
        
        // Check for usmap file (required for unversioned assets)
        let usmap_full_path = if !pak.usmap_path.is_empty() {
            // Construct full path to Usmap folder
            if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
                let usmap_file = exe_dir.join("Usmap").join(&pak.usmap_path);
                if usmap_file.exists() {
                    Some(usmap_file.to_string_lossy().to_string())
                } else {
                    warn!("USmap file not found in Usmap folder: {}", pak.usmap_path);
                    None
                }
            } else {
                warn!("Could not determine executable directory for USmap");
                None
            }
        } else {
            warn!("No usmap file specified - detection may be limited for unversioned assets");
            None
        };
        
        let usmap_path = usmap_full_path.as_deref();
        
        match process_static_mesh_serializesize(&to_pak_dir, usmap_path) {
            Ok(fixed_count) => {
                if fixed_count > 0 {
                    info!("✓ Fixed SerializeSize for {} Static Mesh(es)", fixed_count);
                    info!("   Proceeding with IoStore conversion...");
                } else {
                    info!("✓ No Static Mesh SerializeSize fixes needed");
                }
            }
            Err(e) => {
                error!("✗ Static Mesh SerializeSize fix failed: {}", e);
                return Err(repak::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("SerializeSize fix failed: {}", e),
                )));
            }
        }
    }

    // Skeletal Mesh patching (separate workflow, not interfered with)
    if pak.fix_mesh {
        patch_meshes::mesh_patch(&mut paths, &to_pak_dir.to_path_buf())?;
    }

    if pak.fix_textures {
        if let Err(e) = process_texture_files(&paths) {
            error!("Failed to process texture files: {}", e);
        }
    }

    let action = ActionToZen::new(
        to_pak_dir.clone(),
        mod_dir.join(utoc_name),
        EngineVersion::UE5_3,
    );
    let mut config = Config {
        container_header_version_override: None,
        ..Default::default()
    };

    let aes_toc =
        retoc::AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
            .unwrap();

    config.aes_keys.insert(FGuid::default(), aes_toc.clone());
    let config = Arc::new(config);

    action_to_zen(action, config).expect("Failed to convert to zen");

    // NOW WE CREATE THE FAKE PAK FILE WITH THE CONTENTS BEING A TEXT FILE LISTING ALL CHUNKNAMES

    let output_file = File::create(mod_dir.join(pak_name))?;

    let rel_paths = paths
        .par_iter()
        .map(|p| {
            let rel = &p
                .strip_prefix(to_pak_dir.clone())
                .expect("file not in input directory")
                .to_slash()
                .expect("failed to convert to slash path");
            rel.to_string()
        })
        .collect::<Vec<_>>();

    // Build the tiny companion PAK uncompressed on purpose.
    // Rationale: Only UCAS should be compressed; the small PAK is only a mount aid (chunknames)
    // and keeping it uncompressed improves compatibility.
    // To revert: add `.compression(vec![pak.compression])` back below and set build_entry to true.
    let builder = repak::PakBuilder::new()
        .key(AES_KEY.clone().0);

    let mut pak_writer = builder.writer(
        BufWriter::new(output_file),
        Version::V11,
        pak.mount_point.clone(),
        Some(pak.path_hash_seed.parse().unwrap()),
    );
    let entry_builder = pak_writer.entry_builder();

    let rel_paths_bytes: Vec<u8> = rel_paths.join("\n").into_bytes();
    // Write the chunknames entry without compression
    let entry = entry_builder
        .build_entry(false, rel_paths_bytes, "chunknames")
        .expect("Failed to build entry");

    pak_writer.write_entry("chunknames".to_string(), entry)?;
    pak_writer.write_index()?;

    log::info!("Wrote pak file successfully");
    packed_files_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(())

    // now generate the fake pak file
}

/// Process texture files to set MipGenSettings to NoMipmaps
pub fn process_texture_files(paths: &Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Filter for .uasset files that are in a "texture" related folder
    let uasset_files: Vec<_> = paths
        .iter()
        .filter(|p| {
            // Must be a .uasset file
            if p.extension().and_then(|ext| ext.to_str()) != Some("uasset") {
                return false;
            }

            // Check if any parent folder contains "texture" (case-insensitive)
            let path_str = p.to_string_lossy().to_lowercase();
            
            // Check for "texture" in the path (simple substring check is robust enough and covers "Textures", "texture", "MyTextures", etc.)
            // This covers both folder names and file names, but since we're looking for folder structure mostly, this is a good heuristic.
            // User requested: "detect if there is a folder in the pak directories that contains the word 'texture'"
            if path_str.contains("texture") {
                return true;
            }
            
            false
        })
        .collect();

    debug!("Found {} potential texture files (in 'texture' folders) to check", uasset_files.len());

    for uasset_file in &uasset_files {
        // Create backups
        if let Err(e) = std::fs::copy(uasset_file, format!("{}.bak", uasset_file.display())) {
            warn!("Failed to create backup for {}: {}", uasset_file.display(), e);
        }

        // Use UAssetAPI exclusively for MipGenSettings -> NoMipmaps
        match process_texture_with_uasset_api(uasset_file) {
            Ok(true) => {
                debug!("Successfully processed texture with UAssetAPI (NoMipmaps): {:?}", uasset_file);
            }
            Ok(false) => {
                // Not a texture or couldn't be processed
                // debug!("File is not a texture or skipped: {:?}", uasset_file);
            }
            Err(e) => {
                error!(
                    "UAssetAPI texture processing error for {:?}: {}. Mipmaps were NOT modified.",
                    uasset_file,
                    e
                );
            }
        }
    }
    
    Ok(())
}

/// Asset type detection result from UAssetAPI tool
#[derive(Debug, Deserialize, Serialize)]
struct AssetDetectionResult {
    path: String,
    asset_type: String,
    export_count: usize,
    import_count: usize,
}

/// SerializeSize fix result from UAssetAPI tool
#[derive(Debug, Deserialize, Serialize)]
struct SerializeSizeFixResult {
    success: bool,
    message: String,
    fixed_count: Option<usize>,
    asset_type: Option<String>,
}

/// Detect asset type using UAssetAPI (no heuristics!)
fn detect_asset_type_with_uasset_api(uasset_path: &Path, usmap_path: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let tool_paths = [
        "../../UAssetAPI/StaticMeshSerializeSizeFixer/bin/Release/net8.0/win-x64/StaticMeshSerializeSizeFixer.exe",
        "../../../UAssetAPI/StaticMeshSerializeSizeFixer/bin/Release/net8.0/win-x64/StaticMeshSerializeSizeFixer.exe",
        "./StaticMeshSerializeSizeFixer.exe",
        "./tools/StaticMeshSerializeSizeFixer/StaticMeshSerializeSizeFixer.exe",
    ];

    let mut tool_path = None;
    for path in &tool_paths {
        if std::path::Path::new(path).exists() {
            tool_path = Some(*path);
            info!("   🔧 Found tool at: {}", path);
            
            // Get file metadata to check when it was last modified
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    info!("   📅 Tool last modified: {:?}", modified);
                }
                info!("   📏 Tool size: {} bytes", metadata.len());
            }
            break;
        }
    }

    let tool_path = tool_path.ok_or("StaticMeshSerializeSizeFixer.exe not found in any search path")?;

    let mut cmd = Command::new(tool_path);
    
    // Hide CMD window on Windows (CREATE_NO_WINDOW flag)
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    
    cmd.arg("detect").arg(uasset_path);
    
    // Add usmap path if provided
    if let Some(usmap) = usmap_path {
        cmd.arg(usmap);
        debug!("   Running: {} detect {:?} {}", tool_path, uasset_path, usmap);
    } else {
        debug!("   Running: {} detect {:?}", tool_path, uasset_path);
    }
    
    let output = cmd.output()?;

    // ALWAYS log stderr for debugging
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        for line in stderr.lines() {
            info!("   [C# Tool] {}", line);
        }
    }

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!("   Tool failed:");
        error!("   stdout: {}", stdout);
        error!("   stderr: {}", stderr);
        return Err(format!("Asset detection failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("   Tool output: {}", stdout);
    
    let result: AssetDetectionResult = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse tool output: {}. Output was: {}", e, stdout))?;
    
    Ok(result.asset_type)
}

/// Fix SerializeSize for Static Meshes using UAssetAPI (calculation only) + binary patching
fn fix_static_mesh_serializesize(uasset_path: &Path, usmap_path: Option<&str>) -> Result<usize, Box<dyn std::error::Error>> {
    let tool_paths = [
        "../../UAssetAPI/StaticMeshSerializeSizeFixer/bin/Release/net8.0/win-x64/StaticMeshSerializeSizeFixer.exe",
        "../../../UAssetAPI/StaticMeshSerializeSizeFixer/bin/Release/net8.0/win-x64/StaticMeshSerializeSizeFixer.exe",
        "./StaticMeshSerializeSizeFixer.exe",
        "./tools/StaticMeshSerializeSizeFixer/StaticMeshSerializeSizeFixer.exe",
    ];

    let mut tool_path = None;
    for path in &tool_paths {
        if std::path::Path::new(path).exists() {
            tool_path = Some(*path);
            break;
        }
    }

    let tool_path = tool_path.ok_or("StaticMeshSerializeSizeFixer.exe not found")?;

    let mut cmd = Command::new(tool_path);
    
    // Hide CMD window on Windows (CREATE_NO_WINDOW flag)
    #[cfg(windows)]
    cmd.creation_flags(0x08000000);
    
    cmd.arg("fix").arg(uasset_path);
    
    // Add usmap path if provided (REQUIRED for unversioned assets)
    if let Some(usmap) = usmap_path {
        cmd.arg(usmap);
    }
    
    let output = cmd.output()?;

    // Always log stderr for debugging
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        for line in stderr.lines() {
            debug!("   [C# Tool] {}", line);
        }
    }

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!("   Fix tool failed:");
        error!("   stdout: {}", stdout);
        return Err(format!("SerializeSize fix failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let result: SerializeSizeFixResult = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse fix tool output: {}. Output was: {}", e, stdout))?;
    
    if !result.success {
        return Ok(0);
    }
    
    let fixed_count = result.fixed_count.unwrap_or(0);
    if fixed_count == 0 {
        return Ok(0);
    }

    // CRITICAL: Apply binary patches at exact byte offsets
    // This preserves file structure that retoc expects (unlike UAssetAPI's Write())
    info!("   Applying binary patches at exact byte offsets...");
    let fixes_json: serde_json::Value = serde_json::from_str(&stdout)?;
    if let Some(fixes_array) = fixes_json.get("fixes").and_then(|f| f.as_array()) {
        for fix in fixes_array {
            let old_size = fix.get("old_size").and_then(|v| v.as_i64()).ok_or("Missing old_size")?;
            let new_size = fix.get("new_size").and_then(|v| v.as_i64()).ok_or("Missing new_size")?;
            apply_binary_patch(uasset_path, old_size, new_size)?;
        }
    }
    
    Ok(fixed_count)
}

/// Apply a binary patch to replace old SerializeSize with new SerializeSize
/// This preserves the exact file structure that retoc expects
fn apply_binary_patch(uasset_path: &Path, old_size: i64, new_size: i64) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{Seek, SeekFrom, Write};
    
    // Read the entire file
    let mut uasset_data = std::fs::read(uasset_path)?;
    
    // Search for the old SerializeSize value in the binary
    let search_bytes = old_size.to_le_bytes();
    let mut found_offset = None;
    
    for i in 0..uasset_data.len().saturating_sub(8) {
        if &uasset_data[i..i+8] == &search_bytes {
            found_offset = Some(i);
            info!("      Found old SerializeSize {} at offset {}", old_size, i);
            break;
        }
    }
    
    let offset = found_offset.ok_or(format!("Could not find old SerializeSize {} in file", old_size))?;
    
    // Patch the 8 bytes at that offset with the new value
    let new_bytes = new_size.to_le_bytes();
    uasset_data[offset..offset+8].copy_from_slice(&new_bytes);
    
    // Write the file back
    std::fs::write(uasset_path, &uasset_data)?;
    
    info!("      Patched SerializeSize: {} → {} at offset {}", old_size, new_size, offset);
    
    Ok(())
}

/// Process all Static Mesh files in a directory and fix their SerializeSize
fn process_static_mesh_serializesize(dir: &Path, usmap_path: Option<&str>) -> Result<usize, Box<dyn std::error::Error>> {
    let mut total_fixed = 0;
    let mut static_mesh_files = Vec::new();

    // Collect all .uasset files
    collect_uasset_files(dir, &mut static_mesh_files)?;

    info!("📁 Found {} .uasset files to check in: {:?}", static_mesh_files.len(), dir);
    
    if let Some(usmap) = usmap_path {
        info!("🗺️  Using USmap file: {}", usmap);
    } else {
        warn!("⚠️  No USmap file provided - detection may fail for unversioned assets!");
    }

    let mut detected_types = std::collections::HashMap::new();

    // Detect and process only Static Meshes using UAssetAPI (NO HEURISTICS!)
    for uasset_file in &static_mesh_files {
        let filename = uasset_file.file_name().unwrap_or_default().to_string_lossy();
        info!("🔍 Checking: {}", filename);
        
        // Use UAssetAPI to accurately detect asset type
        match detect_asset_type_with_uasset_api(uasset_file, usmap_path) {
            Ok(asset_type) => {
                *detected_types.entry(asset_type.clone()).or_insert(0) += 1;
                info!("   → Detected as: {}", asset_type);
                
                // ONLY process if it's a Static Mesh
                // NOT Skeletal Meshes (SK_*)
                // NOT Material Instances (MI_*)
                if asset_type == "static_mesh" {
                    info!("   ✅ Is Static Mesh - Processing...");
                    
                    match fix_static_mesh_serializesize(uasset_file, usmap_path) {
                        Ok(count) => {
                            total_fixed += count;
                            if count > 0 {
                                info!("   ✓ Fixed {} export(s)", count);
                            } else {
                                info!("   → No SerializeSize fixes needed for this mesh");
                            }
                        }
                        Err(e) => {
                            warn!("   ✗ Failed to fix: {}", e);
                        }
                    }
                } else {
                    info!("   ⏭️  Skipping (not a Static Mesh)");
                }
            }
            Err(e) => {
                warn!("   ❌ Could not detect asset type: {}", e);
            }
        }
    }

    // Summary
    info!("📊 Detection Summary:");
    for (asset_type, count) in detected_types.iter() {
        info!("   - {}: {} file(s)", asset_type, count);
    }
    info!("🔧 Total SerializeSize fixes applied: {}", total_fixed);

    Ok(total_fixed)
}

/// Recursively collect all .uasset files in a directory
fn collect_uasset_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                collect_uasset_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("uasset") {
                files.push(path);
            }
        }
    }
    Ok(())
}

