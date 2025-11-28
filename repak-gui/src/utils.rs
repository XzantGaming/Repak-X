use std::collections::HashSet;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, io};

use log::info;
use regex_lite::Regex;

// Use the runtime character_data module instead of compile-time embedded data
use crate::character_data;

static SKIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[0-9]{4}\/[0-9]{7}").unwrap()
});

// Regex to extract just the character ID (4 digits) from paths like /Characters/1021/
static CHAR_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Characters/(\d{4})").unwrap()
});

// Regex to extract character ID from filenames (e.g., bnk_vo_1044001.bnk -> 1044)
// Matches 7-digit patterns and extracts first 4 digits as character ID
static FILENAME_CHAR_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"_(\d{4})\d{3}").unwrap()
});

/// Result of mod characteristics detection, includes mod type and detected heroes
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModCharacteristics {
    pub mod_type: String,
    pub heroes: Vec<String>,
    /// Character name for display (e.g., "Blade" or "Hawkeye - Default")
    /// Empty if no specific character or multiple characters
    pub character_name: String,
    /// Pure mod category (e.g., "Audio", "Mesh", "VFX")
    /// Without character name prefix
    pub category: String,
}

impl ModCharacteristics {
    /// Format the mod type with hero info for display
    pub fn display_type(&self) -> String {
        if self.heroes.is_empty() {
            self.mod_type.clone()
        } else if self.heroes.len() == 1 {
            format!("{} ({})", self.heroes[0], self.mod_type)
        } else {
            format!("Multiple Heroes ({}) ({})", self.heroes.len(), self.mod_type)
        }
    }
}

pub fn collect_files(paths: &mut Vec<PathBuf>, dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(paths, &path)?;
        } else {
            paths.push(entry.path());
        }
    }
    Ok(())
}

pub enum ModType {
    Default(String),
    Custom(String),
}

/// Get character/skin info from file path using runtime character data cache
/// This uses the updated data from roaming folder, not the compile-time embedded data
pub fn get_character_mod_skin(file: &str) -> Option<ModType> {
    let skin_id_match = SKIN_REGEX.captures(file);
    if let Some(caps) = skin_id_match {
        let full_match = caps[0].to_string();
        // Extract just the 7-digit skin ID (skip the "1234/" prefix)
        let skin_id = &full_match[5..];
        
        // Use the runtime character data lookup
        if let Some(skin) = character_data::get_character_by_skin_id(skin_id) {
            if skin.skin_name == "Default" {
                return Some(ModType::Default(format!(
                    "{} - {}",
                    &skin.name, &skin.skin_name
                )));
            }
            return Some(ModType::Custom(format!(
                "{} - {}",
                &skin.name, &skin.skin_name
            )));
        }
        None
    } else {
        None
    }
}
/// Get detailed mod characteristics including mod type and all detected heroes
pub fn get_pak_characteristics_detailed(mod_contents: Vec<String>) -> ModCharacteristics {
    let mut fallback: Option<String> = None;
    
    // Track what content types we find
    let mut has_skeletal_mesh = false;
    let mut has_static_mesh = false;
    let mut has_texture = false;
    let mut has_material = false;
    let mut has_audio = false;
    let mut has_movies = false;
    let mut has_ui = false;
    let mut character_name: Option<String> = None;  // Full skin-specific name (e.g., "Hawkeye - Default")
    let mut hero_names: HashSet<String> = HashSet::new();  // All detected hero names

    for file in &mod_contents {
        let path = file
            .strip_prefix("Marvel/Content/Marvel/")
            .or_else(|| file.strip_prefix("/Game/Marvel/"))
            .unwrap_or(file);
        
        let filename = path.split('/').last().unwrap_or("");
        let filename_lower = filename.to_lowercase();
        let path_lower = path.to_lowercase();

        // Check for specific asset types by filename pattern
        // Note: Internal paths may or may not have .uasset extension
        let is_uasset = filename_lower.ends_with(".uasset") || !filename_lower.contains('.');
        
        if filename_lower.starts_with("sk_") && is_uasset {
            has_skeletal_mesh = true;
        }
        if filename_lower.starts_with("sm_") && is_uasset {
            has_static_mesh = true;
        }
        if filename_lower.starts_with("t_") && is_uasset {
            has_texture = true;
        }
        
        // VFX: MI_ files in VFX path (e.g. /Game/Marvel/VFX/Materials/...)
        // Check both original file path and stripped path
        let file_lower = file.to_lowercase();
        if filename_lower.starts_with("mi_") && (path_lower.contains("/vfx/") || path_lower.starts_with("vfx/") || file_lower.contains("/vfx/")) {
            has_material = true;
        }
        
        // Check path-based categories
        if path_lower.contains("wwiseaudio") || file_lower.contains("wwiseaudio") {
            has_audio = true;
        }
        
        // UI: Files in UI folder
        if path_lower.contains("/ui/") || path_lower.starts_with("ui/") || file_lower.contains("/ui/") {
            has_ui = true;
        }
        
        // Movies: Files in Movies folder (placeholder - user to research exact criteria)
        if path_lower.contains("/movies/") || path_lower.starts_with("movies/") || file_lower.contains("/movies/") || path_lower.ends_with(".bik") || path_lower.ends_with(".mp4") {
            has_movies = true;
        }

        // Try to get skin-specific name from Characters paths
        let category = path.split('/').next().unwrap_or_default();
        if category == "Characters" {
            match get_character_mod_skin(path) {
                Some(ModType::Custom(skin)) => character_name = Some(skin),
                Some(ModType::Default(name)) => fallback = Some(name),
                None => {}
            }
        }
        
        // Extract ALL hero names from character IDs anywhere in path
        // This handles paths like /Game/Marvel/VFX/Meshes/Characters/1048/...
        if let Some(caps) = CHAR_ID_REGEX.captures(file) {
            if let Some(char_id) = caps.get(1) {
                if let Some(name) = character_data::get_character_name_from_id(char_id.as_str()) {
                    hero_names.insert(name);
                }
            }
        }
        
        // Also try to extract character ID from filenames (for audio/UI mods)
        // Handles patterns like bnk_vo_1044001.bnk or UI_1048_icon.uasset
        if let Some(caps) = FILENAME_CHAR_ID_REGEX.captures(filename) {
            if let Some(char_id) = caps.get(1) {
                if let Some(name) = character_data::get_character_name_from_id(char_id.as_str()) {
                    hero_names.insert(name);
                }
            }
        }
    }
    
    // Convert to sorted Vec for consistent ordering
    let mut heroes: Vec<String> = hero_names.into_iter().collect();
    heroes.sort();

    // Determine the pure category (without character name)
    let category = if has_audio && !has_skeletal_mesh && !has_static_mesh && !has_texture && !has_material {
        "Audio"
    } else if has_movies && !has_skeletal_mesh && !has_static_mesh && !has_texture && !has_material {
        "Movies"
    } else if has_ui && !has_skeletal_mesh && !has_static_mesh && !has_texture && !has_material {
        "UI"
    } else if has_skeletal_mesh {
        "Mesh"
    } else if has_static_mesh {
        "Static Mesh"
    } else if has_material {
        "VFX"
    } else if has_audio {
        "Audio"
    } else if has_texture {
        "Retexture"
    } else {
        "Unknown"
    };
    
    // Determine character_name for display
    // Priority: skin-specific name > single hero > empty
    let display_character_name = if let Some(ref char_name) = character_name {
        char_name.clone()
    } else if heroes.len() == 1 {
        heroes[0].clone()
    } else {
        String::new()
    };
    
    // Build the combined mod_type string with " - " separator for easy splitting
    let mod_type = if !display_character_name.is_empty() {
        // Character detected - combine with " - " separator
        format!("{} - {}", display_character_name, category)
    } else if heroes.len() > 1 {
        // Multiple heroes
        format!("Multiple Heroes ({}) - {}", heroes.len(), category)
    } else {
        // No heroes detected - just category
        category.to_string()
    };
    
    ModCharacteristics {
        mod_type,
        heroes,
        character_name: display_character_name,
        category: category.to_string(),
    }
}

