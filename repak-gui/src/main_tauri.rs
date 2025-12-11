// Tauri-based main.rs - React + Tauri implementation
// Original egui implementation backed up in src/egui_backup_original/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod install_mod;
mod uasset_detection;
mod uasset_api_integration;
mod utils;
mod utoc_utils;
mod character_data;
mod crash_monitor;
mod p2p_sharing;
mod p2p_libp2p;
mod p2p_manager;
mod p2p_security;
mod p2p_stream;
mod p2p_protocol;
mod ip_obfuscation;

use uasset_detection::{detect_mesh_files_async, detect_texture_files_async, detect_static_mesh_files_async};
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Window};
use utils::find_marvel_rivals;
use walkdir::WalkDir;
use regex_lite::Regex;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

// ============================================================================
// STATE MANAGEMENT
// ============================================================================

struct WatcherState {
    watcher: Mutex<Option<RecommendedWatcher>>,
    #[allow(dead_code)]
    last_event_time: Mutex<std::time::Instant>,
}

/// P2P Sharing state management
struct P2PState {
    manager: Arc<p2p_manager::UnifiedP2PManager>,
}

/// Crash monitoring state
struct CrashMonitorState {
    game_start_time: Mutex<Option<std::time::SystemTime>>,
    last_checked_crash: Mutex<Option<std::time::SystemTime>>,
}

