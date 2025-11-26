// Application State - Business Logic
// Ported from egui RepakModManager
// This contains all core functionality separate from UI

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use std::{fs, thread};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, BTreeSet};
use std::io::BufReader;
use std::fs::File;

use serde::{Deserialize, Serialize};
use log::{debug, error, info, warn};
use notify::{Event, RecommendedWatcher};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use uuid::Uuid;
use walkdir::WalkDir;
// Re-export types that will be used
pub use repak::PakReader;
use repak::utils::AesKey;
use std::sync::LazyLock;
use std::str::FromStr;

// Marvel Rivals AES encryption key
static AES_KEY: LazyLock<AesKey> = LazyLock::new(|| {
    AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
        .expect("Unable to initialise AES_KEY")
});

// Supporting types that need to be defined
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CustomPalette {
    pub accent: [u8; 4],
    pub panel_fill: [u8; 4],
    pub window_fill: [u8; 4],
    pub widget_inactive: [u8; 4],
    pub widget_hovered: [u8; 4],
    pub widget_active: [u8; 4],
    pub widget_open: [u8; 4],
    pub text: Option<[u8; 4]>,
    #[serde(default = "CustomPalette::default_toggle_on")]
    pub toggle_on_bg: [u8; 4],
    #[serde(default = "CustomPalette::default_toggle_off")]
    pub toggle_off_bg: [u8; 4],
    #[serde(default = "CustomPalette::default_toggle_border")]
    pub toggle_border: [u8; 4],
}

impl Default for CustomPalette {
    fn default() -> Self {
        Self {
            accent: [0xff, 0x6b, 0x9d, 0xff],
            panel_fill: [0x1f, 0x29, 0x37, 0xff],
            window_fill: [0x11, 0x18, 0x27, 0xff],
            widget_inactive: [0x2a, 0x2d, 0x3a, 0xff],
            widget_hovered: [0x3d, 0x43, 0x54, 0xff],
            widget_active: [0x4a, 0x55, 0x68, 0xff],
            widget_open: [0x55, 0x3c, 0x4e, 0xff],
            text: Some([0xf9, 0xfa, 0xfb, 0xff]),
            toggle_on_bg: [0xff, 0x6b, 0x9d, 0xc8],
            toggle_off_bg: [0x78, 0x78, 0x8c, 0x96],
            toggle_border: Self::default_toggle_border(),
        }
    }
}

impl CustomPalette {
    fn default_toggle_on() -> [u8; 4] { [0xff, 0x6b, 0x9d, 0xc8] }
    fn default_toggle_off() -> [u8; 4] { [0x78, 0x78, 0x8c, 0x96] }
    fn default_toggle_border() -> [u8; 4] { [0xff, 0x9f, 0xf3, 0xb4] }
}

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub latest: String,
    pub url: String,
    pub asset_url: Option<String>,
    pub asset_name: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ModFolder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub expanded: bool,
}

#[derive(Clone, Debug)]
pub struct ModEntry {
    pub path: PathBuf,
    pub enabled: bool,
    pub reader: PakReader,
    pub folder_id: Option<String>,
    pub custom_tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ModMetadata {
    pub pak_name: String,
    pub folder_id: Option<String>,
    pub custom_tags: Vec<String>,
}

// Main application state
#[derive(Deserialize, Serialize)]
pub struct AppState {
    // Configuration
    pub game_path: PathBuf,
    pub default_font_size: f32,
    pub folders: Vec<ModFolder>,
    #[serde(default)]
    pub mod_metadata: Vec<ModMetadata>,
    
    // UI state (not serialized)
    #[serde(skip)]
    pub current_pak_file_idx: Option<usize>,
    #[serde(skip)]
    pub pak_files: Vec<ModEntry>,
    #[serde(skip)]
    pub selected_file_in_table: Option<usize>,
    #[serde(skip)]
    pub search_query: String,
    #[serde(skip)]
    pub filtered_mods: Vec<usize>,
    #[serde(skip)]
    pub expanded_folders_for_search: HashSet<String>,
    
    // Filtering
    #[serde(skip)]
    pub tag_filter_enabled: bool,
    #[serde(skip)]
    pub selected_tag_filters: HashSet<String>,
    #[serde(skip)]
    pub show_tag_filter_dropdown: bool,
    #[serde(skip)]
    pub custom_tag_filter_enabled: bool,
    #[serde(skip)]
    pub selected_custom_tag_filters: HashSet<String>,
    #[serde(default)]
    pub custom_tag_catalog: Vec<String>,
    #[serde(skip)]
    pub show_custom_tag_filter_dropdown: bool,
    
