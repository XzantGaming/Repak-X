// Core installation logic without UI dependencies
// Pure business logic that can be used by any UI framework

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use repak::PakReader;
use log::{debug, error, info};

// TODO: Re-enable when install_mod_logic is ported
// pub mod install_mod_logic;
// pub use install_mod_logic::*;

/// Represents a mod that can be installed
#[derive(Debug, Clone)]
pub struct InstallableMod {
    pub mod_name: String,
    pub mod_type: String,
    pub reader: Option<PakReader>,
    pub is_dir: bool,
    pub path: PathBuf,
    pub file_count: usize,
    pub iostore: bool,
    pub custom_tags: Vec<String>,
}

impl PartialEq for InstallableMod {
    fn eq(&self, other: &Self) -> bool {
        // Compare everything except reader (which doesn't implement PartialEq)
        self.mod_name == other.mod_name
            && self.mod_type == other.mod_type
            && self.is_dir == other.is_dir
            && self.path == other.path
            && self.file_count == other.file_count
            && self.iostore == other.iostore
            && self.custom_tags == other.custom_tags
    }
}

/// Installation options
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub fix_mesh: bool,
    pub fix_texture: bool,
    pub to_iostore: bool,
    pub game_path: PathBuf,
    pub usmap_path: Option<PathBuf>,
}

/// Installation progress callback
pub type ProgressCallback = Box<dyn Fn(f32, String) + Send + Sync>;

/// Parse dropped files/folders into installable mods
pub fn parse_dropped_files(
    paths: Vec<PathBuf>,
    game_path: &Path,
    usmap_path: Option<&Path>,
) -> Result<Vec<InstallableMod>, String> {
    let mut mods = Vec::new();
    
    for path in paths {
        if path.is_file() {
            // Handle .pak or .zip files
            if let Some(ext) = path.extension() {
                match ext.to_str() {
                    Some("pak") => {
                        if let Ok(mod_info) = parse_pak_file(&path) {
                            mods.push(mod_info);
                        }
                    }
                    Some("zip") => {
                        if let Ok(mut extracted_mods) = parse_zip_file(&path, game_path) {
                            mods.append(&mut extracted_mods);
                        }
                    }
                    _ => {}
                }
            }
        } else if path.is_dir() {
            // Handle directories
            if let Ok(mut dir_mods) = parse_directory(&path, game_path) {
                mods.append(&mut dir_mods);
            }
        }
    }
    
    if mods.is_empty() {
        return Err("No valid mods found in dropped files".to_string());
    }
    
    Ok(mods)
}

/// Parse a .pak file
fn parse_pak_file(path: &Path) -> Result<InstallableMod, String> {
    // Open file and create reader
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open PAK: {}", e))?;
    let mut reader = std::io::BufReader::new(file);
    
    // Create PAK builder with AES key
    let mut builder = repak::PakBuilder::new();
    // TODO: Get AES key from config
    // builder = builder.key(AES_KEY.clone().0);
    
    // Try to read PAK
    let pak_reader = builder.reader(&mut reader)
        .map_err(|e| format!("Failed to read PAK: {}", e))?;
    
    let file_count = pak_reader.files().len();
    let mod_name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Check for .utoc (IoStore format)
    let mut utoc_path = path.to_path_buf();
    utoc_path.set_extension("utoc");
    let iostore = utoc_path.exists();
    
    // Detect mod type
    let files: Vec<String> = if iostore {
        crate::utoc_utils::read_utoc(&utoc_path, &pak_reader, path)
            .iter()
            .map(|e| e.file_path.clone())
            .collect()
    } else {
        pak_reader.files().to_vec()
    };
    
    let mod_type = crate::utils::get_current_pak_characteristics(files);
    
    Ok(InstallableMod {
        mod_name,
        mod_type,
        reader: Some(pak_reader),
        is_dir: false,
        path: path.to_path_buf(),
        file_count,
        iostore,
        custom_tags: Vec::new(),
    })
}

/// Parse a .zip file
fn parse_zip_file(path: &Path, game_path: &Path) -> Result<Vec<InstallableMod>, String> {
    // Extract zip to temp directory
    let temp_dir = std::env::temp_dir().join(format!("repak_extract_{}", 
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("mod")));
    
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    // Extract zip
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip: {}", e))?;
    
    archive.extract(&temp_dir)
        .map_err(|e| format!("Failed to extract zip: {}", e))?;
    
    // Parse extracted contents
    parse_directory(&temp_dir, game_path)
}

/// Parse a directory
fn parse_directory(path: &Path, game_path: &Path) -> Result<Vec<InstallableMod>, String> {
    let mut mods = Vec::new();
    
    // Look for .pak files in directory
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if ext == "pak" {
                        if let Ok(mod_info) = parse_pak_file(&entry_path) {
                            mods.push(mod_info);
                        }
                    }
                }
            }
        }
    }
    
    // If no .pak files found, treat directory as loose files mod
    if mods.is_empty() {
        let mut files = Vec::new();
        collect_files_recursive(path, &mut files)?;
        
        if !files.is_empty() {
            let mod_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let file_paths: Vec<String> = files.iter()
                .filter_map(|p| p.to_str().map(|s| s.to_string()))
                .collect();
            
            let mod_type = crate::utils::get_current_pak_characteristics(file_paths);
            
            mods.push(InstallableMod {
                mod_name,
                mod_type,
                reader: None,
                is_dir: true,
                path: path.to_path_buf(),
                file_count: files.len(),
                iostore: false,
                custom_tags: Vec::new(),
            });
        }
    }
    
    Ok(mods)
}

/// Recursively collect files
fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// Install mods with progress callback
pub async fn install_mods(
    mods: Vec<InstallableMod>,
    options: InstallOptions,
    progress_callback: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let total_mods = mods.len() as f32;
    
    for (index, mod_info) in mods.iter().enumerate() {
        // Check if cancelled
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Installation cancelled".to_string());
        }
        
        // Update progress
        let progress = (index as f32) / total_mods;
        progress_callback(progress, format!("Installing {}...", mod_info.mod_name));
        
        // Install the mod
        install_single_mod(mod_info, &options).await?;
    }
    
    progress_callback(1.0, "Installation complete!".to_string());
    Ok(())
}

/// Install a single mod
async fn install_single_mod(
    mod_info: &InstallableMod,
    options: &InstallOptions,
) -> Result<(), String> {
    info!("Installing mod: {}", mod_info.mod_name);
    
    let target_path = options.game_path.join(format!("{}_P.pak", mod_info.mod_name));
    
    if mod_info.is_dir {
        // Install from directory
        install_from_directory(&mod_info.path, &target_path, options).await?;
    } else {
        // Install from PAK
        install_from_pak(&mod_info.path, &target_path, options).await?;
    }
    
    info!("Successfully installed: {}", mod_info.mod_name);
    Ok(())
}

/// Install from directory
async fn install_from_directory(
    source_dir: &Path,
    _target_path: &Path,
    _options: &InstallOptions,
) -> Result<(), String> {
    // Collect all files
    let mut files = Vec::new();
    collect_files_recursive(source_dir, &mut files)?;
    
    // Create PAK from files
    // TODO: Implement actual PAK creation with options
    info!("Would create PAK from {} files", files.len());
    
    Ok(())
}

/// Install from PAK
async fn install_from_pak(
    source_pak: &Path,
    _target_path: &Path,
    _options: &InstallOptions,
) -> Result<(), String> {
    // Copy or recompress PAK
    // TODO: Implement actual PAK processing with options
    info!("Would process PAK: {:?}", source_pak);
    
    Ok(())
}
