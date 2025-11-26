// Tauri-based main.rs - React + Tauri implementation
// Original egui implementation backed up in src/egui_backup_original/

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod install_mod;
mod uasset_detection;
mod uasset_api_integration;
mod utils;
mod utoc_utils;

use install_mod::InstallableMod;
use log::{info, error};
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
    
    // Create a new watcher
    let window_clone = window.clone();
    let watcher_result = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                // We only care about Create, Remove, Rename, and Modify events on files
                match event.kind {
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                         // Debouncing or simple filtering can be done here if needed.
                         // For now, just emit 'mods_dir_changed'
                         window_clone.emit("mods_dir_changed", ()).unwrap_or_else(|e| {
                             error!("Failed to emit mods_dir_changed: {}", e);
                         });
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
    
    // Scan root ~mods directory and all subdirectories (folders)
    for entry in WalkDir::new(&game_path)
        .max_depth(2) // Scan root and one level of subdirectories
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
            
            // Determine which folder this mod is in (if any)
            let folder_id = if let Some(parent) = path.parent() {
                if parent != game_path {
                    parent.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            info!("Found PAK file: {} (enabled: {}, folder: {:?})", path.display(), is_enabled, folder_id);
            
            let metadata = state.mod_metadata.iter()
                .find(|m| {
                    m.path == path || 
                    m.path.with_extension("pak") == path || 
                    m.path.with_extension("bak_repak") == path ||
                    m.path.with_extension("pak_disabled") == path
                });
            
            let file_size = std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0);
            
            // Calculate priority (number of 9s)
            let mut priority = 0;
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            
            if file_stem.ends_with("_P") {
                let base_no_p = file_stem.strip_suffix("_P").unwrap();
                // Check for _999... suffix
                let re_nums = Regex::new(r"_(\d+)$").unwrap();
                if let Some(caps) = re_nums.captures(base_no_p) {
                    let nums = &caps[1];
                    // Verify they are all 9s
                    if nums.chars().all(|c| c == '9') {
                        priority = nums.len();
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
    
    // 1. Strip _P if present
    let base_no_p = if stem.ends_with("_P") {
        stem.strip_suffix("_P").unwrap()
    } else {
        stem
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
    let new_nines = "9".repeat(priority);
    let new_stem = format!("{}_{}_P", clean_base, new_nines);
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
async fn parse_dropped_files(paths: Vec<String>) -> Result<Vec<InstallableModInfo>, String> {
    use crate::utils::get_current_pak_characteristics;
    use repak::PakBuilder;
    use repak::utils::AesKey;
    use std::str::FromStr;
    use std::fs::File;
    use std::io::BufReader;
    
    let mut mods = Vec::new();
    
    for path_str in paths {
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
            ("Directory".to_string(), false, false, false)
        } else if path.extension().and_then(|s| s.to_str()) == Some("pak") {
            // Try to read PAK and determine type
            if let Ok(file) = File::open(&path) {
                // Create AES key for each file
                if let Ok(aes_key) = AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74") {
                    let mut reader = BufReader::new(file);
                    if let Ok(pak) = PakBuilder::new().key(aes_key.0).reader(&mut reader) {
                        let files: Vec<String> = pak.files();
                        let mod_type = get_current_pak_characteristics(files.clone());
                        
                        // Auto-detect mesh and texture files
                        use crate::uasset_detection::{detect_mesh_files, detect_texture_files, detect_static_mesh_files};
                        
                        // detect_mesh_files = skeletal meshes (sk_*) -> Fix Mesh
                        let has_skeletal_mesh = detect_mesh_files(&files);
                        
                        // detect_static_mesh_files = static meshes (sm_*) -> Fix SerializeSize
                        let has_static_mesh = detect_static_mesh_files(&files);
                        
                        // detect_texture_files = textures -> Fix Texture
                        let has_texture = detect_texture_files(&files);
                        
                        info!("Auto-detection for {}: skeletal={}, static={}, texture={}", 
                              mod_name, has_skeletal_mesh, has_static_mesh, has_texture);
                        
                        (mod_type, has_skeletal_mesh, has_texture, has_static_mesh)
                    } else {
                        ("Unknown".to_string(), false, false, false)
                    }
                } else {
                    ("Unknown".to_string(), false, false, false)
                }
            } else {
                ("Unknown".to_string(), false, false, false)
            }
        } else {
            ("Unknown".to_string(), false, false, false)
        };
        
        // For .pak files, auto-enable repak
        let auto_to_repak = path.extension().and_then(|s| s.to_str()) == Some("pak");
        
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

#[tauri::command]
async fn install_mods(
    mods: Vec<ModToInstall>,
    window: Window,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    use crate::install_mod::{InstallableMod, ModInstallRequest};
    use std::sync::atomic::{AtomicI32, AtomicBool};
    use std::sync::Arc as StdArc;
    
    let state_guard = state.lock().unwrap();
    let mod_directory = state_guard.game_path.clone();
    let usmap_path = state_guard.usmap_path.clone();
    drop(state_guard);

    // Propagate USMAP path to UAssetBridge via environment for UAssetAPI-based processing
    if !usmap_path.is_empty() {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let usmap_full_path = exe_dir.join("Usmap").join(&usmap_path);
                if usmap_full_path.exists() {
                    std::env::set_var("USMAP_PATH", &usmap_full_path);
                    info!(
                        "Set USMAP_PATH for UAssetBridge: {}",
                        usmap_full_path.display()
                    );
                } else {
                    error!(
                        "USMAP file not found at expected path for UAssetBridge: {}",
                        usmap_full_path.display()
                    );
                }
            }
        }
    }

    if !mod_directory.exists() {
        std::fs::create_dir_all(&mod_directory)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    }

    // Convert paths to properly initialized InstallableMods
    use crate::install_mod::map_paths_to_mods;
    
    let paths: Vec<PathBuf> = mods.iter().map(|m| PathBuf::from(&m.path)).collect();
    let mut installable_mods = map_paths_to_mods(&paths);
    
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
            installable.usmap_path = usmap_path.clone();
        }
    }

    // Use existing installation logic
    let installed_counter = StdArc::new(AtomicI32::new(0));
    let stop_flag = StdArc::new(AtomicBool::new(false));
    
    let total = installable_mods.len() as i32;
    let counter_clone = installed_counter.clone();
    let stop_clone = stop_flag.clone();
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
    let path = PathBuf::from(path);
    std::fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete mod: {}", e))?;
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
    
    std::fs::create_dir(&folder_path)
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
    
    // Scan for subdirectories in ~mods
    for entry in std::fs::read_dir(game_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_dir() {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            // Count mods in this folder
            let mod_count = WalkDir::new(&path)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let ext = e.path().extension().and_then(|s| s.to_str());
                    ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled")
                })
                .count();
            
            folders.push(ModFolder {
                id: name.clone(),
                name,
                enabled: true,
                expanded: true,
                color: None,
            });
        }
    }
    
    Ok(folders)
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

#[tauri::command]
async fn copy_usmap_to_folder(source_path: String) -> Result<String, String> {
    let source = PathBuf::from(&source_path);
    
    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }
    
    // Get exe directory
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?
        .parent()
        .ok_or("Failed to get exe directory")?
        .to_path_buf();
    
    // Create Usmap/ folder
    let usmap_dir = exe_dir.join("Usmap");
    std::fs::create_dir_all(&usmap_dir)
        .map_err(|e| format!("Failed to create Usmap directory: {}", e))?;
    
    // Get filename from source
    let filename = source.file_name()
        .ok_or("Invalid filename")?
        .to_str()
        .ok_or("Invalid UTF-8 in filename")?;
    
    // Copy file to Usmap/ folder
    let dest_path = usmap_dir.join(filename);
    std::fs::copy(&source, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    
    info!("Copied USmap file {} to Usmap folder", filename);
    
    // Return just the filename
    Ok(filename.to_string())
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
// HELPER FUNCTIONS
// ============================================================================

fn app_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RepakGuiRevamped")
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
    if let Ok(file) = File::open(path) {
        if let Ok(state) = serde_json::from_reader(file) {
            return state;
        }
    }
    AppState::default()
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
    file_count: usize,
    total_size: u64,
    files: Vec<String>,
    is_iostore: bool,
}

#[tauri::command]
async fn get_mod_details(mod_path: String) -> Result<ModDetails, String> {
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
    
    // Determine mod type using the same function as egui
    use crate::utils::get_current_pak_characteristics;
    let mod_type = get_current_pak_characteristics(files.clone());
    info!("Detected mod type: {}", mod_type);
    
    // Get total size
    let total_size = std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let mod_name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    Ok(ModDetails {
        mod_name,
        mod_type,
        file_count,
        total_size,
        files,
        is_iostore,
    })
}

fn determine_mod_type(files: &[String]) -> String {
    use crate::utils::get_current_pak_characteristics;
    get_current_pak_characteristics(files.to_vec())
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
    
    let state = Arc::new(Mutex::new(load_state()));
    let watcher_state = WatcherState { watcher: Mutex::new(None) };

    tauri::Builder::default()
        .manage(state)
        .manage(watcher_state)
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
            delete_mod,
            create_folder,
            get_folders,
            update_folder,
            delete_folder,
            assign_mod_to_folder,
            add_custom_tag,
            copy_usmap_to_folder,
            get_all_tags,
            toggle_mod,
            check_game_running,
            get_app_version,
            check_for_updates,
            get_mod_details,
            set_mod_priority,
            extract_pak_to_destination
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