    // Tag management
    #[serde(skip)]
    pub show_tag_manager: bool,
    #[serde(skip)]
    pub new_global_tag_input: String,
    #[serde(skip)]
    pub rename_tag_from: Option<String>,
    #[serde(skip)]
    pub rename_tag_to: String,
    
    // Selection mode
    #[serde(skip)]
    pub selection_mode: bool,
    #[serde(skip)]
    pub selected_mods: BTreeSet<usize>,
    #[serde(skip)]
    pub bulk_tag_input: String,
    
    // UI settings
    pub version: Option<String>,
    #[serde(default)]
    pub use_custom_palette: bool,
    #[serde(default)]
    pub use_custom_font: bool,
    #[serde(default)]
    pub custom_palette: CustomPalette,
    #[serde(skip)]
    pub show_palette_window: bool,
    #[serde(skip)]
    pub show_settings: bool,
    
    // More UI settings
    #[serde(default)]
    pub hide_internal_suffix: bool,
    #[serde(default = "AppState::default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default)]
    pub compact_mode: bool,
    #[serde(default)]
    pub usmap_path: String,
    #[serde(default)]
    pub auto_check_updates: bool,
    #[serde(default)]
    pub confirm_on_delete: bool,
    #[serde(default)]
    pub apply_palette_in_light_mode: bool,
    #[serde(default)]
    pub show_tag_chips: bool,
    
    // Caching
    #[serde(skip)]
    pub mod_type_cache: RefCell<HashMap<PathBuf, String>>,
    #[serde(skip)]
    pub all_mod_types_cache: Vec<String>,
    #[serde(skip)]
    pub mod_types_dirty: bool,
    
    // Update system
    #[serde(skip)]
    pub update_info: Option<UpdateInfo>,
    #[serde(skip)]
    #[allow(clippy::type_complexity)]
    pub update_rx: Option<Receiver<Result<UpdateInfo, String>>>,
    #[serde(skip)]
    pub update_in_flight: bool,
    #[serde(skip)]
    pub last_update_error: Option<String>,
    #[serde(skip)]
    pub update_manual_last: bool,
    
    // File watching
    #[serde(skip)]
    pub receiver: Option<Receiver<Event>>,
    #[serde(skip)]
    pub pending_restart: bool,
    
    // Deletion tracking
    #[serde(skip)]
    pub deleting_mods: HashSet<PathBuf>,
    #[serde(skip)]
    pub pending_remove_paths: Vec<PathBuf>,
    
    // Folder management
    #[serde(skip)]
    pub creating_folder: bool,
    #[serde(skip)]
    pub new_folder_name: String,
    #[serde(skip)]
    pub game_path_input: String,
    
    // Mod installation
    #[serde(skip)]
    pub show_install_dialog: bool,
    #[serde(skip)]
    pub pending_install_mods: Vec<crate::install_mod_core::InstallableMod>,
    #[serde(skip)]
    pub install_fix_mesh: bool,
    #[serde(skip)]
    pub install_fix_texture: bool,
    #[serde(skip)]
    pub install_to_iostore: bool,
    #[serde(skip)]
    pub install_progress: f32,
    #[serde(skip)]
    pub install_status: String,
    #[serde(skip)]
    pub installing: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            game_path: PathBuf::new(),
            default_font_size: 18.0,
            folders: Vec::new(),
            mod_metadata: Vec::new(),
            current_pak_file_idx: None,
            pak_files: Vec::new(),
            selected_file_in_table: None,
            search_query: String::new(),
            filtered_mods: Vec::new(),
            expanded_folders_for_search: HashSet::new(),
            tag_filter_enabled: false,
            selected_tag_filters: HashSet::new(),
            show_tag_filter_dropdown: false,
            custom_tag_filter_enabled: false,
            selected_custom_tag_filters: HashSet::new(),
            custom_tag_catalog: Vec::new(),
            show_custom_tag_filter_dropdown: false,
            show_tag_manager: false,
            new_global_tag_input: String::new(),
            rename_tag_from: None,
            rename_tag_to: String::new(),
            selection_mode: false,
            selected_mods: BTreeSet::new(),
            bulk_tag_input: String::new(),
            version: None,
            use_custom_palette: false,
            use_custom_font: false,
            custom_palette: CustomPalette::default(),
            show_palette_window: false,
            show_settings: false,
            hide_internal_suffix: false,
            ui_scale: Self::default_ui_scale(),
            compact_mode: false,
            usmap_path: String::new(),
            auto_check_updates: false,
            confirm_on_delete: false,
            apply_palette_in_light_mode: false,
            show_tag_chips: false,
            mod_type_cache: RefCell::new(HashMap::new()),
            all_mod_types_cache: Vec::new(),
            mod_types_dirty: false,
            update_info: None,
            update_rx: None,
            update_in_flight: false,
            last_update_error: None,
            update_manual_last: false,
            receiver: None,
            pending_restart: false,
            deleting_mods: HashSet::new(),
            pending_remove_paths: Vec::new(),
            creating_folder: false,
            new_folder_name: String::new(),
            game_path_input: String::new(),
            show_install_dialog: false,
            pending_install_mods: Vec::new(),
            install_fix_mesh: false,
            install_fix_texture: false,
            install_to_iostore: true,
            install_progress: 0.0,
            install_status: String::new(),
            installing: false,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn default_ui_scale() -> f32 {
        1.0
    }
    