#[derive(Default, Serialize, Deserialize)]
struct AppState {
    game_path: PathBuf,
    folders: Vec<ModFolder>,
    mod_metadata: Vec<ModMetadata>,
    usmap_path: String,
    auto_check_updates: bool,
    hide_internal_suffix: bool,
    custom_tag_catalog: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ModFolder {
    id: String,
    name: String,
    enabled: bool,
    expanded: bool,
    color: Option<[u8; 3]>,
    /// Depth in folder hierarchy (0 = root, 1 = direct child, etc.)
    #[serde(default)]
    depth: usize,
    /// Parent folder ID (None = root folder, "_root" for root's direct children)
    #[serde(default)]
    parent_id: Option<String>,
    /// Is this the root folder (the ~mods directory itself)
    #[serde(default)]
    is_root: bool,
    /// Number of mods directly in this folder
    #[serde(default)]
    mod_count: usize,
}

/// Root folder info for hierarchy display
#[derive(Clone, Serialize, Deserialize)]
struct RootFolderInfo {
    /// The actual folder name (e.g., "~mods")
    name: String,
    /// Full path to the root folder
    path: String,
    /// Total number of mods in root (not in subfolders)
    direct_mod_count: usize,
    /// Total number of subfolders
    subfolder_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct ModMetadata {
    path: PathBuf,
    custom_name: Option<String>,
    folder_id: Option<String>,
    #[serde(default)]
    custom_tags: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ModEntry {
    path: PathBuf,
    enabled: bool,
    custom_name: Option<String>,
    folder_id: Option<String>,
    custom_tags: Vec<String>,
    file_size: u64,
    priority: usize,
    // Character/skin info from character_data (dynamically looked up)
    character_name: Option<String>,
    skin_name: Option<String>,
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
async fn get_game_path(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().unwrap();
    Ok(state.game_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn set_game_path(path: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    state.game_path = PathBuf::from(path);
    save_state(&state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn auto_detect_game_path(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    match find_marvel_rivals() {
        Some(game_root) => {
            // game_path should be the ~mods directory (matching egui behavior)
            let mods_path = game_root.join("~mods");
            
            // Create ~mods directory if it doesn't exist
            if !mods_path.exists() {
                std::fs::create_dir_all(&mods_path)
                    .map_err(|e| format!("Failed to create ~mods directory: {}", e))?;
            }
            
            let mut state = state.lock().unwrap();
            state.game_path = mods_path.clone();
            save_state(&state).map_err(|e| e.to_string())?;
            Ok(mods_path.to_string_lossy().to_string())
        }
        None => Err("Could not auto-detect Marvel Rivals installation".to_string()),
    }
}

#[tauri::command]
async fn start_file_watcher(
    window: Window,
    state: State<'_, Arc<Mutex<AppState>>>,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    let state_guard = state.lock().unwrap();
    let game_path = state_guard.game_path.clone();
    drop(state_guard);

    if !game_path.exists() {
        return Ok(()); // Can't watch non-existent path
    }

    let mut watcher_guard = watcher_state.watcher.lock().unwrap();
    
    // Create a new watcher with debouncing
    let window_clone = window.clone();
    let last_event_time = Arc::new(Mutex::new(std::time::Instant::now()));
    
    let watcher_result = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                // We only care about Create, Remove, Rename, and Modify events (files and directories)
                match event.kind {
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                         // Debounce: only emit if 500ms have passed since last event
                         let mut last_time = last_event_time.lock().unwrap();
                         let now = std::time::Instant::now();
                         let elapsed = now.duration_since(*last_time);
                         
                         if elapsed.as_millis() >= 500 {
                             *last_time = now;
                             window_clone.emit("mods_dir_changed", ()).unwrap_or_else(|e| {
                                 error!("Failed to emit mods_dir_changed: {}", e);
                             });
                         }
                    },
                    _ => {}
                }
            },
            Err(e) => error!("Watch error: {:?}", e),
        }
    });

    match watcher_result {
        Ok(mut watcher) => {
            if let Err(e) = watcher.watch(&game_path, RecursiveMode::Recursive) {
                error!("Failed to watch game path: {}", e);
                return Err(e.to_string());
            }
            info!("Started watching game path: {:?}", game_path);
            *watcher_guard = Some(watcher);
            Ok(())
        },
        Err(e) => {
            error!("Failed to create watcher: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_pak_files(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<ModEntry>, String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    info!("Loading mods from: {}", game_path.display());
    
    if !game_path.exists() {
        info!("Game path does not exist: {}", game_path.display());
        return Err(format!("Game path does not exist: {}", game_path.display()));
    }

    // game_path IS the ~mods directory (matching egui behavior)
    let mut mods = Vec::new();
    
    // Scan root ~mods directory and all subdirectories recursively (no depth limit)
    for entry in WalkDir::new(&game_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip directories themselves
        if path.is_dir() {
            continue;
        }
        
        let ext = path.extension().and_then(|s| s.to_str());
        
        // Check for .pak, .bak_repak, and .pak_disabled files
        if ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled") {
            let is_enabled = ext == Some("pak");
            
            // Determine which folder this mod is in
            let root_folder_name = game_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("~mods")
                .to_string();
            
            // Determine folder_id based on relative path from game_path
            let folder_id = if let Some(parent) = path.parent() {
                if parent == game_path {
                    // Mod is directly in root - use root folder name (e.g., "~mods")
                    Some(root_folder_name)
                } else {
                    // Mod is in a subfolder - use relative path from game_path as ID
                    parent.strip_prefix(game_path)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .ok()
                }
            } else {
                Some(root_folder_name)
            };
            
            info!("Found PAK file: {} (enabled: {}, folder: {:?})", path.display(), is_enabled, folder_id);
            
            let metadata = state.mod_metadata.iter()
                .find(|m| {
                    m.path == path || 
                    m.path.with_extension("pak") == path || 
                    m.path.with_extension("bak_repak") == path ||
                    m.path.with_extension("pak_disabled") == path
                });
            
            let ucas_path = path.with_extension("ucas");
            let file_size = if ucas_path.exists() {
                std::fs::metadata(&ucas_path)
                    .map(|m| m.len())
                    .unwrap_or(0)
            } else {
                std::fs::metadata(path)
                    .map(|m| m.len())
                    .unwrap_or(0)
            };
            
            // Calculate priority
            // Priority 0 = "!" prefix (highest priority)
            // Priority 1-N = 7-N+6 nines displayed as 1-based (7 nines → Priority 1, 8 nines → Priority 2, etc.)
            let mut priority = 0;
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            
            // Check for "!" prefix (highest priority)
            if file_stem.starts_with("!") {
                priority = 0; // Highest priority
            } else if file_stem.ends_with("_P") {
                let base_no_p = file_stem.strip_suffix("_P").unwrap();
                // Check for _999... suffix
                let re_nums = Regex::new(r"_(\d+)$").unwrap();
                if let Some(caps) = re_nums.captures(base_no_p) {
                    let nums = &caps[1];
                    // Verify they are all 9s
                    if nums.chars().all(|c| c == '9') {
                        let actual_nines = nums.len();
                        // Convert actual nines count to UI priority (1-based)
                        // 7 nines → Priority 1, 8 nines → Priority 2, etc.
                        if actual_nines >= 7 {
                            priority = actual_nines - 6;
                        }
                    }
                }
            }
            
            mods.push(ModEntry {
                path: path.to_path_buf(),
                enabled: is_enabled,
                custom_name: metadata.and_then(|m| m.custom_name.clone()),
                folder_id,
                custom_tags: metadata.map(|m| m.custom_tags.clone()).unwrap_or_default(),
                file_size,
                priority,
                character_name: None,
                skin_name: None,
            });
        }
    }

    info!("Found {} mod(s)", mods.len());
    Ok(mods)
}

#[tauri::command]
async fn set_mod_priority(mod_path: String, priority: usize) -> Result<(), String> {
    let path = PathBuf::from(&mod_path);
    if !path.exists() {
         return Err("Mod file does not exist".to_string());
    }
    
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let stem = path.file_stem().and_then(|s| s.to_str()).ok_or("Invalid filename")?;
    
    // Strip leading "!" if present (highest priority marker)
    let stem_no_exclaim = stem.strip_prefix("!").unwrap_or(stem);
    
    // 1. Strip _P if present
    let base_no_p = if stem_no_exclaim.ends_with("_P") {
        stem_no_exclaim.strip_suffix("_P").unwrap()
    } else {
        stem_no_exclaim
    };
    
    // 2. Strip _999... if present
    let re = Regex::new(r"^(.*)_(\d+)$").unwrap();
    let clean_base = if let Some(caps) = re.captures(base_no_p) {
        let prefix = &caps[1];
        let numbers = &caps[2];
        if numbers.chars().all(|c| c == '9') {
            prefix.to_string()
        } else {
            base_no_p.to_string()
        }
    } else {
        base_no_p.to_string()
    };
    
    // 3. Construct new name with new priority
    // Priority 0 = "!" prefix (highest priority) with minimum 7 nines
    // Priority 1-N = 7-N+6 nines (1→7 nines, 2→8 nines, etc.)
    let new_stem = if priority == 0 {
        // Highest priority: use "!" prefix with minimum 7 nines
        let min_nines = "9".repeat(7);
        format!("!{}_{}_P", clean_base, min_nines)
    } else {
        // Convert UI priority (1-based) to actual nines count (7-based)
        // Remove "!" prefix if present (since priority > 0)
        let actual_nines = priority + 6; // Priority 1 → 7 nines, Priority 2 → 8 nines, etc.
        let new_nines = "9".repeat(actual_nines);
        format!("{}_{}_P", clean_base, new_nines)
    };
    
    let new_filename = format!("{}.{}", new_stem, extension);
    let new_path = path.with_file_name(&new_filename);
    
    if new_path == path {
        return Ok(()); // No change
    }

    if new_path.exists() {
        return Err("A mod with this priority already exists".to_string());
    }
    
    // Rename main file
    std::fs::rename(&path, &new_path).map_err(|e| format!("Failed to rename mod: {}", e))?;
    
    // Rename associated files (.utoc, .ucas)
    let exts = ["utoc", "ucas"];
    for ext in exts {
        let old_f = path.with_extension(ext);
        if old_f.exists() {
             let new_f = new_path.with_extension(ext);
             let _ = std::fs::rename(old_f, new_f);
        }
    }
    
    Ok(())
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct InstallableModInfo {
    mod_name: String,
    mod_type: String,
    is_dir: bool,
    path: String,
    auto_fix_mesh: bool,
    auto_fix_texture: bool,
    auto_fix_serialize_size: bool,
    auto_to_repak: bool,
}

#[tauri::command]
async fn parse_dropped_files(
    paths: Vec<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
    window: Window
) -> Result<Vec<InstallableModInfo>, String> {
    use crate::utils::get_current_pak_characteristics;
    use repak::PakBuilder;
    use repak::utils::AesKey;
    use std::str::FromStr;
    use std::fs::File;
    use std::io::BufReader;
    
    // Emit start detection log
    let _ = window.emit("install_log", "[Detection] Starting UAssetAPI detection...");
    
    // Set USMAP_PATH for detection (from roaming folder)
    {
        let state_guard = state.lock().unwrap();
        let usmap_filename = state_guard.usmap_path.clone();
        
        if !usmap_filename.is_empty() {
            if let Some(usmap_full_path) = get_usmap_full_path(&usmap_filename) {
                std::env::set_var("USMAP_PATH", &usmap_full_path);
                let msg = format!("[Detection] Set USMAP_PATH: {}", usmap_full_path.display());
                info!("{}", msg);
                let _ = window.emit("install_log", &msg);
            } else {
                let expected_path = usmap_dir().join(&usmap_filename);
                let msg = format!("[Detection] WARNING: USMAP not found at: {}", expected_path.display());
                info!("{}", msg);
                let _ = window.emit("install_log", &msg);
            }
        } else {
            let _ = window.emit("install_log", "[Detection] WARNING: No USMAP configured in settings");
        }
    }
    
    let mut mods = Vec::new();
    
    // Filter out .utoc and .ucas files - they will be handled with their .pak file
    let filtered_paths: Vec<String> = paths.into_iter()
        .filter(|p| {
            let path = PathBuf::from(p);
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                ext != "utoc" && ext != "ucas"
            } else {
                true
            }
        })
        .collect();
    
    for path_str in filtered_paths {
        let path = PathBuf::from(&path_str);
        
        if !path.exists() {
            continue;
        }
        
        let mod_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        // Determine mod type and auto-detection flags
        let (mod_type, auto_fix_mesh, auto_fix_texture, auto_fix_serialize_size) = if path.is_dir() {
            // First check if directory contains multiple PAK files - if so, process each PAK separately
            use walkdir::WalkDir;
            let mut pak_files = Vec::new();
            
            for entry in WalkDir::new(&path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if let Some(ext) = entry_path.extension().and_then(|s| s.to_str()) {
                    if ext == "pak" {
                        pak_files.push(entry_path.to_path_buf());
                    }
                }
            }
            
            if pak_files.len() > 1 {
                // Multiple PAK files - process each separately
                let _ = window.emit("install_log", format!("[Detection] Found {} PAK files in directory, processing each separately", pak_files.len()));
                
                for pak_file in pak_files {
                    let pak_mods = Box::pin(parse_dropped_files(vec![pak_file.to_string_lossy().to_string()], state.clone(), window.clone())).await?;
                    for pak_mod in pak_mods {
                        mods.push(pak_mod);
                    }
                }
                
                return Ok(mods);
            } else if pak_files.len() == 1 {
                // Single PAK file in directory - process it directly (handles IoStore if present)
                let pak_file = &pak_files[0];
                let _ = window.emit("install_log", format!("[Detection] Found single PAK in directory: {}", pak_file.display()));
                
                let pak_mods = Box::pin(parse_dropped_files(vec![pak_file.to_string_lossy().to_string()], state.clone(), window.clone())).await?;
                for pak_mod in pak_mods {
                    mods.push(pak_mod);
                }
                
                return Ok(mods);
            }
            
            // No PAK files - analyze directory contents for loose assets
            let _ = window.emit("install_log", "[Detection] No PAK files found, analyzing directory for loose assets...");
            
            use crate::utils::collect_files;
            let mut all_files = Vec::new();
            if collect_files(&mut all_files, &path).is_ok() {
                let _ = window.emit("install_log", format!("[Detection] Collected {} files from directory", all_files.len()));
                
                // Convert absolute paths to relative paths for proper classification
                // Strip the base directory path to get relative paths
                let content_files_relative: Vec<String> = all_files.iter()
                    .filter_map(|p| {
                        p.strip_prefix(&path).ok()
                            .map(|rel| rel.to_string_lossy().to_string().replace('\\', "/"))
                    })
                    .collect();
                
                if !content_files_relative.is_empty() {
                    let _ = window.emit("install_log", format!("[Detection] Processing {} files for classification", content_files_relative.len()));
                    
                    // Use detailed characteristics for proper classification (needs relative paths)
                    use crate::utils::get_pak_characteristics_detailed;
                    let characteristics = get_pak_characteristics_detailed(content_files_relative.clone());
                    let mod_type = characteristics.mod_type.clone();
                    
                    let _ = window.emit("install_log", format!("[Detection] Detected mod type: {}", mod_type));
                    
                    // Get uasset files for detection (needs absolute paths for UAssetAPI to read files)
                    // Prioritize skeletal mesh files (SK_*), static mesh (SM_*), and textures (T_*) over materials
                    // Limit to 100 total to prevent hangs on large directories
                    let mut uasset_files_absolute: Vec<String> = Vec::new();
                    let mut priority_files: Vec<String> = Vec::new();
                    let mut other_files: Vec<String> = Vec::new();
                    
                    for file in all_files.iter() {
                        if file.extension().and_then(|s| s.to_str()) == Some("uasset") {
                            let filename = file.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            let filename_lower = filename.to_lowercase();
                            
                            // Prioritize SK_, SM_, T_ files (skeletal mesh, static mesh, textures)
                            if filename_lower.starts_with("sk_") || filename_lower.starts_with("sm_") || filename_lower.starts_with("t_") {
                                priority_files.push(file.to_string_lossy().to_string());
                            } else {
                                other_files.push(file.to_string_lossy().to_string());
                            }
                        }
                    }
                    
                    // Add priority files first, then fill up to 100 with other files
                    uasset_files_absolute.extend(priority_files);
                    let remaining = 100usize.saturating_sub(uasset_files_absolute.len());
                    uasset_files_absolute.extend(other_files.into_iter().take(remaining));
                    
                    let _ = window.emit("install_log", format!("[Detection] Checking {} uasset files for asset types...", uasset_files_absolute.len()));
                    
                    // Log first few files for debugging
                    if !uasset_files_absolute.is_empty() {
                        let sample_count = uasset_files_absolute.len().min(3);
                        for (i, file) in uasset_files_absolute.iter().take(sample_count).enumerate() {
                            let _ = window.emit("install_log", format!("[Detection] Sample file {}: {}", i + 1, file));
                        }
                    }
                    
                    let has_skeletal_mesh = detect_mesh_files_async(&uasset_files_absolute).await;
                    let _ = window.emit("install_log", format!("[Detection] SkeletalMesh result: {}", has_skeletal_mesh));
                    
                    let has_static_mesh = detect_static_mesh_files_async(&uasset_files_absolute).await;
                    let _ = window.emit("install_log", format!("[Detection] StaticMesh result: {}", has_static_mesh));
                    
                    let has_texture = detect_texture_files_async(&uasset_files_absolute).await;
                    let _ = window.emit("install_log", format!("[Detection] Texture result: {}", has_texture));
                    
                    let summary = format!("[Detection] Directory results: skeletal={}, static={}, texture={}", 
                        has_skeletal_mesh, has_static_mesh, has_texture);
                    info!("{}", summary);
                    let _ = window.emit("install_log", &summary);
                    
                    (mod_type, has_skeletal_mesh, has_texture, has_static_mesh)
                } else {
                    ("Directory".to_string(), false, false, false)
                }
            } else {
                ("Directory".to_string(), false, false, false)
            }
        } else {
            // Get file extension
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            
            // Check if it's an archive file (zip, rar, 7z)
            if ext == "zip" || ext == "rar" || ext == "7z" {
                use crate::install_mod::install_mod_logic::archives::{extract_zip, extract_rar, extract_7z};
                use walkdir::WalkDir;
                
                let _ = window.emit("install_log", format!("[Detection] Archive detected: {} ({})", mod_name, ext));
                
                // Extract archive to temp directory for analysis
                let temp_dir = tempfile::tempdir().ok();
                if let Some(ref temp) = temp_dir {
                    let temp_path = temp.path().to_str().unwrap();
                    
                    // Extract based on type
                    let extract_result = if ext == "zip" {
                        extract_zip(path.to_str().unwrap(), temp_path)
                    } else if ext == "rar" {
                        extract_rar(path.to_str().unwrap(), temp_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                    } else {
                        extract_7z(path.to_str().unwrap(), temp_path)
                    };
                    
                    if extract_result.is_ok() {
                        let _ = window.emit("install_log", "[Detection] Archive extracted successfully");
                        
                        // Look for PAK files in extracted content
                        let mut pak_files_in_archive = Vec::new();
                        for entry in WalkDir::new(temp_path).into_iter().filter_map(|e| e.ok()) {
                            let entry_path = entry.path();
                            if let Some(entry_ext) = entry_path.extension().and_then(|s| s.to_str()) {
                                if entry_ext == "pak" {
                                    pak_files_in_archive.push(entry_path.to_path_buf());
                                }
                            }
                        }
                        
                        if pak_files_in_archive.len() > 1 {
                            // Multiple PAK files found in archive
                            let _ = window.emit("install_log", format!("[Detection] Found {} PAK files in archive, processing each separately", pak_files_in_archive.len()));
                            
                            for pak_file_path in pak_files_in_archive {
                                let pak_mods = Box::pin(parse_dropped_files(vec![pak_file_path.to_string_lossy().to_string()], state.clone(), window.clone())).await?;
                                for pak_mod in pak_mods {
                                    mods.push(pak_mod);
                                }
                            }
                            
                            return Ok(mods);
                        }
                        
                        // Single PAK file or no PAK files - continue with existing logic
                        let found_pak = !pak_files_in_archive.is_empty();
                        if found_pak {
                            let entry_path = &pak_files_in_archive[0];
                            
                            // Check if this is an IoStore package (has .utoc and .ucas companions)
                            let utoc_path = entry_path.with_extension("utoc");
                            let ucas_path = entry_path.with_extension("ucas");
                            let is_iostore = utoc_path.exists() && ucas_path.exists();
                            
                            if is_iostore {
                                let _ = window.emit("install_log", format!("[Detection] IoStore package detected in archive: {}", mod_name));
                            }
                            
                            // Found a single pak file, analyze it
                            if let Ok(file) = File::open(entry_path) {
                                if let Ok(aes_key) = AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74") {
                                    let mut reader = BufReader::new(file);
                                    if let Ok(pak) = PakBuilder::new().key(aes_key.0).reader(&mut reader) {
                                        // Get file list - for IoStore, read from utoc; otherwise use pak.files()
                                        let files: Vec<String> = if is_iostore {
                                            use crate::utoc_utils::read_utoc;
                                            let _ = window.emit("install_log", "[Detection] Reading IoStore .utoc file for accurate file list");
                                            read_utoc(&utoc_path, &pak, entry_path)
                                                .iter()
                                                .map(|entry| entry.file_path.clone())
                                                .collect()
                                        } else {
                                            pak.files()
                                        };
                                        
                                        // Use detailed characteristics (same as get_mod_details)
                                        use crate::utils::get_pak_characteristics_detailed;
                                        let characteristics = get_pak_characteristics_detailed(files.clone());
                                        let mod_type = characteristics.mod_type.clone();
                                        
                                        let _ = window.emit("install_log", format!("[Detection] Detected mod type: {}", mod_type));
                                        
                                        // Get uasset files for extraction (also extract matching .uexp files)
                                        let uasset_files: Vec<String> = files.iter()
                                            .filter(|f| f.ends_with(".uasset"))
                                            .cloned()
                                            .collect();
                                        
                                        let has_skeletal_mesh = detect_mesh_files_async(&uasset_files).await;
                                        let _ = window.emit("install_log", format!("[Detection] SkeletalMesh result: {}", has_skeletal_mesh));
                                        
                                        let has_static_mesh = detect_static_mesh_files_async(&uasset_files).await;
                                        let _ = window.emit("install_log", format!("[Detection] StaticMesh result: {}", has_static_mesh));
                                        
                                        let has_texture = detect_texture_files_async(&uasset_files).await;
                                        let _ = window.emit("install_log", format!("[Detection] Texture result: {}", has_texture));
                                        
                                        let summary = format!("[Detection] Archive PAK results: skeletal={}, static={}, texture={}", 
                                            has_skeletal_mesh, has_static_mesh, has_texture);
                                        info!("{}", summary);
                                        let _ = window.emit("install_log", &summary);
                                        
                                        // Clean up temp dir
                                        drop(temp_dir);
                                        
                                        return Ok(vec![InstallableModInfo {
                                            mod_name,
                                            mod_type,
                                            is_dir: false,
                                            path: path_str,
                                            auto_fix_mesh: has_skeletal_mesh,
                                            auto_fix_texture: has_texture,
                                            auto_fix_serialize_size: has_static_mesh,
                                            auto_to_repak: !is_iostore,  // Don't repak IoStore packages
                                        }]);
                                    }
                                }
                            }
                        }
                        
                        // If no .pak files found, look for content folders with loose assets
                        if !found_pak {
                            let _ = window.emit("install_log", "[Detection] No PAK files found in archive, looking for content folders...");
                            
                            use crate::utils::collect_files;
                            
                            // Collect all files from the extracted archive
                            let mut all_files = Vec::new();
                            let temp_path_buf = PathBuf::from(temp_path);
                            if collect_files(&mut all_files, &temp_path_buf).is_ok() {
                                // Check if there are content files (.uasset, .uexp, .ubulk, etc.)
                                let content_files: Vec<String> = all_files.iter()
                                    .filter(|f| {
                                        if let Some(ext) = f.extension().and_then(|s| s.to_str()) {
                                            matches!(ext, "uasset" | "uexp" | "ubulk" | "bnk" | "wem")
                                        } else {
                                            false
                                        }
                                    })
                                    .map(|p| p.to_string_lossy().to_string())
                                    .collect();
                                
                                if !content_files.is_empty() {
                                    let _ = window.emit("install_log", format!("[Detection] Found {} content files in archive folder", content_files.len()));
                                    
                                    // Get mod type from content
                                    let mod_type = get_current_pak_characteristics(content_files.clone());
                                    
                                    // Run UAsset detection on the content files
                                    let _ = window.emit("install_log", "[Detection] Checking for SkeletalMesh assets...");
                                    let has_skeletal_mesh = detect_mesh_files_async(&content_files).await;
                                    let _ = window.emit("install_log", format!("[Detection] SkeletalMesh result: {}", has_skeletal_mesh));
                                    
                                    let _ = window.emit("install_log", "[Detection] Checking for StaticMesh assets...");
                                    let has_static_mesh = detect_static_mesh_files_async(&content_files).await;
                                    let _ = window.emit("install_log", format!("[Detection] StaticMesh result: {}", has_static_mesh));
                                    
                                    let _ = window.emit("install_log", "[Detection] Checking for Texture assets with .ubulk...");
                                    let has_texture = detect_texture_files_async(&content_files).await;
                                    let _ = window.emit("install_log", format!("[Detection] Texture result: {}", has_texture));
                                    
                                    let summary = format!("[Detection] Archive folder results: skeletal={}, static={}, texture={}", 
                                        has_skeletal_mesh, has_static_mesh, has_texture);
                                    info!("{}", summary);
                                    let _ = window.emit("install_log", &summary);
                                    
                                    // Clean up temp dir
                                    drop(temp_dir);
                                    
                                    // Return as a directory mod (will be converted to IoStore)
                                    return Ok(vec![InstallableModInfo {
                                        mod_name,
                                        mod_type,
                                        is_dir: true,
                                        path: path_str,
                                        auto_fix_mesh: has_skeletal_mesh,
                                        auto_fix_texture: has_texture,
                                        auto_fix_serialize_size: has_static_mesh,
                                        auto_to_repak: false,
                                    }]);
                                }
                            }
                        }
                    }
                }
                
                // Fallback if extraction/analysis failed
                ("Archive".to_string(), false, false, false)
            } else if ext == "pak" {
                // Check if this is an IoStore package (has .utoc and .ucas companions)
                let utoc_path = path.with_extension("utoc");
                let ucas_path = path.with_extension("ucas");
                let is_iostore = utoc_path.exists() && ucas_path.exists();
                
                if is_iostore {
                    let _ = window.emit("install_log", format!("[Detection] IoStore package detected: {}", mod_name));
                }
                
                // Read file list for mod type detection
                let mod_type = if let Ok(file) = File::open(&path) {
                    if let Ok(aes_key) = AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74") {
                        let aes_key_for_extraction = aes_key.clone();
                        let mut reader = BufReader::new(file);
                        if let Ok(pak) = PakBuilder::new().key(aes_key.0).reader(&mut reader) {
                            // Get file list - for IoStore, read from utoc; otherwise use pak.files()
                            let files: Vec<String> = if is_iostore {
                                use crate::utoc_utils::read_utoc;
                                let _ = window.emit("install_log", "[Detection] Reading IoStore .utoc file for accurate file list");
                                read_utoc(&utoc_path, &pak, &path)
                                    .iter()
                                    .map(|entry| entry.file_path.clone())
                                    .collect()
                            } else {
                                pak.files()
                            };
                            
                            // Use detailed characteristics (same as get_mod_details)
                            use crate::utils::get_pak_characteristics_detailed;
                            let characteristics = get_pak_characteristics_detailed(files.clone());
                            let mod_type = characteristics.mod_type.clone();
                            
                            let _ = window.emit("install_log", format!("[Detection] Detected mod type: {}", mod_type));
                            
                            // Get files to extract (both .uasset and .uexp needed by UAssetAPI)
                            let files_to_extract: Vec<&String> = files.iter()
                                .filter(|f| {
                                    let lower = f.to_lowercase();
                                    (lower.ends_with(".uasset") || lower.ends_with(".uexp")) &&
                                    // Prioritize SK_, SM_, T_ files
                                    if let Some(filename) = std::path::Path::new(f).file_name().and_then(|n| n.to_str()) {
                                        let fname_lower = filename.to_lowercase();
                                        fname_lower.starts_with("sk_") || fname_lower.starts_with("sm_") || fname_lower.starts_with("t_")
                                    } else {
                                        false
                                    }
                                })
                                .take(40)  // Limit to 40 files (20 uasset + 20 uexp pairs)
                                .collect();
                            
                            let _ = window.emit("install_log", format!("[Detection] Extracting {} files from PAK for analysis...", files_to_extract.len()));
                            
                            // Extract to temp directory
                            let mut extracted_paths: Vec<String> = Vec::new();
                            let uasset_temp_dir = tempfile::tempdir().ok();
                            
                            if let Some(ref uasset_temp) = uasset_temp_dir {
                                use rayon::prelude::*;
                                use std::sync::Mutex;
                                
                                let extracted = Mutex::new(Vec::new());
                                let pak_path = path.clone();
                                
                                // Parallel extraction
                                files_to_extract.par_iter().for_each(|internal_path| {
                                    // Each thread opens its own file handle
                                    if let Ok(file) = File::open(&pak_path) {
                                        let mut reader = BufReader::new(file);
                                        if let Ok(pak) = PakBuilder::new().key(aes_key_for_extraction.0.clone()).reader(&mut reader) {
                                            // Sanitize filename for filesystem
                                            let safe_name = internal_path.replace("/", "_").replace("\\", "_");
                                            let dest_path = uasset_temp.path().join(&safe_name);
                                            
                                            // Re-open file for extraction (pak.get needs mutable reader)
                                            if let Ok(extract_file) = File::open(&pak_path) {
                                                let mut extract_reader = BufReader::new(extract_file);
                                                // Extract file
                                                if let Ok(data) = pak.get(internal_path, &mut extract_reader) {
                                                    if let Ok(_) = std::fs::write(&dest_path, data) {
                                                        if internal_path.to_lowercase().ends_with(".uasset") {
                                                            extracted.lock().unwrap().push(dest_path.to_string_lossy().to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });
                                
                                extracted_paths = extracted.into_inner().unwrap();
                                let _ = window.emit("install_log", format!("[Detection] Extracted {} uasset files for UAssetAPI", extracted_paths.len()));
                            }
                            
                            // Use extracted paths if available, otherwise fall back to internal paths
                            let detection_files = if !extracted_paths.is_empty() {
                                extracted_paths.clone()
                            } else {
                                files.iter().filter(|f| f.ends_with(".uasset")).cloned().collect()
                            };
                            
                            let has_skeletal_mesh = detect_mesh_files_async(&detection_files).await;
                            let _ = window.emit("install_log", format!("[Detection] SkeletalMesh result: {}", has_skeletal_mesh));
                            
                            let has_static_mesh = detect_static_mesh_files_async(&detection_files).await;
                            let _ = window.emit("install_log", format!("[Detection] StaticMesh result: {}", has_static_mesh));
                            
                            let has_texture = detect_texture_files_async(&detection_files).await;
                            let _ = window.emit("install_log", format!("[Detection] Texture result: {}", has_texture));
                            
                            let summary = format!("[Detection] PAK file results: skeletal={}, static={}, texture={}", 
                                has_skeletal_mesh, has_static_mesh, has_texture);
                            info!("{}", summary);
                            let _ = window.emit("install_log", &summary);
                            
                            return Ok(vec![InstallableModInfo {
                                mod_name,
                                mod_type,
                                is_dir: false,
                                path: path_str,
                                auto_fix_mesh: has_skeletal_mesh,
                                auto_fix_texture: has_texture,
                                auto_fix_serialize_size: has_static_mesh,
                                auto_to_repak: !is_iostore,  // Don't repak IoStore packages
                            }]);
                        }
                    }
                    
                    "PAK".to_string()
                } else {
                    "Unknown".to_string()
                };
                
                (mod_type, false, false, false)
            } else {
                ("Unknown".to_string(), false, false, false)
            }
        };

        // For .pak files, auto-enable repak UNLESS it's an IoStore package
        let is_pak = path.extension().and_then(|s| s.to_str()) == Some("pak");
        let is_iostore_pkg = is_pak && path.with_extension("utoc").exists() && path.with_extension("ucas").exists();
        let auto_to_repak = is_pak && !is_iostore_pkg;

        mods.push(InstallableModInfo {
            mod_name,
            mod_type,
            is_dir: path.is_dir(),
            path: path_str,
            auto_fix_mesh,
            auto_fix_texture,
            auto_fix_serialize_size,
            auto_to_repak,
        });
    }

    Ok(mods)
}

#[derive(serde::Deserialize)]
struct ModToInstall {
    path: String,
    #[serde(rename = "customName")]
    custom_name: Option<String>,
    #[serde(rename = "fixMesh")]
    fix_mesh: bool,
    #[serde(rename = "fixTexture")]
    fix_texture: bool,
    #[serde(rename = "fixSerializeSize")]
    fix_serialize_size: bool,
    #[serde(rename = "toRepak")]
    to_repak: bool,
}

/// Quick Organize: Simply copy/move files to a target folder without any repak processing
/// This is for organizing existing mod files into subfolders
#[tauri::command]
async fn quick_organize(
    paths: Vec<String>,
    target_folder: String,
    state: State<'_, Arc<Mutex<AppState>>>,
    window: Window,
) -> Result<i32, String> {
    use crate::install_mod::install_mod_logic::archives::{extract_zip, extract_rar, extract_7z};
    use walkdir::WalkDir;
    
    let state_guard = state.lock().unwrap();
    let mod_directory = state_guard.game_path.clone();
    drop(state_guard);
    
    // Determine the output directory
    let output_dir = if target_folder.is_empty() || target_folder == "~mods" {
        mod_directory.clone()
    } else {
        mod_directory.join(&target_folder)
    };
    
    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create target folder '{}': {}", target_folder, e));
    }
    
    info!("[QuickOrganize] Copying {} file(s) to '{}'", paths.len(), output_dir.display());
    let _ = window.emit("install_log", format!("[QuickOrganize] Copying to folder: {}", target_folder));
    
    let mut copied_count = 0;
    
    for path_str in paths {
        let path = PathBuf::from(&path_str);
        
        if !path.exists() {
            warn!("[QuickOrganize] Path does not exist: {}", path_str);
            continue;
        }
        
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        
        // Handle archives - extract and copy contents
        if ext == "zip" || ext == "rar" || ext == "7z" {
            let _ = window.emit("install_log", format!("[QuickOrganize] Extracting archive: {}", path.file_name().unwrap_or_default().to_string_lossy()));
            
            let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
            let temp_path = temp_dir.path().to_str().unwrap();
            
            // Extract archive
            let extract_result = if ext == "zip" {
                extract_zip(path.to_str().unwrap(), temp_path)
            } else if ext == "rar" {
                extract_rar(path.to_str().unwrap(), temp_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            } else {
                extract_7z(path.to_str().unwrap(), temp_path)
            };
            
            if let Err(e) = extract_result {
                error!("[QuickOrganize] Failed to extract archive: {}", e);
                let _ = window.emit("install_log", format!("[QuickOrganize] ERROR: Failed to extract archive: {}", e));
                continue;
            }
            
            // Find and copy all pak/utoc/ucas files from extracted content
            for entry in WalkDir::new(temp_path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if let Some(entry_ext) = entry_path.extension().and_then(|s| s.to_str()) {
                    if entry_ext == "pak" || entry_ext == "utoc" || entry_ext == "ucas" {
                        let file_name = entry_path.file_name().unwrap();
                        let dest = output_dir.join(file_name);
                        
                        if let Err(e) = std::fs::copy(entry_path, &dest) {
                            error!("[QuickOrganize] Failed to copy {}: {}", file_name.to_string_lossy(), e);
                        } else {
                            info!("[QuickOrganize] Copied: {}", file_name.to_string_lossy());
                            let _ = window.emit("install_log", format!("[QuickOrganize] Copied: {}", file_name.to_string_lossy()));
                            copied_count += 1;
                        }
                    }
                }
            }
        }
        // Handle pak files (and their iostore companions)
        else if ext == "pak" {
            let file_name = path.file_name().unwrap();
            let dest = output_dir.join(file_name);
            
            // Copy the pak file
            if let Err(e) = std::fs::copy(&path, &dest) {
                error!("[QuickOrganize] Failed to copy {}: {}", file_name.to_string_lossy(), e);
                continue;
            }
            
            info!("[QuickOrganize] Copied: {}", file_name.to_string_lossy());
            let _ = window.emit("install_log", format!("[QuickOrganize] Copied: {}", file_name.to_string_lossy()));
            copied_count += 1;
            
            // Also copy utoc and ucas if they exist (IoStore package)
            let utoc_path = path.with_extension("utoc");
            let ucas_path = path.with_extension("ucas");
            
            if utoc_path.exists() {
                let utoc_name = utoc_path.file_name().unwrap();
                if let Err(e) = std::fs::copy(&utoc_path, output_dir.join(utoc_name)) {
                    error!("[QuickOrganize] Failed to copy {}: {}", utoc_name.to_string_lossy(), e);
                } else {
                    copied_count += 1;
                }
            }
            
            if ucas_path.exists() {
                let ucas_name = ucas_path.file_name().unwrap();
                if let Err(e) = std::fs::copy(&ucas_path, output_dir.join(ucas_name)) {
                    error!("[QuickOrganize] Failed to copy {}: {}", ucas_name.to_string_lossy(), e);
                } else {
                    copied_count += 1;
                }
            }
        }
        // Handle directories - copy all pak/utoc/ucas files
        else if path.is_dir() {
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if let Some(entry_ext) = entry_path.extension().and_then(|s| s.to_str()) {
                    if entry_ext == "pak" || entry_ext == "utoc" || entry_ext == "ucas" {
                        let file_name = entry_path.file_name().unwrap();
                        let dest = output_dir.join(file_name);
                        
                        if let Err(e) = std::fs::copy(entry_path, &dest) {
                            error!("[QuickOrganize] Failed to copy {}: {}", file_name.to_string_lossy(), e);
                        } else {
                            info!("[QuickOrganize] Copied: {}", file_name.to_string_lossy());
                            let _ = window.emit("install_log", format!("[QuickOrganize] Copied: {}", file_name.to_string_lossy()));
                            copied_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    let _ = window.emit("install_log", format!("[QuickOrganize] Done! Copied {} file(s)", copied_count));
    info!("[QuickOrganize] Completed: {} files copied to {}", copied_count, output_dir.display());
    
    Ok(copied_count)
}

#[tauri::command]
async fn install_mods(
    mods: Vec<ModToInstall>,
    window: Window,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    use std::sync::atomic::{AtomicI32, AtomicBool};
    use std::sync::Arc as StdArc;

    let state_guard = state.lock().unwrap();
    let mod_directory = state_guard.game_path.clone();
    let usmap_filename = state_guard.usmap_path.clone();
    drop(state_guard);

    // Propagate USMAP path to UAssetBridge via environment for UAssetAPI-based processing (from roaming folder)
    if !usmap_filename.is_empty() {
        if let Some(usmap_full_path) = get_usmap_full_path(&usmap_filename) {
            std::env::set_var("USMAP_PATH", &usmap_full_path);
            info!(
                "Set USMAP_PATH for UAssetBridge: {}",
                usmap_full_path.display()
            );
        } else {
            let expected_path = usmap_dir().join(&usmap_filename);
            error!(
                "USMAP file not found at expected path for UAssetBridge: {}",
                expected_path.display()
            );
        }
    }

    if !mod_directory.exists() {
        std::fs::create_dir_all(&mod_directory)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    }

    // Convert paths to properly initialized InstallableMods
    use crate::install_mod::map_paths_to_mods;

    let paths: Vec<PathBuf> = mods.iter().map(|m| PathBuf::from(&m.path)).collect();

    // Log the paths we're trying to install
    for p in &paths {
        info!("[Install] Processing path: {}", p.display());
        let _ = window.emit("install_log", format!("[Install] Processing path: {}", p.display()));
    }

    let mut installable_mods = map_paths_to_mods(&paths);

    // Check if we actually have mods to install
    if installable_mods.is_empty() {
        error!("[Install] No valid mods found from {} input path(s)", paths.len());
        let _ = window.emit("install_log", "ERROR: No valid mods found to install!");
        let _ = window.emit("install_log", "Possible causes:");
        let _ = window.emit("install_log", "  - PAK file couldn't be read (wrong AES key or corrupted)");
        let _ = window.emit("install_log", "  - Archive contains no .pak files or content folders");
        let _ = window.emit("install_log", "  - Directory contains no valid content");
        return Err("No valid mods found to install. Check the install logs for details.".to_string());
    }

    // Apply user settings to each mod
    for (idx, mod_to_install) in mods.iter().enumerate() {
        if let Some(installable) = installable_mods.get_mut(idx) {
            // Apply custom name if provided
            if let Some(ref custom) = mod_to_install.custom_name {
                if !custom.is_empty() {
                    installable.mod_name = custom.clone();
                }
            }

            // Apply fix settings
            installable.fix_mesh = mod_to_install.fix_mesh;
            installable.fix_textures = mod_to_install.fix_texture;
            installable.fix_serialsize_header = mod_to_install.fix_serialize_size;
            installable.repak = mod_to_install.to_repak;
            installable.usmap_path = usmap_filename.clone();
        }
    }

    // Use existing installation logic
    let installed_counter = StdArc::new(AtomicI32::new(0));
    let stop_flag = StdArc::new(AtomicBool::new(false));

    let total = installable_mods.len() as i32;
    let counter_clone = installed_counter.clone();
    let _stop_clone = stop_flag.clone();
    let window_clone = window.clone();
    
    // Spawn installation thread
    let window_for_logs = window.clone();
    std::thread::spawn(move || {
        use std::panic;
        
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            window_for_logs.emit("install_log", "Starting installation...").ok();
            window_for_logs.emit("install_log", format!("Installing {} mod(s)", installable_mods.len())).ok();
            
            for (idx, imod) in installable_mods.iter().enumerate() {
                window_for_logs.emit("install_log", format!("[{}/{}] Mod: {}", idx + 1, installable_mods.len(), imod.mod_name)).ok();
                window_for_logs.emit("install_log", format!("  - Fix Mesh: {}", imod.fix_mesh)).ok();
                window_for_logs.emit("install_log", format!("  - Fix Textures: {}", imod.fix_textures)).ok();
                window_for_logs.emit("install_log", format!("  - Fix SerializeSize: {}", imod.fix_serialsize_header)).ok();
                window_for_logs.emit("install_log", format!("  - Repak: {}", imod.repak)).ok();
            }
            
            window_for_logs.emit("install_log", "Calling installation logic...").ok();
            window_for_logs.emit("install_log", format!("Mod directory: {}", mod_directory.display())).ok();
            
            use crate::install_mod::install_mod_logic::install_mods_in_viewport;
            
            window_for_logs.emit("install_log", "Entering install_mods_in_viewport...").ok();
            
            // Log each mod's path before processing
            for (idx, m) in installable_mods.iter().enumerate() {
                window_for_logs.emit("install_log", format!("  Mod {} path exists: {}", idx, m.mod_path.exists())).ok();
                window_for_logs.emit("install_log", format!("  Mod {} path: {}", idx, m.mod_path.display())).ok();
            }
            
            install_mods_in_viewport(
                &mut installable_mods,
                &mod_directory,
                &installed_counter,
                &stop_flag,
            );
            window_for_logs.emit("install_log", "Exited install_mods_in_viewport").ok();
        }));
        
        match result {
            Ok(_) => {
                window_for_logs.emit("install_log", "Installation completed successfully!").ok();
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    format!("PANIC: {}", s)
                } else if let Some(s) = e.downcast_ref::<String>() {
                    format!("PANIC: {}", s)
                } else {
                    "PANIC: Unknown error".to_string()
                };
                window_for_logs.emit("install_log", msg).ok();
                error!("Installation thread panicked!");
            }
        }
    });
    
    // Monitor progress
    std::thread::spawn(move || {
        loop {
            let current = counter_clone.load(std::sync::atomic::Ordering::SeqCst);
            if current == -255 {
                window_clone.emit("install_complete", ()).ok();
                break;
            }
            let progress = (current as f32 / total as f32) * 100.0;
            window_clone.emit("install_progress", progress).ok();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    
    Ok(())
}

#[tauri::command]
async fn delete_mod(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    
    // Try to delete the main file
    if path_buf.exists() {
        std::fs::remove_file(&path_buf)
            .map_err(|e| format!("Failed to delete mod file: {}", e))?;
    }

    // If it's a .pak file, try to delete associated IOStore files
    if let Some(extension) = path_buf.extension() {
        if extension.to_string_lossy().to_lowercase() == "pak" {
            // We need to handle the case where the file name might have multiple dots, 
            // but with_extension replaces the last one, which is what we want for .pak -> .ucas
            let ucas_path = path_buf.with_extension("ucas");
            if ucas_path.exists() {
                let _ = std::fs::remove_file(ucas_path);
            }
            
            let utoc_path = path_buf.with_extension("utoc");
            if utoc_path.exists() {
                let _ = std::fs::remove_file(utoc_path);
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
async fn create_folder(name: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    // Create physical directory in ~mods
    let folder_path = game_path.join(&name);
    
    if folder_path.exists() {
        return Err("Folder already exists".to_string());
    }
    
    // Use create_dir_all to support nested paths like "Category/Subcategory"
    std::fs::create_dir_all(&folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;
    
    Ok(name)
}

#[tauri::command]
async fn get_folders(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<ModFolder>, String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    if !game_path.exists() {
        return Ok(Vec::new());
    }
    
    let mut folders = Vec::new();
    
    // Get root folder name (e.g., "~mods")
    let root_name = game_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Mods")
        .to_string();
    
    // Count mods directly in root (not in subfolders)
    let root_mod_count = std::fs::read_dir(game_path)
        .map(|entries| {
            entries.filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|s| s.to_str());
                        ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled")
                    } else {
                        false
                    }
                })
                .count()
        })
        .unwrap_or(0);
    
    // Add root folder first (depth 0) - use actual folder name as ID
    folders.push(ModFolder {
        id: root_name.clone(),  // Use actual name like "~mods" as ID
        name: root_name.clone(),
        enabled: true,
        expanded: true,
        color: None,
        depth: 0,
        parent_id: None,
        is_root: true,
        mod_count: root_mod_count,
    });
    
    // Recursively scan for subdirectories using WalkDir
    for entry in WalkDir::new(game_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        let path = entry.path();
        
        if path.is_dir() {
            // Calculate relative path from game_path to get ID
            let relative_path = path.strip_prefix(game_path)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| "Unknown".to_string());
                
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            // Calculate depth (number of path segments)
            let depth = relative_path.split('/').count();
            
            // Calculate parent ID
            let parent_id = if depth > 1 {
                // If depth > 1, parent is the directory containing this one
                // e.g. "A/B" -> parent is "A"
                let parent_rel = std::path::Path::new(&relative_path)
                    .parent()
                    .map(|p| p.to_string_lossy().replace('\\', "/"));
                parent_rel
            } else {
                // If depth is 1, parent is the root folder
                Some(root_name.clone())
            };

            // Count mods in this folder (only direct children)
            let mod_count = std::fs::read_dir(&path)
                .map(|entries| {
                    entries.filter_map(|e| e.ok())
                        .filter(|e| {
                            let p = e.path();
                            if p.is_file() {
                                let ext = p.extension().and_then(|s| s.to_str());
                                ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled")
                            } else {
                                false
                            }
                        })
                        .count()
                })
                .unwrap_or(0);
            
            folders.push(ModFolder {
                id: relative_path, // ID is the relative path (e.g. "Category/Subcategory")
                name,
                enabled: true,
                expanded: true,
                color: None,
                depth,
                parent_id,
                is_root: false,
                mod_count,
            });
        }
    }
    
    Ok(folders)
}

/// Get detailed info about the root mods folder
#[tauri::command]
async fn get_root_folder_info(state: State<'_, Arc<Mutex<AppState>>>) -> Result<RootFolderInfo, String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    if !game_path.exists() {
        return Err("Game path does not exist".to_string());
    }
    
    let root_name = game_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Mods")
        .to_string();
    
    let mut direct_mod_count = 0;
    let mut subfolder_count = 0;
    
    for entry in std::fs::read_dir(game_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_dir() {
            subfolder_count += 1;
        } else if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str());
            if ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled") {
                direct_mod_count += 1;
            }
        }
    }
    
    Ok(RootFolderInfo {
        name: root_name,
        path: game_path.to_string_lossy().to_string(),
        direct_mod_count,
        subfolder_count,
    })
}

#[tauri::command]
async fn update_folder(
    folder: ModFolder,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(existing) = state.folders.iter_mut().find(|f| f.id == folder.id) {
        *existing = folder;
        save_state(&state).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn delete_folder(id: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    let folder_path = game_path.join(&id);
    
    if !folder_path.exists() {
        return Err("Folder does not exist".to_string());
    }
    
    // Delete physical directory (will fail if not empty, which is good for safety)
    std::fs::remove_dir(&folder_path)
        .map_err(|e| format!("Failed to delete folder (may not be empty): {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn assign_mod_to_folder(
    mod_path: String,
    folder_id: Option<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    let source_path = PathBuf::from(&mod_path);
    
    if !source_path.exists() {
        return Err("Mod file does not exist".to_string());
    }
    
    let filename = source_path.file_name()
        .ok_or("Invalid file name")?;
    
    let dest_path = if let Some(folder_name) = folder_id {
        // Move to folder
        let folder_path = game_path.join(&folder_name);
        if !folder_path.exists() {
            return Err("Folder does not exist".to_string());
        }
        folder_path.join(filename)
    } else {
        // Move back to root ~mods directory
        game_path.join(filename)
    };
    
    // Move the main file
    std::fs::rename(&source_path, &dest_path)
        .map_err(|e| format!("Failed to move mod: {}", e))?;
    
    // Also move .utoc and .ucas files if they exist (IoStore files)
    let utoc_source = source_path.with_extension("utoc");
    let ucas_source = source_path.with_extension("ucas");
    
    if utoc_source.exists() {
        let utoc_dest = dest_path.with_extension("utoc");
        let _ = std::fs::rename(&utoc_source, &utoc_dest);
    }
    
    if ucas_source.exists() {
        let ucas_dest = dest_path.with_extension("ucas");
        let _ = std::fs::rename(&ucas_source, &ucas_dest);
    }
    
    Ok(())
}

#[tauri::command]
async fn add_custom_tag(
    mod_path: String,
    tag: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let path = PathBuf::from(&mod_path);
    
    // Find or create mod metadata
    if let Some(metadata) = state.mod_metadata.iter_mut().find(|m| m.path == path) {
        if !metadata.custom_tags.contains(&tag) {
            metadata.custom_tags.push(tag);
        }
    } else {
        state.mod_metadata.push(ModMetadata {
            path,
            custom_name: None,
            folder_id: None,
            custom_tags: vec![tag],
        });
    }
    
    save_state(&state).map_err(|e| e.to_string())?;
    Ok(())
}

/// Copy a USMAP file to the roaming folder, replacing any existing USMAP files.
/// 
/// # Arguments
/// * `source_path` - Full path to the source .usmap file
/// 
/// # Returns
/// The filename of the copied USMAP file (just the name, not full path)
/// 
/// # Behavior
/// - Deletes ALL existing .usmap files in the roaming Usmap folder before copying
/// - Copies the new file to `%APPDATA%/RepakGuiRevamped/Usmap/`
/// - Only one USMAP file should exist at a time
#[tauri::command]
async fn copy_usmap_to_folder(source_path: String) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    
    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }
    
    // Get the Usmap directory in roaming folder
    let usmap_folder = usmap_dir();
    std::fs::create_dir_all(&usmap_folder)
        .map_err(|e| format!("Failed to create Usmap directory: {}", e))?;
    
    // Delete all existing .usmap files in the folder
    if let Ok(entries) = std::fs::read_dir(&usmap_folder) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("usmap") {
                if let Err(e) = std::fs::remove_file(&path) {
                    warn!("Failed to delete old USMAP file {:?}: {}", path, e);
                } else {
                    info!("Deleted old USMAP file: {:?}", path);
                }
            }
        }
    }
    
    // Get filename from source
    let filename = source.file_name()
        .ok_or("Invalid source filename")?
        .to_str()
        .ok_or("Invalid UTF-8 in filename")?;
    
    // Copy file to Usmap/ folder in roaming
    let dest_path = usmap_folder.join(filename);
    std::fs::copy(&source, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    
    info!("Copied USmap file {} to {}", filename, usmap_folder.display());
    
    // Return just the filename
    Ok(filename.to_string())
}

#[tauri::command]
async fn set_usmap_path(usmap_path: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    state.usmap_path = usmap_path.clone();
    info!("Set USMAP path in AppState: {}", usmap_path);
    Ok(())
}

#[tauri::command]
async fn get_usmap_path(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().unwrap();
    Ok(state.usmap_path.clone())
}

/// Get the USMAP directory path in the roaming folder.
/// 
/// # Returns
/// Full path to `%APPDATA%/RepakGuiRevamped/Usmap/`
#[tauri::command]
async fn get_usmap_dir_path() -> Result<String, String> {
    Ok(usmap_dir().to_string_lossy().to_string())
}

/// List all USMAP files currently in the roaming Usmap folder.
/// Reads from filesystem at runtime, not from saved state.
/// 
/// # Returns
/// Vector of filenames (not full paths) of .usmap files in the folder
#[tauri::command]
async fn list_usmap_files() -> Result<Vec<String>, String> {
    let usmap_folder = usmap_dir();
    
    if !usmap_folder.exists() {
        return Ok(Vec::new());
    }
    
    let entries = std::fs::read_dir(&usmap_folder)
        .map_err(|e| format!("Failed to read Usmap directory: {}", e))?;
    
    let mut files = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("usmap") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                files.push(filename.to_string());
            }
        }
    }
    
    Ok(files)
}

/// Get the currently active USMAP file by reading from filesystem.
/// This reads the actual files in the Usmap folder, not the saved state.
/// 
/// # Returns
/// - Filename of the first .usmap file found (there should only be one)
/// - Empty string if no .usmap files exist
#[tauri::command]
async fn get_current_usmap_file() -> Result<String, String> {
    let files = list_usmap_files().await?;
    Ok(files.into_iter().next().unwrap_or_default())
}

/// Get the full path to the currently active USMAP file.
/// 
/// # Returns
/// - Full path to the .usmap file if one exists
/// - Empty string if no .usmap file exists
#[tauri::command]
async fn get_current_usmap_full_path() -> Result<String, String> {
    let files = list_usmap_files().await?;
    if let Some(filename) = files.into_iter().next() {
        let full_path = usmap_dir().join(&filename);
        Ok(full_path.to_string_lossy().to_string())
    } else {
        Ok(String::new())
    }
}

/// Delete the currently active USMAP file from the roaming folder.
/// 
/// # Returns
/// - `true` if a file was deleted
/// - `false` if no file existed to delete
#[tauri::command]
async fn delete_current_usmap() -> Result<bool, String> {
    let usmap_folder = usmap_dir();
    
    if !usmap_folder.exists() {
        return Ok(false);
    }
    
    let entries = std::fs::read_dir(&usmap_folder)
        .map_err(|e| format!("Failed to read Usmap directory: {}", e))?;
    
    let mut deleted = false;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("usmap") {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete USMAP file: {}", e))?;
            info!("Deleted USMAP file: {:?}", path);
            deleted = true;
        }
    }
    
    Ok(deleted)
}

#[tauri::command]
async fn get_all_tags(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<String>, String> {
    let state = state.lock().unwrap();
    let mut tags = std::collections::HashSet::new();
    
    for metadata in &state.mod_metadata {
        for tag in &metadata.custom_tags {
            tags.insert(tag.clone());
        }
    }
    
    let mut tags_vec: Vec<String> = tags.into_iter().collect();
    tags_vec.sort();
    Ok(tags_vec)
}

#[tauri::command]
async fn toggle_mod(mod_path: String) -> Result<bool, String> {
    let path = PathBuf::from(&mod_path);
    
    if !path.exists() {
        return Err("Mod file does not exist".to_string());
    }
    
    // Check current state
    let is_enabled = path.extension().and_then(|s| s.to_str()) == Some("pak");
    
    // Toggle by renaming
    let new_path = if is_enabled {
        path.with_extension("bak_repak")
    } else {
        path.with_extension("pak")
    };
    
    std::fs::rename(&path, &new_path)
        .map_err(|e| format!("Failed to toggle mod: {}", e))?;
    
    Ok(!is_enabled)
}

#[tauri::command]
async fn extract_pak_to_destination(mod_path: String, dest_path: String) -> Result<(), String> {
    use crate::install_mod::install_mod_logic::pak_files::extract_pak_to_dir;
    use crate::install_mod::InstallableMod;
    use repak::PakBuilder;
    use repak::utils::AesKey;
    use std::str::FromStr;
    use std::io::BufReader;
    
    let pak_path = PathBuf::from(&mod_path);
    if !pak_path.exists() {
        return Err("Pak file not found".to_string());
    }

    let dest_dir = PathBuf::from(&dest_path);
    let mod_name = pak_path.file_stem().unwrap().to_string_lossy().to_string();
    let to_create = dest_dir.join(&mod_name);
    
    std::fs::create_dir_all(&to_create).map_err(|e| e.to_string())?;
    
    // Open PAK
    let file = File::open(&pak_path).map_err(|e| e.to_string())?;
    let aes_key = AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
        .map_err(|e| e.to_string())?;
        
    let mut reader = BufReader::new(file);
    let pak_reader = PakBuilder::new()
        .key(aes_key.0)
        .reader(&mut reader)
        .map_err(|e| e.to_string())?;
        
    let installable_mod = InstallableMod {
        mod_name: mod_name.clone(),
        mod_type: "".to_string(),
        reader: Option::from(pak_reader),
        mod_path: pak_path.clone(),
        ..Default::default()
    };
    
    extract_pak_to_dir(&installable_mod, to_create).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn check_game_running() -> Result<bool, String> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};
    
    let s = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new())
    );
    
    let game_process_names = ["Marvel-Win64-Shipping.exe"];
    
    for (_pid, process) in s.processes() {
        let process_name = process.name().to_string_lossy().to_lowercase();
        for game_name in &game_process_names {
            if process_name == game_name.to_lowercase() {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

/// Launch Marvel Rivals via Steam, temporarily skipping the launcher
/// 
/// This function:
/// 1. Backs up the current launch_record value
/// 2. DELETES the launch_record file
/// 3. RECREATES it with "0" to skip the launcher
/// 4. Launches the game via Steam protocol
/// 5. Restores the original launch_record after game starts
/// 
/// This ensures the game launches without the launcher when using our app,
/// but preserves the user's Steam launch settings for manual launches
#[tauri::command]
async fn launch_game(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    use std::process::Command;
    
    // Get game path (this is the ~mods folder inside Paks)
    let mods_path = {
        let state = state.lock().unwrap();
        state.game_path.clone()
    };
    
    // Go up 5 levels to get the actual game root
    // ~mods -> Paks -> Content -> Marvel -> MarvelGame -> MarvelRivals (game root)
    let game_root = mods_path
        .parent() // Paks
        .and_then(|p| p.parent()) // Content
        .and_then(|p| p.parent()) // Marvel
        .and_then(|p| p.parent()) // MarvelGame
        .and_then(|p| p.parent()) // MarvelRivals (game root)
        .ok_or_else(|| "Could not determine game root directory".to_string())?;
    
    // Path to launch_record file (in the game root, next to MarvelRivals_Launcher.exe)
    let launch_record_path = game_root.join("launch_record");
    
    // Backup original value
    let original_value = match std::fs::read_to_string(&launch_record_path) {
        Ok(content) => {
            info!("Backed up launch_record value: {}", content.trim());
            Some(content)
        }
        Err(e) => {
            warn!("Could not read launch_record (file may not exist): {}", e);
            None
        }
    };
    
    // DELETE the launch_record file
    if launch_record_path.exists() {
        if let Err(e) = std::fs::remove_file(&launch_record_path) {
            error!("Failed to delete launch_record: {}", e);
            return Err(format!("Failed to delete launch_record: {}", e));
        }
        info!("Deleted launch_record file");
    }
    
    // RECREATE it with "0" to skip launcher
    if let Err(e) = std::fs::write(&launch_record_path, "0") {
        error!("Failed to recreate launch_record: {}", e);
        return Err(format!("Failed to recreate launch_record: {}", e));
    }
    info!("Recreated launch_record with value 0 (skip launcher)");
    
    // Launch the game via Steam with RUNASINVOKER to skip UAC prompt
    #[cfg(target_os = "windows")]
    let launch_result = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        Command::new("cmd")
            .arg("/C")
            .arg("set")
            .arg("__COMPAT_LAYER=RUNASINVOKER")
            .arg("&&")
            .arg("start")
            .arg("")
            .arg("steam://run/2767030")
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
    };
    
    #[cfg(target_os = "macos")]
    let launch_result = Command::new("open")
        .arg("steam://run/2767030")
        .spawn();
    
    #[cfg(target_os = "linux")]
    let launch_result = Command::new("xdg-open")
        .arg("steam://run/2767030")
        .spawn();
    
    // Check launch result
    match launch_result {
        Ok(_) => {
            info!("Successfully launched Marvel Rivals via Steam");
            
            // Spawn a background task to restore the launch_record after the game starts
            let launch_record_path_clone = launch_record_path.clone();
            std::thread::spawn(move || {
                use sysinfo::{ProcessRefreshKind, RefreshKind, System};
                
                // Wait for the game process to start (up to 30 seconds)
                let mut waited = 0;
                let mut game_started = false;
                
                while waited < 30000 {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    waited += 1000;
                    
                    // Check if game process is running
                    let s = System::new_with_specifics(
                        RefreshKind::new().with_processes(ProcessRefreshKind::new())
                    );
                    
                    let mut found = false;
                    for (_pid, process) in s.processes() {
                        let process_name = process.name().to_string_lossy().to_lowercase();
                        if process_name == "marvel-win64-shipping.exe" {
                            info!("Game process detected, waiting 2 more seconds before restoring launch_record");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            found = true;
                            game_started = true;
                            break;
                        }
                    }
                    
                    if found {
                        break;
                    }
                }
                
                if !game_started {
                    warn!("Timeout waiting for game to start, restoring launch_record anyway");
                }
                
                // DELETE and RECREATE with original value
                if let Some(original) = original_value {
                    if launch_record_path_clone.exists() {
                        if let Err(e) = std::fs::remove_file(&launch_record_path_clone) {
                            warn!("Failed to delete launch_record for restoration: {}", e);
                            return;
                        }
                    }
                    
                    if let Err(e) = std::fs::write(&launch_record_path_clone, original.trim()) {
                        warn!("Failed to recreate launch_record with original value: {}", e);
                    } else {
                        info!("Restored launch_record to original value (game_started: {})", game_started);
                    }
                }
            });
            
            Ok(())
        }
        Err(e) => {
            error!("Failed to launch game: {}", e);
            Err(format!("Failed to launch game. Please ensure Steam is installed. Error: {}", e))
        }
    }
}

/// Toggle the skip launcher patch (manual control)
/// Returns true if skip launcher is now enabled (0), false if disabled (6)
#[tauri::command]
async fn skip_launcher_patch(state: State<'_, Arc<Mutex<AppState>>>) -> Result<bool, String> {
    // Get game path (this is the ~mods folder inside Paks)
    let mods_path = {
        let state = state.lock().unwrap();
        state.game_path.clone()
    };
    
    // Go up 5 levels to get the actual game root
    // ~mods -> Paks -> Content -> Marvel -> MarvelGame -> MarvelRivals (game root)
    let game_root = mods_path
        .parent() // Paks
        .and_then(|p| p.parent()) // Content
        .and_then(|p| p.parent()) // Marvel
        .and_then(|p| p.parent()) // MarvelGame
        .and_then(|p| p.parent()) // MarvelRivals (game root)
        .ok_or_else(|| "Could not determine game root directory".to_string())?;
    
    // Path to launch_record file
    let launch_record_path = game_root.join("launch_record");
    
    info!("Mods path: {:?}", mods_path);
    info!("Game root: {:?}", game_root);
    info!("Launch record path: {:?}", launch_record_path);
    
    // Read current value
    let current_value = match std::fs::read_to_string(&launch_record_path) {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            // If file doesn't exist, assume default (6)
            "6".to_string()
        }
    };
    
    // Determine new value (toggle between 0 and 6)
    let new_value = if current_value == "0" {
        "6" // Disable skip launcher (show launcher)
    } else {
        "0" // Enable skip launcher
    };
    
    // Delete and recreate the file with new value
    if launch_record_path.exists() {
        std::fs::remove_file(&launch_record_path)
            .map_err(|e| format!("Failed to delete launch_record: {}", e))?;
    }
    
    std::fs::write(&launch_record_path, &new_value)
        .map_err(|e| format!("Failed to write launch_record: {}", e))?;
    
    let skip_enabled = new_value == "0";
    info!("Skip launcher patch toggled: {} (value: {})", skip_enabled, new_value);
    
    Ok(skip_enabled)
}

/// Check if skip launcher patch is currently enabled
#[tauri::command]
async fn get_skip_launcher_status(state: State<'_, Arc<Mutex<AppState>>>) -> Result<bool, String> {
    // Get game path (this is the ~mods folder inside Paks)
    let mods_path = {
        let state = state.lock().unwrap();
        state.game_path.clone()
    };
    
    // Go up 5 levels to get the actual game root
    // ~mods -> Paks -> Content -> Marvel -> MarvelGame -> MarvelRivals (game root)
    let game_root = mods_path
        .parent() // Paks
        .and_then(|p| p.parent()) // Content
        .and_then(|p| p.parent()) // Marvel
        .and_then(|p| p.parent()) // MarvelGame
        .and_then(|p| p.parent()) // MarvelRivals (game root)
        .ok_or_else(|| "Could not determine game root directory".to_string())?;
    
    // Path to launch_record file
    let launch_record_path = game_root.join("launch_record");
    
    // Read current value
    let current_value = match std::fs::read_to_string(&launch_record_path) {
        Ok(content) => content.trim().to_string(),
        Err(_) => "6".to_string(), // Default if file doesn't exist
    };
    
    Ok(current_value == "0")
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let client = reqwest::Client::new();
    // Assuming repository is correct based on context
    let url = "https://api.github.com/repos/XzantGaming/Repak-Gui-Revamped/releases/latest";
    
    let res = client.get(url)
        .header("User-Agent", "Repak-Gui-Revamped")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
        
    if !res.status().is_success() {
        return Ok(None);
    }
    
    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    let tag_name = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v');
    let current = env!("CARGO_PKG_VERSION");
    
    if let (Ok(remote_ver), Ok(current_ver)) = (semver::Version::parse(tag_name), semver::Version::parse(current)) {
        if remote_ver > current_ver {
             let url = json["html_url"].as_str().unwrap_or("").to_string();
             let assets = json["assets"].as_array();
             
             let mut asset_url = None;
             let mut asset_name = None;
             
             if let Some(assets) = assets {
                 if let Some(asset) = assets.iter().find(|a| {
                     let name = a["name"].as_str().unwrap_or("");
                     name.ends_with(".exe") || name.ends_with(".msi") || name.ends_with(".zip")
                 }) {
                     asset_url = asset["browser_download_url"].as_str().map(|s| s.to_string());
                     asset_name = asset["name"].as_str().map(|s| s.to_string());
                 }
             }
             
             return Ok(Some(UpdateInfo {
                 latest: tag_name.to_string(),
                 url,
                 asset_url,
                 asset_name,
             }));
        }
    }
    
    Ok(None)
}

#[derive(Serialize, Deserialize)]
struct UpdateInfo {
    latest: String,
    url: String,
    asset_url: Option<String>,
    asset_name: Option<String>,
}

// ============================================================================
// CRASH MONITORING COMMANDS
// ============================================================================

/// Monitor game state and detect crashes
/// This should be called periodically (every 2-5 seconds) from the frontend
#[tauri::command]
async fn monitor_game_for_crashes(
    crash_state: State<'_, CrashMonitorState>,
) -> Result<Option<crash_monitor::CrashInfo>, String> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};
    
    let s = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new())
    );
    
    let game_process_names = ["Marvel-Win64-Shipping.exe"];
    let mut game_running = false;
    
    for (_pid, process) in s.processes() {
        let process_name = process.name().to_string_lossy().to_lowercase();
        for game_name in &game_process_names {
            if process_name == game_name.to_lowercase() {
                game_running = true;
                break;
            }
        }
    }
    
    let mut game_start_time = crash_state.game_start_time.lock().unwrap();
    let mut last_checked = crash_state.last_checked_crash.lock().unwrap();
    
    // Game just started - record the start time
    if game_running && game_start_time.is_none() {
        let now = std::time::SystemTime::now();
        *game_start_time = Some(now);
        *last_checked = Some(now);
        info!("Game started - monitoring for crashes from: {:?}", now);
        return Ok(None);
    }
    
    // Game just stopped - check for crashes that occurred during THIS session
    if !game_running && game_start_time.is_some() {
        let session_start = game_start_time.unwrap();
        info!("Game stopped - checking for crashes since session start: {:?}", session_start);
        
        // Check for crashes created AFTER the game started
        let new_crashes = crash_monitor::check_for_new_crashes(session_start);
        
        // Reset state for next session
        *game_start_time = None;
        
        if !new_crashes.is_empty() {
            error!("⚠️ ═══════════════════════════════════════════════════════════════");
            error!("⚠️ CRASH DETECTED! Marvel Rivals crashed during this session!");
            error!("⚠️ ═══════════════════════════════════════════════════════════════");
            error!("⚠️ Found {} crash folder(s) from this session", new_crashes.len());
            
            // Parse the most recent crash
            if let Some(crash_folder) = new_crashes.first() {
                let crash_info = crash_monitor::parse_crash_info(crash_folder, Vec::new());
                
                if let Some(ref info) = crash_info {
                    let unknown_error = "Unknown error".to_string();
                    let error_msg = info.error_message.as_ref().unwrap_or(&unknown_error);
                    
                    error!("⚠️ Crash Details:");
                    error!("⚠️   Type: {}", info.crash_type.as_ref().unwrap_or(&"Unknown".to_string()));
                    
                    // Parse and display detailed error information
                    let (asset_path, error_type, details) = crash_monitor::parse_error_details(error_msg);
                    
                    if let Some(err_type) = error_type {
                        error!("⚠️   Error Type: {}", err_type);
                    }
                    
                    if let Some(asset) = asset_path {
                        error!("⚠️   Affected Asset: {}", asset);
                        
                        // Extract character ID if present
                        if let Some(char_id) = crash_monitor::extract_character_id(error_msg) {
                            error!("⚠️   Character ID: {}", char_id);
                        }
                    }
                    
                    if let Some(detail) = details {
                        error!("⚠️   Details: {}", detail);
                    }
                    
                    // Check if it's a mesh-related crash
                    if crash_monitor::is_mesh_related_crash(error_msg) {
                        error!("⚠️   ⚡ MESH LOADING ERROR - Likely caused by incorrect SerializeSize");
                        error!("⚠️   💡 Tip: Re-run UAssetGUI on this mod or disable 'Fix Mesh' option");
                    }
                    
                    if let Some(seconds) = info.seconds_since_start {
                        let minutes = seconds / 60;
                        let secs = seconds % 60;
                        error!("⚠️   Time in game: {}m {}s", minutes, secs);
                    }
                    
                    error!("⚠️   Crash folder: {:?}", crash_folder);
                    error!("⚠️   Mods enabled: {} mod(s)", info.enabled_mods.len());
                    
                    if !info.enabled_mods.is_empty() {
                        error!("⚠️   Active mods:");
                        for mod_name in &info.enabled_mods {
                            error!("⚠️     - {}", mod_name);
                        }
                    }
                    
                    // Show full error message at the end for reference
                    error!("⚠️");
                    error!("⚠️   Full Error Message:");
                    error!("⚠️   {}", error_msg);
                    error!("⚠️ ═══════════════════════════════════════════════════════════════");
                    
                    // Update last checked time to avoid re-reporting this crash
                    *last_checked = Some(info.timestamp);
                }
                
                return Ok(crash_info);
            }
        } else {
            info!("✓ ═══════════════════════════════════════════════════════════════");
            info!("✓ Game closed normally - no crashes detected this session");
            info!("✓ ═══════════════════════════════════════════════════════════════");
        }
    }
    
    Ok(None)
}

#[tauri::command]
async fn get_crash_history() -> Result<Vec<PathBuf>, String> {
    let crash_dir = crash_monitor::get_crash_log_path();
    
    if !crash_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut crashes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&crash_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                crashes.push(path);
            }
        }
    }
    