/// Get mod characteristics as a display string (backward compatible)
/// Returns the mod_type string which uses " - " separator between character and category
/// Format: "Character - Category" or just "Category" if no character
pub fn get_current_pak_characteristics(mod_contents: Vec<String>) -> String {
    let chars = get_pak_characteristics_detailed(mod_contents);
    chars.mod_type
}

pub fn find_marvel_rivals() -> Option<PathBuf> {
    let shit = get_steam_library_paths();
    if shit.is_empty() {
        return None;
    }

    for lib in shit {
        let path = lib.join("steamapps/common/MarvelRivals/MarvelGame/Marvel/Content/Paks");
        if path.exists() {
            return Some(path);
        }
    }
    println!("Marvel Rivals not found.");
    None
}

/// Reads `libraryfolders.vdf` to find additional Steam libraries.
fn get_steam_library_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    let vdf_path = PathBuf::from("C:/Program Files (x86)/Steam/steamapps/libraryfolders.vdf");

    #[cfg(target_os = "linux")]
    let vdf_path = PathBuf::from("~/.steam/steam/steamapps/libraryfolders.vdf");

    if !vdf_path.exists() {
        return vec![];
    }

    let content = fs::read_to_string(vdf_path).ok().unwrap_or_default();
    let mut paths = Vec::new();

    for line in content.lines() {
        // if line.contains('"') {
        //     let path: String = line
        //         .split('"')
        //         .nth(3)  // Extracts the path
        //         .map(|s| s.replace("\\\\", "/"))?; // Fix Windows paths
        //     paths.push(PathBuf::from(path).join("steamapps/common"));
        // }
        if line.trim().starts_with("\"path\"") {
            let path = line
                .split("\"")
                .nth(3)
                .map(|s| PathBuf::from(s.replace("\\\\", "\\")));
            info!("Found steam library path: {:?}", path);
            paths.push(path.unwrap());
        }
    }

    paths
}