    // Configuration management
    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("repak_mod_manager");
        fs::create_dir_all(&path).ok();
        path.push("repak_mod_manager.json");
        path
    }
    
    pub fn load_config() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        let mut state: Self = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        state.game_path_input = state.game_path.to_string_lossy().to_string();
        
        // Load mods if game path exists
        if state.game_path.exists() {
            info!("Loading mods from config: {}", state.game_path.to_string_lossy());
            state.collect_pak_files();
            state.update_search_filter();
        }
        
        Ok(state)
    }
    
    pub fn save_config(&mut self) -> Result<(), String> {
        // Sync metadata before saving
        self.sync_metadata();
        
        let path = Self::config_path();
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&path, contents)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        
        info!("Saved config: {}", path.to_string_lossy());
        Ok(())
    }
    
    fn sync_metadata(&mut self) {
        // Clear existing metadata and rebuild from current pak_files
        self.mod_metadata.clear();
        
        for pak_file in &self.pak_files {
            let pak_name = pak_file.path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let metadata = ModMetadata {
                pak_name,
                folder_id: pak_file.folder_id.clone(),
                custom_tags: pak_file.custom_tags.clone(),
            };
            self.mod_metadata.push(metadata);
        }
    }
    
    // Game detection
    pub fn is_game_running() -> bool {
        let s = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::new())
        );
        let game_process_names = ["Marvel-Win64-Shipping.exe"];
        
        for (_pid, process) in s.processes() {
            let process_name = process.name().to_string_lossy().to_lowercase();
            for game_name in &game_process_names {
                if process_name == game_name.to_lowercase() {
                    return true;
                }
            }
        }
        false
    }
    
    // Search and filtering
    pub fn update_search_filter(&mut self) {
        self.filtered_mods.clear();
        self.expanded_folders_for_search.clear();
        
        let has_search = !self.search_query.trim().is_empty();
        let has_tag_filter = self.tag_filter_enabled && !self.selected_tag_filters.is_empty();
        let has_custom_tag_filter = self.custom_tag_filter_enabled 
            && !self.selected_custom_tag_filters.is_empty();
        
        if !has_search && !has_tag_filter && !has_custom_tag_filter {
            return;
        }
        
        let query_lower = self.search_query.to_lowercase();
        
        for (idx, mod_entry) in self.pak_files.iter().enumerate() {
            let mut matches = true;
            
            // Search query matching
            if has_search {
                let name = mod_entry.path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                matches = name.to_lowercase().contains(&query_lower);
            }
            
            // Tag filter matching
            if matches && has_tag_filter {
                // TODO: Implement tag matching
                // For now, just pass through
            }
            
            // Custom tag filter matching
            if matches && has_custom_tag_filter {
                let has_any_selected_tag = mod_entry.custom_tags.iter()
                    .any(|t| self.selected_custom_tag_filters.contains(t));
                matches = has_any_selected_tag;
            }
            
            if matches {
                self.filtered_mods.push(idx);
                if let Some(folder_id) = &mod_entry.folder_id {
                    self.expanded_folders_for_search.insert(folder_id.clone());
                }
            }
        }
    }
    
    // Mod visibility
    pub fn is_mod_visible(&self, mod_index: usize) -> bool {
        if let Some(p) = self.pak_files.get(mod_index).map(|m| m.path.clone()) {
            if self.deleting_mods.contains(&p) {
                return false;
            }
        }
        
        let has_search = !self.search_query.trim().is_empty();
        let has_tag_filter = self.tag_filter_enabled && !self.selected_tag_filters.is_empty();
        let has_custom_tag_filter = self.custom_tag_filter_enabled
            && !self.selected_custom_tag_filters.is_empty();
        
        if !has_search && !has_tag_filter && !has_custom_tag_filter {
            return true;
        }
        
        self.filtered_mods.contains(&mod_index)
    }
    
    // Tag operations
    pub fn get_all_custom_tags(&self) -> BTreeSet<String> {
        let mut tags = BTreeSet::new();
        for t in &self.custom_tag_catalog {
            tags.insert(t.clone());
        }
        for pak_file in &self.pak_files {
            for t in &pak_file.custom_tags {
                tags.insert(t.clone());
            }
        }
        tags
    }
    
    pub fn bulk_add_tag_to_selected(&mut self, tag: &str) {
        if tag.trim().is_empty() {
            return;
        }
        
        for &i in &self.selected_mods {
            if let Some(p) = self.pak_files.get_mut(i) {
                if !p.custom_tags.contains(&tag.to_string()) {
                    p.custom_tags.push(tag.to_string());
                    p.custom_tags.sort();
                    p.custom_tags.dedup();
                }
            }
        }
        
        if !self.custom_tag_catalog.contains(&tag.to_string()) {
            self.custom_tag_catalog.push(tag.to_string());
            self.custom_tag_catalog.sort();
            self.custom_tag_catalog.dedup();
        }
        
        self.update_search_filter();
        let _ = self.save_config();
    }
    
    // Mod loading
    pub fn load_pak_files(&mut self) {
        if !self.game_path.exists() {
            warn!("Game path does not exist: {:?}", self.game_path);
            return;
        }
        
        info!("Loading pak files from: {:?}", self.game_path);
        self.collect_pak_files();
    }
    
    fn collect_pak_files(&mut self) {
        if !self.game_path.exists() {
            warn!("Game path does not exist: {:?}", self.game_path);
            return;
        }
        
        info!("Scanning for mods in: {:?}", self.game_path);
        let mut mods = vec![];
        let mut files_found = 0;
        let mut pak_files_found = 0;
        
        for entry in WalkDir::new(&self.game_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            files_found += 1;
            let path = entry.path();
            
            let mut disabled = false;
            
            // Check file extension
            let ext = path.extension().unwrap_or_default();
            if ext != "pak" {
                // Check for disabled extensions
                if ext == "pak_disabled" || ext == "bak_repak" {
                    disabled = true;
                } else {
                    continue;
                }
            }
            
            pak_files_found += 1;
            info!("Found pak file: {:?}", path);
            
            // Try to open the pak file with AES key
            let mut builder = repak::PakBuilder::new();
            builder = builder.key(AES_KEY.clone().0);
            
            let pak_result = File::open(path)
                .map_err(|e| format!("Failed to open file: {}", e))
                .and_then(|f| {
                    builder.reader(&mut BufReader::new(f))
                        .map_err(|e| format!("Failed to read pak: {}", e))
                });
            
            let pak = match pak_result {
                Ok(p) => p,
                Err(e) => {
                    warn!("Error opening pak file {:?}: {}", path, e);
                    continue;
                }
            };
            
            // Find existing metadata for this path
            let metadata = self.mod_metadata.iter()
                .find(|m| m.pak_name == path.file_name().unwrap_or_default().to_string_lossy());
            
            let entry = ModEntry {
                reader: pak,
                path: path.to_path_buf(),
                enabled: !disabled,
                folder_id: metadata.and_then(|m| m.folder_id.clone()),
                custom_tags: metadata
                    .map(|m| m.custom_tags.clone())
                    .unwrap_or_default(),
            };
            
            mods.push(entry);
        }
        
        info!("Scan complete: {} total files, {} pak files, {} loaded successfully", 
              files_found, pak_files_found, mods.len());
        
        self.pak_files = mods;
        self.update_search_filter();
        
        // Reset mod type caches
        self.mod_type_cache.borrow_mut().clear();
        self.all_mod_types_cache.clear();
        self.mod_types_dirty = true;
    }
    
    pub fn refresh_mods(&mut self) {
        self.collect_pak_files();
    }
    
    // Mod operations
    pub fn toggle_mod(&mut self, index: usize) {
        if let Some(mod_entry) = self.pak_files.get_mut(index) {
            mod_entry.enabled = !mod_entry.enabled;
            
            // Rename file to enable/disable
            let new_extension = if mod_entry.enabled {
                "pak"
            } else {
                "pak_disabled"
            };
            
            let new_path = mod_entry.path.with_extension(new_extension);
            
            match std::fs::rename(&mod_entry.path, &new_path) {
                Ok(_) => {
                    mod_entry.path = new_path.clone();
                    info!("{} mod: {:?}", 
                        if mod_entry.enabled { "Enabled" } else { "Disabled" },
                        new_path
                    );
                }
                Err(e) => {
                    error!("Failed to rename mod file: {}", e);
                    // Revert the enabled state if rename failed
                    mod_entry.enabled = !mod_entry.enabled;
                }
            }
        }
    }
    
    pub fn delete_mod(&mut self, index: usize) {
        if let Some(mod_entry) = self.pak_files.get(index) {
            let path = mod_entry.path.clone();
            match std::fs::remove_file(&path) {
                Ok(_) => {
                    info!("Deleted mod: {:?}", path);
                    self.pak_files.remove(index);
                    self.update_search_filter();
                }
                Err(e) => {
                    error!("Failed to delete mod file: {}", e);
                }
            }
        }
    }
    
    // Folder operations
    pub fn create_folder(&mut self, name: String) {
        use uuid::Uuid;
        let folder = ModFolder {
            id: Uuid::new_v4().to_string(),
            name,
            expanded: true,
        };
        self.folders.push(folder);
        let _ = self.save_config();
        info!("Created folder");
    }
    
    pub fn delete_folder(&mut self, folder_id: &str) {
        // Remove folder
        self.folders.retain(|f| f.id != folder_id);
        
        // Remove folder assignment from mods
        for mod_entry in &mut self.pak_files {
            if mod_entry.folder_id.as_ref() == Some(&folder_id.to_string()) {
                mod_entry.folder_id = None;
            }
        }
        
        let _ = self.save_config();
        info!("Deleted folder: {}", folder_id);
    }
    
    pub fn toggle_folder(&mut self, folder_id: &str) {
        // Check if folder exists
        if !self.folders.iter().any(|f| f.id == folder_id) {
            return;
        }
        
        // Get target state - if any mod in folder is disabled, enable all
        let target_enabled = self.pak_files.iter()
            .filter(|m| m.folder_id.as_ref() == Some(&folder_id.to_string()))
            .any(|m| !m.enabled);
        
        // Collect paths that need toggling
        let paths_to_toggle: Vec<_> = self.pak_files.iter()
            .filter(|m| m.folder_id.as_ref() == Some(&folder_id.to_string()))
            .filter(|m| m.enabled != target_enabled)
            .map(|m| m.path.clone())
            .collect();
        
        // Toggle collected paths
        for path in paths_to_toggle {
            self.toggle_mod_by_path(&path);
        }
        
        let _ = self.save_config();
    }
    
    fn toggle_mod_by_path(&mut self, path: &std::path::Path) {
        if let Some(index) = self.pak_files.iter().position(|m| m.path == path) {
            self.toggle_mod(index);
        }
    }
    
    // Selection and details
    pub fn get_selected_pak(&self) -> Option<&ModEntry> {
        self.current_pak_file_idx.and_then(|idx| self.pak_files.get(idx))
    }
    
    pub fn get_pak_files_list(&self) -> Vec<String> {
        if let Some(pak_entry) = self.get_selected_pak() {
            // Check if a .utoc file exists (IoStore format)
            let mut utoc_path = pak_entry.path.clone();
            utoc_path.set_extension("utoc");
            
            if utoc_path.exists() {
                // Use utoc to get the actual file list
                let entries = crate::utoc_utils::read_utoc(&utoc_path, &pak_entry.reader, &pak_entry.path);
                entries.iter().map(|e| e.file_path.clone()).collect()
            } else {
                // Fall back to reading from PAK directly
                pak_entry.reader.files().to_vec()
            }
        } else {
            Vec::new()
        }
    }
    
    pub fn select_mod(&mut self, index: usize) {
        self.current_pak_file_idx = Some(index);
        self.selected_file_in_table = None;
    }
    
    pub fn clear_selection(&mut self) {
        self.current_pak_file_idx = None;
        self.selected_file_in_table = None;
    }
    
    // Update checking
    pub fn spawn_update_check(&mut self, manual: bool) {
        use std::sync::mpsc::{channel, Sender};
        
        if self.update_in_flight {
            return;
        }
        
        self.update_in_flight = true;
        self.last_update_error = None;
        self.update_manual_last = manual;
        
        let (tx, rx): (Sender<Result<UpdateInfo, String>>, _) = channel();
        self.update_rx = Some(rx);
        
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        
        thread::spawn(move || {
            use std::time::Duration;
            
            let url = "https://api.github.com/repos/XzantGaming/Repak-Gui-Revamped/releases/latest";
            let client = match reqwest::blocking::Client::builder()
                .user_agent(format!("Repak-Gui-Revamped/{}", current_version))
                .timeout(Duration::from_secs(10))
                .build() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Err(format!("HTTP client error: {}", e)));
                    return;
                }
            };
            
            let resp = match client.get(url).send() {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(format!("Request failed: {}", e)));
                    return;
                }
            };
            
            if !resp.status().is_success() {
                let _ = tx.send(Err(format!("GitHub returned status {}", resp.status())));
                return;
            }
            
            let json: serde_json::Value = match resp.json() {
                Ok(j) => j,
                Err(e) => {
                    let _ = tx.send(Err(format!("Invalid JSON: {}", e)));
                    return;
                }
            };
            
            let tag = json.get("tag_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let html_url = json.get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://github.com/XzantGaming/Repak-Gui-Revamped/releases");
            
            // Try to find downloadable asset
            let mut asset_url: Option<String> = None;
            let mut asset_name: Option<String> = None;
            
            if let Some(assets) = json.get("assets").and_then(|v| v.as_array()) {
                // Look for .msi installer first
                for asset in assets {
                    let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let url = asset.get("browser_download_url").and_then(|v| v.as_str()).unwrap_or("");
                    
                    if name.to_ascii_lowercase().ends_with(".msi") {
                        asset_url = Some(url.to_string());
                        asset_name = Some(name.to_string());
                        break;
                    }
                }
                
                // Fall back to .zip if no .msi
                if asset_url.is_none() {
                    for asset in assets {
                        let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let url = asset.get("browser_download_url").and_then(|v| v.as_str()).unwrap_or("");
                        
                        if name.to_ascii_lowercase().ends_with(".zip") {
                            asset_url = Some(url.to_string());
                            asset_name = Some(name.to_string());
                            break;
                        }
                    }
                }
            }
            
            let _ = tx.send(Ok(UpdateInfo {
                latest: tag.to_string(),
                url: html_url.to_string(),
                asset_url,
                asset_name,
            }));
        });
        
        if manual {
            info!("Checking for updates...");
        }
    }
    
    pub fn check_update_result(&mut self) {
        if let Some(rx) = &self.update_rx {
            if let Ok(result) = rx.try_recv() {
                self.update_in_flight = false;
                
                match result {
                    Ok(info) => {
                        self.update_info = Some(info.clone());
                        
                        // Check if update is available
                        let current = env!("CARGO_PKG_VERSION");
                        if Self::compare_versions(&info.latest, current) > 0 {
                            info!("Update available: {} -> {}", current, info.latest);
                        } else if self.update_manual_last {
                            info!("You are up to date! ({})", current);
                        }
                    }
                    Err(e) => {
                        self.last_update_error = Some(e.clone());
                        if self.update_manual_last {
                            error!("Update check failed: {}", e);
                        }
                    }
                }
                
                self.update_rx = None;
            }
        }
    }
    
    fn compare_versions(v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<u32> = v1.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let v2_parts: Vec<u32> = v2.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        for i in 0..v1_parts.len().max(v2_parts.len()) {
            let p1 = v1_parts.get(i).copied().unwrap_or(0);
            let p2 = v2_parts.get(i).copied().unwrap_or(0);
            
            if p1 > p2 { return 1; }
            if p1 < p2 { return -1; }
        }
        
        0
    }
}