    // Sort by creation time (newest first)
    crashes.sort_by(|a, b| {
        let a_time = std::fs::metadata(a).and_then(|m| m.created()).ok();
        let b_time = std::fs::metadata(b).and_then(|m| m.created()).ok();
        b_time.cmp(&a_time)
    });
    
    Ok(crashes)
}

#[tauri::command]
async fn get_total_crashes() -> Result<usize, String> {
    Ok(crash_monitor::count_total_crashes())
}

#[tauri::command]
async fn clear_crash_logs() -> Result<usize, String> {
    crash_monitor::clear_all_crashes()
}

#[tauri::command]
async fn dismiss_crash_dialog() -> Result<(), String> {
    // This is a no-op in Tauri version - frontend handles dialog state
    Ok(())
}

#[tauri::command]
async fn get_crash_log_path() -> Result<String, String> {
    Ok(crash_monitor::get_crash_log_path().to_string_lossy().to_string())
}

// ============================================================================
// CHARACTER DATA COMMANDS
// ============================================================================

#[tauri::command]
async fn get_character_data() -> Result<Vec<character_data::CharacterSkin>, String> {
    Ok(character_data::get_all_character_data())
}

#[tauri::command]
async fn get_character_by_skin_id(skin_id: String) -> Result<Option<character_data::CharacterSkin>, String> {
    Ok(character_data::get_character_by_skin_id(&skin_id))
}

#[tauri::command]
async fn refresh_character_cache() -> Result<String, String> {
    info!("Manually refreshing character data cache...");
    character_data::refresh_cache();
    info!("Character data cache refreshed successfully");
    Ok("Character data cache refreshed successfully".to_string())
}

/// Update character data from GitHub MarvelRivalsCharacterIDs with progress events
/// Supports cancellation via cancel_character_update command
#[tauri::command]
async fn update_character_data_from_github(window: Window) -> Result<usize, String> {
    let _ = window.emit("install_log", "[Character Data] Starting GitHub data fetch...");
    
    // Create progress callback that emits events
    let window_clone = window.clone();
    let on_progress = move |msg: &str| {
        let _ = window_clone.emit("install_log", format!("[Character Data] {}", msg));
    };
    
    match character_data::update_from_github_with_progress(on_progress).await {
        Ok(new_count) => {
            let msg = format!("[Character Data] ✓ Complete! {} new skins added.", new_count);
            let _ = window.emit("install_log", &msg);
            // Trigger mod list refresh so new character names show up
            let _ = window.emit("character_data_updated", new_count);
            info!("Successfully updated character data. {} new skins added.", new_count);
            Ok(new_count)
        }
        Err(e) if e == "Cancelled" => {
            let _ = window.emit("install_log", "[Character Data] ✗ Update cancelled by user");
            Err(e)
        }
        Err(e) => {
            let msg = format!("[Character Data] ✗ Error: {}", e);
            let _ = window.emit("install_log", &msg);
            error!("Failed to update character data: {}", e);
            Err(e)
        }
    }
}

/// Cancel an ongoing character data update
#[tauri::command]
async fn cancel_character_update() -> Result<(), String> {
    character_data::request_cancel_update();
    info!("Character data update cancellation requested");
    Ok(())
}

#[tauri::command]
async fn get_character_data_path() -> Result<String, String> {
    Ok(character_data::character_data_path().to_string_lossy().to_string())
}

#[tauri::command]
async fn identify_mod_character(file_paths: Vec<String>) -> Result<Option<(String, String)>, String> {
    Ok(character_data::identify_mod_from_paths(&file_paths))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn app_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RepakGuiRevamped")
}

/// Directory for USMAP files - stored in roaming folder
fn usmap_dir() -> PathBuf {
    app_dir().join("Usmap")
}

/// Get the full path to a USMAP file by filename
fn get_usmap_full_path(usmap_filename: &str) -> Option<PathBuf> {
    if usmap_filename.is_empty() {
        return None;
    }
    
    let usmap_path = usmap_dir().join(usmap_filename);
    if usmap_path.exists() {
        Some(usmap_path)
    } else {
        None
    }
}

/// Directory for log files - placed next to the executable for easy access
fn log_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("Logs");
        }
    }
    // Fallback to config-based app_dir if current_exe fails
    app_dir()
}

fn save_state(state: &AppState) -> std::io::Result<()> {
    let dir = app_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("state.json");
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, state)?;
    Ok(())
}

fn load_state() -> AppState {
    let path = app_dir().join("state.json");
    let mut state = if let Ok(file) = File::open(path) {
        serde_json::from_reader(file).unwrap_or_default()
    } else {
        AppState::default()
    };
    
    // Auto-detect USMAP file from roaming folder on startup
    // This ensures the app always uses whatever USMAP is actually in the folder
    let usmap_folder = usmap_dir();
    if usmap_folder.exists() {
        if let Ok(entries) = std::fs::read_dir(&usmap_folder) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("usmap") {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        state.usmap_path = filename.to_string();
                        break; // Use first .usmap file found
                    }
                }
            }
        }
    }
    
    state
}

fn setup_logging() {
    // Try exe-relative Logs folder first
    let log_dir = log_dir();
    let log_file = log_dir.join("repak-gui.log");
    
    // Attempt to create the log directory
    let log_file_result = std::fs::create_dir_all(&log_dir)
        .and_then(|_| File::create(&log_file));
    
    let final_log_file = match log_file_result {
        Ok(file) => {
            // Successfully created log file at exe-relative location
            eprintln!("Logging to: {}", log_file.display());
            file
        }
        Err(e) => {
            // Fallback to temp directory if exe-relative fails
            eprintln!("Failed to create log at {}: {}", log_file.display(), e);
            let temp_log = std::env::temp_dir().join("repak-gui.log");
            eprintln!("Fallback logging to: {}", temp_log.display());
            File::create(&temp_log).expect("Failed to create log file even in temp directory")
        }
    };
    
    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            log::LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            log::LevelFilter::Debug,
            Config::default(),
            final_log_file,
        ),
    ]);
}

#[derive(Debug, Clone, serde::Serialize)]
struct ModDetails {
    mod_name: String,
    mod_type: String,
    character_name: String,
    category: String,
    file_count: usize,
    total_size: u64,
    files: Vec<String>,
    is_iostore: bool,
    has_blueprint: bool,
}

#[tauri::command]
async fn get_mod_details(mod_path: String, _detect_blueprint: Option<bool>) -> Result<ModDetails, String> {
    use repak::PakBuilder;
    use repak::utils::AesKey;
    use std::str::FromStr;
    use std::fs::File;
    use std::io::BufReader;
    
    let path = PathBuf::from(&mod_path);
    
    info!("Getting details for mod: {}", path.display());
    
    if !path.exists() {
        return Err(format!("Mod file does not exist: {}", path.display()));
    }
    
    // Get AES key
    let aes_key = AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
        .map_err(|e| format!("Failed to create AES key: {}", e))?;
    
    // Open PAK file directly (no temp file needed)
    let file = File::open(&path)
        .map_err(|e| format!("Failed to open PAK file: {}", e))?;
    
    let mut reader = BufReader::new(file);
    let pak = PakBuilder::new()
        .key(aes_key.0)
        .reader(&mut reader)
        .map_err(|e| format!("Failed to read PAK (bad AES key or corrupted file): {}", e))?;
    
    // Check if it's IoStore (has .utoc file)
    let mut utoc_path = path.clone();
    utoc_path.set_extension("utoc");
    let is_iostore = utoc_path.exists();
    
    // Get file list - same as egui version
    let files: Vec<String> = if is_iostore {
        // For IoStore, read from utoc
        use crate::utoc_utils::read_utoc;
        read_utoc(&utoc_path, &pak, &path)
            .iter()
            .map(|entry| entry.file_path.clone())
            .collect()
    } else {
        // For regular PAK
        pak.files()
    };
    
    let file_count = files.len();
    
    info!("PAK contains {} files", file_count);
    if file_count > 0 && file_count <= 10 {
        info!("Files: {:?}", files);
    } else if file_count > 10 {
        info!("First 10 files: {:?}", &files[..10]);
    }
    
    // Determine mod type using the detailed function
    use crate::utils::get_pak_characteristics_detailed;
    let characteristics = get_pak_characteristics_detailed(files.clone());
    info!("Detected mod type: {}", characteristics.mod_type);
    info!("Character name: {}", characteristics.character_name);
    info!("Category: {}", characteristics.category);
    
    // Run fast Blueprint detection using filename heuristics
    let has_blueprint = files.iter().any(|f| {
        let filename = f.split('/').last().unwrap_or("");
        let name_lower = filename.to_lowercase();
        let path_lower = f.to_lowercase();
        
        // Common Blueprint patterns:
        // 1. BP_Something (Blueprint prefix)
        // 2. Something_C (Blueprint class suffix)
        // 3. SomethingBP (Blueprint suffix without underscore)
        // 4. /Blueprints/ folder path
        name_lower.starts_with("bp_") || 
        name_lower.contains("_c.") ||
        name_lower.contains("bp.") ||
        name_lower.ends_with("bp") ||
        path_lower.contains("/blueprints/")
    });
    
    if has_blueprint {
        info!("Blueprint detected via filename patterns");
    }
    
    // Get total size
    let ucas_path_for_size = path.with_extension("ucas");
    let total_size = if ucas_path_for_size.exists() {
        std::fs::metadata(&ucas_path_for_size)
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0)
    };
    
    let mod_name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    Ok(ModDetails {
        mod_name,
        mod_type: characteristics.mod_type,
        character_name: characteristics.character_name,
        category: characteristics.category,
        file_count,
        total_size,
        files,
        is_iostore,
        has_blueprint,
    })
}

#[derive(Clone, Serialize, Deserialize)]
struct ModClash {
    file_path: String,
    mod_paths: Vec<String>,
}

#[tauri::command]
async fn check_mod_clashes(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<ModClash>, String> {
    use repak::PakBuilder;
    use repak::utils::AesKey;
    use std::str::FromStr;
    use std::fs::File;
    use std::io::BufReader;
    use std::collections::HashMap;
    
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    info!("Checking for mod clashes...");
    
    if !game_path.exists() {
        return Err("Game path does not exist".to_string());
    }

    // Get AES key
    let aes_key = AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
        .map_err(|e| format!("Failed to create AES key: {}", e))?;

    // Structure to hold mod info for clash detection
    #[derive(Clone)]
    struct ModInfo {
        path: PathBuf,
        priority: usize,
        files: Vec<String>,      // List of files inside this mod
    }

    let mut mods_info: Vec<ModInfo> = Vec::new();

    // Scan all enabled mods
    for entry in WalkDir::new(&game_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let ext = path.extension().and_then(|s| s.to_str());

        // Only check enabled .pak files
        if ext != Some("pak") {
            continue;
        }

        // Calculate priority (same as get_pak_files)
        let mut priority = 0;
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // Check for "!" prefix (highest priority)
        if file_stem.starts_with("!") {
            priority = 0; // Highest priority
        } else if file_stem.ends_with("_P") {
            let base_no_p = file_stem.strip_suffix("_P").unwrap();
            let re_nums = Regex::new(r"_(\d+)$").unwrap();
            if let Some(caps) = re_nums.captures(base_no_p) {
                let nums = &caps[1];
                if nums.chars().all(|c| c == '9') {
                    let actual_nines = nums.len();
                    // Convert actual nines count to UI priority (1-based)
                    if actual_nines >= 7 {
                        priority = actual_nines - 6;
                    }
                }
            }
        }

        // Open PAK file to analyze contents
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to open PAK file {:?}: {}", path, e);
                continue;
            }
        };

        let mut reader = BufReader::new(file);
        let pak = match PakBuilder::new()
            .key(aes_key.0.clone())
            .reader(&mut reader) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to read PAK {:?}: {}", path, e);
                continue;
            }
        };

        // Check if it's IoStore
        let mut utoc_path = path.to_path_buf();
        utoc_path.set_extension("utoc");
        let is_iostore = utoc_path.exists();

        // Get file list
        let files: Vec<String> = if is_iostore {
            use crate::utoc_utils::read_utoc;
            read_utoc(&utoc_path, &pak, &path)
                .iter()
                .map(|entry| entry.file_path.clone())
                .collect()
        } else {
            pak.files()
        };


        mods_info.push(ModInfo {
            path: path.to_path_buf(),
            priority,
            files,
        });
    }

    info!("Analyzed {} enabled mods", mods_info.len());

    // Don't group by character - instead, compare all mods at the same priority level
    // Group by priority first
    let mut by_priority: HashMap<usize, Vec<ModInfo>> = HashMap::new();
    
    for mod_info in mods_info {
        by_priority.entry(mod_info.priority).or_insert_with(Vec::new).push(mod_info);
    }

    // Find clashes: same priority and overlapping files
    let mut clashes: Vec<ModClash> = Vec::new();
    use std::collections::HashSet;

    for (priority, same_priority_mods) in by_priority {
        if same_priority_mods.len() < 2 {
            continue;
        }

        info!("Checking priority {} with {} mods", priority, same_priority_mods.len());

        // Compare each pair of mods at this priority level
        for i in 0..same_priority_mods.len() {
            for j in (i + 1)..same_priority_mods.len() {
                let mod1 = &same_priority_mods[i];
                let mod2 = &same_priority_mods[j];

                // Convert file lists to HashSets for efficient intersection
                let files1: HashSet<&String> = mod1.files.iter().collect();
                let files2: HashSet<&String> = mod2.files.iter().collect();

                // Find overlapping files
                let overlapping_files: Vec<String> = files1
                    .intersection(&files2)
                    .map(|s| (*s).clone())
                    .collect();

                if !overlapping_files.is_empty() {
                    // Found a clash! These two mods modify the same files
                    let mod_paths = vec![
                        mod1.path.to_string_lossy().to_string(),
                        mod2.path.to_string_lossy().to_string(),
                    ];

                    // Build a description showing which characters are affected
                    let mut affected_characters = HashSet::new();
                    
                    // Extract character IDs from overlapping file paths
                    for file_path in &overlapping_files {
                        // Look for pattern like "Characters/1050/" or "1050/1050800/"
                        if let Some(char_match) = file_path.split('/').find(|s| {
                            s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) && s.starts_with("10")
                        }) {
                            affected_characters.insert(char_match.to_string());
                        }
                    }

                    let character_info = if !affected_characters.is_empty() {
                        let char_ids: Vec<String> = affected_characters.iter().cloned().collect();
                        format!("Characters: {} - ", char_ids.join(", "))
                    } else {
                        String::new()
                    };

                    let clash_description = format!(
                        "{}Priority: {} - {} overlapping file(s)",
                        character_info,
                        priority,
                        overlapping_files.len()
                    );

                    info!(
                        "Found clash between {} and {} at priority {} ({} overlapping files, characters: {:?})",
                        mod1.path.file_name().unwrap_or_default().to_string_lossy(),
                        mod2.path.file_name().unwrap_or_default().to_string_lossy(),
                        priority,
                        overlapping_files.len(),
                        affected_characters
                    );

                    clashes.push(ModClash {
                        file_path: clash_description,
                        mod_paths,
                    });
                }
            }
        }
    }
    info!("Found {} clashes", clashes.len());
    Ok(clashes)
}

// ============================================================================
// P2P SHARING COMMANDS
// ============================================================================

/// Start sharing a mod pack
#[tauri::command]
async fn p2p_start_sharing(
    name: String,
    description: String,
    mod_paths: Vec<String>,
    creator: Option<String>,
    p2p_state: State<'_, P2PState>,
) -> Result<p2p_libp2p::ShareInfo, String> {
    let paths: Vec<PathBuf> = mod_paths.iter().map(PathBuf::from).collect();
    
    p2p_state.manager
        .start_sharing(name, description, paths, creator)
        .await
        .map_err(|e| e.to_string())
}

/// Stop sharing
#[tauri::command]
async fn p2p_stop_sharing(share_code: String, p2p_state: State<'_, P2PState>) -> Result<(), String> {
    p2p_state.manager.stop_sharing(&share_code)
        .map_err(|e| e.to_string())
}

/// Get current share session info
#[tauri::command]
async fn p2p_get_share_session(p2p_state: State<'_, P2PState>) -> Result<Option<p2p_libp2p::ShareInfo>, String> {
    // Return the first active share if any
    let shares = p2p_state.manager.active_shares.lock();
    Ok(shares.values().next().map(|s| s.session.clone()).and_then(|session| {
        // Convert ShareSession to ShareInfo
        p2p_libp2p::ShareInfo::decode(&session.connection_string).ok()
    }))
}

/// Check if currently sharing
#[tauri::command]
async fn p2p_is_sharing(p2p_state: State<'_, P2PState>) -> Result<bool, String> {
    Ok(!p2p_state.manager.active_shares.lock().is_empty())
}

/// Start receiving mods from a connection string
#[tauri::command]
async fn p2p_start_receiving(
    connection_string: String,
    client_name: Option<String>,
    window: Window,
    state: State<'_, Arc<Mutex<AppState>>>,
    p2p_state: State<'_, P2PState>,
) -> Result<(), String> {
    let output_dir = {
        let state_guard = state.lock().unwrap();
        state_guard.game_path.clone()
    };
    
    p2p_state.manager
        .start_receiving(&connection_string, output_dir, client_name, window)
        .await
        .map_err(|e| e.to_string())
}

/// Stop receiving
#[tauri::command]
async fn p2p_stop_receiving(p2p_state: State<'_, P2PState>) -> Result<(), String> {
    // Clear all active downloads
    p2p_state.manager.active_downloads.lock().clear();
    Ok(())
}

/// Get current transfer progress
#[tauri::command]
async fn p2p_get_receive_progress(p2p_state: State<'_, P2PState>) -> Result<Option<p2p_sharing::TransferProgress>, String> {
    let downloads = p2p_state.manager.active_downloads.lock();
    
    // Return the first active download's progress (typically only one at a time)
    if let Some((_, download)) = downloads.iter().next() {
        Ok(Some(download.progress.clone()))
    } else {
        Ok(None)
    }
}

/// Check if currently receiving
#[tauri::command]
async fn p2p_is_receiving(p2p_state: State<'_, P2PState>) -> Result<bool, String> {
    Ok(!p2p_state.manager.active_downloads.lock().is_empty())
}

/// Create a shareable mod pack info (for preview before sharing)
#[tauri::command]
async fn p2p_create_mod_pack_preview(
    name: String,
    description: String,
    mod_paths: Vec<String>,
    creator: Option<String>,
) -> Result<p2p_sharing::ShareableModPack, String> {
    let paths: Vec<PathBuf> = mod_paths.iter().map(PathBuf::from).collect();
    p2p_sharing::create_mod_pack(name, description, &paths, creator)
        .map_err(|e| e.to_string())
}

/// Validate a connection string without connecting
#[tauri::command]
async fn p2p_validate_connection_string(connection_string: String) -> Result<bool, String> {
    // Validate base64 ShareInfo format
    match p2p_libp2p::ShareInfo::decode(&connection_string) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

/// Calculate hash for a file (useful for verification)
#[tauri::command]
async fn p2p_hash_file(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(file_path);
    p2p_sharing::hash_file(&path).map_err(|e| e.to_string())
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let log_dir = exe_dir.join("Logs");
            if let Err(e) = std::fs::create_dir_all(&log_dir) {
                eprintln!("Failed to create log directory {:?}: {}", log_dir, e);
            } else {
                let startup_log = log_dir.join("startup.log");
                let _ = std::fs::write(&startup_log, format!(
                    "Repak-Gui (Tauri) startup at {:?}\n",
                    std::time::SystemTime::now()
                ));
            }
        }
    }

    setup_logging();
    info!("Starting Repak Gui Revamped v{}", env!("CARGO_PKG_VERSION"));
    
    // Initialize character data cache on startup
    info!("Initializing character data cache...");
    character_data::refresh_cache();
    
    let state = Arc::new(Mutex::new(load_state()));
    let watcher_state = WatcherState { 
        watcher: Mutex::new(None),
        last_event_time: Mutex::new(std::time::Instant::now()),
    };
    let crash_state = CrashMonitorState {
        game_start_time: Mutex::new(None),
        last_checked_crash: Mutex::new(None),
    };
    let p2p_manager = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(p2p_manager::UnifiedP2PManager::new())
        .expect("Failed to initialize P2P network");
    let p2p_state = P2PState { manager: Arc::new(p2p_manager) };

    tauri::Builder::default()
        .manage(state)
        .manage(watcher_state)
        .manage(crash_state)
        .manage(p2p_state)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_game_path,
            set_game_path,
            auto_detect_game_path,
            start_file_watcher,
            get_pak_files,
            parse_dropped_files,
            install_mods,
            quick_organize,
            delete_mod,
            create_folder,
            get_folders,
            get_root_folder_info,
            update_folder,
            delete_folder,
            assign_mod_to_folder,
            add_custom_tag,
            // USMAP management commands
            copy_usmap_to_folder,
            set_usmap_path,
            get_usmap_path,
            get_usmap_dir_path,
            list_usmap_files,
            get_current_usmap_file,
            get_current_usmap_full_path,
            delete_current_usmap,
            get_all_tags,
            toggle_mod,
            check_game_running,
            launch_game,
            skip_launcher_patch,
            get_skip_launcher_status,
            get_app_version,
            check_for_updates,
            monitor_game_for_crashes,
            get_crash_history,
            get_total_crashes,
            clear_crash_logs,
            dismiss_crash_dialog,
            get_crash_log_path,
            get_mod_details,
            set_mod_priority,
            check_mod_clashes,
            extract_pak_to_destination,
            // Character data commands
            get_character_data,
            get_character_by_skin_id,
            update_character_data_from_github,
            cancel_character_update,
            identify_mod_character,
            get_character_data_path,
            refresh_character_cache,
            // P2P sharing commands
            p2p_start_sharing,
            p2p_stop_sharing,
            p2p_get_share_session,
            p2p_is_sharing,
            p2p_start_receiving,
            p2p_stop_receiving,
            p2p_get_receive_progress,
            p2p_is_receiving,
            p2p_create_mod_pack_preview,
            p2p_validate_connection_string,
            p2p_hash_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
