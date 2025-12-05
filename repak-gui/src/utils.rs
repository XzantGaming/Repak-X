use std::collections::HashSet;
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

// Regex to extract just the character ID (4 digits) from paths like /Characters/1021/ or /Hero_ST/1048/
static CHAR_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:Characters|Hero_ST|Hero)/(\d{4})").unwrap()
});

// Regex to extract character ID from filenames (e.g., bnk_vo_1044001.bnk -> 1044)
// More strict pattern: requires the 7-digit skin ID to start with valid character ID range (10xx)
// This avoids false positives from random 7-digit numbers in filenames
static FILENAME_CHAR_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches patterns like _1044001, vo_1044001, where 1044 is a character ID in 10xx range
    Regex::new(r"[_/](10[1-6]\d)(\d{3})").unwrap()
});

// Alternative strict pattern for skin IDs in paths (7 consecutive digits starting with 10xx)
static SKIN_ID_STRICT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(10[1-6]\d\d{3})").unwrap()
});

// Broad regex to find 4-digit sequences that are likely character IDs
// Must be preceded by / or _ and not part of a longer number (not preceded/followed by digits)
// This catches hero IDs anywhere in the path/filename while avoiding false positives
static BROAD_FOUR_DIGIT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[/_])(\d{4})(?:[/_]|$)").unwrap()
});

// Flexible regex to find 4-digit hero IDs in filenames (10[1-6]X format)
// This can match hero IDs even when part of longer numbers (for UI/audio filenames)
// Example: img_battle_21044020_avatar -> matches 1044
static FILENAME_HERO_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(10[1-6]\d)").unwrap()
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
    /// Additional categories that can appear alongside the main category
    /// e.g., Blueprint, Text - these are additive and don't override the main category
    pub additional_categories: Vec<String>,
}

impl ModCharacteristics {
    /// Format the mod type with hero info for display
    #[allow(dead_code)]
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
        info!("SKIN_REGEX matched: {}", full_match);
        // Extract just the 7-digit skin ID (skip the "1234/" prefix)
        let skin_id = &full_match[5..];
        info!("Extracted skin_id: {}", skin_id);
        
        // Use the runtime character data lookup
        if let Some(skin) = character_data::get_character_by_skin_id(skin_id) {
            info!("Found skin in database: {} - {}", skin.name, skin.skin_name);
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
        } else {
            info!("Skin ID {} not found in character database", skin_id);
        }
        None
    } else {
        None
    }
}
/// Get detailed mod characteristics including mod type and all detected heroes
pub fn get_pak_characteristics_detailed(mod_contents: Vec<String>) -> ModCharacteristics {
    let mut _fallback: Option<String> = None;
    
    // Track what content types we find
    let mut has_skeletal_mesh = false;
    let mut has_static_mesh = false;
    let mut has_texture = false;
    let mut has_material = false;
    let mut has_audio = false;
    let mut has_movies = false;
    let mut has_ui = false;
    let mut has_blueprint = false;
    let mut has_text = false;
    let mut character_name: Option<String> = None;  // Full skin-specific name (e.g., "Hawkeye - Default")
    let mut hero_names: HashSet<String> = HashSet::new();  // All detected hero names

    // FIRST PASS: Extract hero IDs from directory paths only (not filenames)
    // This prevents false positives from filenames like "texture_1048.uasset"
    for file in &mod_contents {
        let path = file
            .strip_prefix("Marvel/Content/Marvel/")
            .or_else(|| file.strip_prefix("/Game/Marvel/"))
            .unwrap_or(file);
        
        // Get directory path without filename
        let dir_path = if let Some(last_slash) = file.rfind('/') {
            &file[..last_slash]
        } else {
            file
        };
        
        // Extract hero names from character IDs in directory paths
        // This handles paths like /Game/Marvel/VFX/Meshes/Characters/1048/...
        if let Some(caps) = CHAR_ID_REGEX.captures(dir_path) {
            if let Some(char_id) = caps.get(1) {
                if let Some(name) = character_data::get_character_name_from_id(char_id.as_str()) {
                    hero_names.insert(name);
                }
            }
        }
        
        // Check for 7-digit skin IDs in directory paths (handles paths like /1044001/)
        // Validate the full skin ID against the character database to ensure both
        // the hero ID (1044) and skin portion (001) are valid
        if let Some(caps) = SKIN_ID_STRICT_REGEX.captures(dir_path) {
            if let Some(skin_id_match) = caps.get(1) {
                let skin_id = skin_id_match.as_str();
                // Validate the full 7-digit skin ID exists in the character database
                if let Some(skin) = character_data::get_character_by_skin_id(skin_id) {
                    hero_names.insert(skin.name);
                }
            }
        }
        
        // Broad fallback for directories: Check for 4-digit sequences in directory paths
        // This catches hero IDs in any position (e.g., /StringTable/Hero_ST/1048/, /Data/1048_CustomData/, etc.)
        // We validate each match against the character database to avoid false positives
        for caps in BROAD_FOUR_DIGIT_REGEX.captures_iter(dir_path) {
            if let Some(four_digits) = caps.get(1) {  // Get capture group 1 (the digits)
                let potential_id = four_digits.as_str();
                // Validate it's actually a character ID by checking the database
                if let Some(name) = character_data::get_character_name_from_id(potential_id) {
                    hero_names.insert(name);
                }
            }
        }
        
        // Try to get skin-specific name from Characters paths
        let category = path.split('/').next().unwrap_or_default();
        if category == "Characters" {
            // Use the original file path for skin detection (not stripped path)
            // because SKIN_REGEX needs the full path structure
            info!("Checking for skin in file: {}", file);
            match get_character_mod_skin(file) {
                Some(ModType::Custom(skin)) => {
                    info!("Found custom skin: {}", skin);
                    character_name = Some(skin);
                },
                Some(ModType::Default(name)) => {
                    info!("Found default skin: {}", name);
                    _fallback = Some(name);
                },
                None => {
                    info!("No skin match found for: {}", file);
                }
            }
        }
    }

    // SECOND PASS: Only check filenames if no heroes found in directories
    // This ensures directory-based detection takes priority
    let check_filenames = hero_names.is_empty();

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
        
        // Text: StringTable files (localization/text mods)
        if path_lower.contains("/stringtable/") || path_lower.starts_with("stringtable/") || file_lower.contains("/stringtable/") || path_lower.contains("/data/stringtable/") {
            has_text = true;
        }
        
        // Blueprint: Common Blueprint patterns
        // 1. BP_Something (Blueprint prefix)
        // 2. Something_C (Blueprint class suffix)
        // 3. SomethingBP (Blueprint suffix)
        // 4. /Blueprints/ folder path
        if (filename_lower.starts_with("bp_") || 
            filename_lower.contains("_c.") ||
            filename_lower.contains("bp.") ||
            filename_lower.ends_with("bp") ||
            path_lower.contains("/blueprints/")) && is_uasset {
            has_blueprint = true;
        }

        // Only check filenames if no heroes were found in directory paths
        if check_filenames {
            let mut found_via_skin_id = false;
            
            // PRIORITY 1: Try to find and validate 7-digit skin IDs in filenames
            // This handles audio mods with skin-specific variants (e.g., bnk_vo_1044001.bnk)
            // and UI textures with skin IDs (e.g., img_battle_21044020_avatar)
            for caps in SKIN_ID_STRICT_REGEX.captures_iter(filename) {
                if let Some(skin_id_match) = caps.get(1) {
                    let skin_id = skin_id_match.as_str();
                    // Validate the full 7-digit skin ID exists in the character database
                    if let Some(skin) = character_data::get_character_by_skin_id(skin_id) {
                        hero_names.insert(skin.name);
                        found_via_skin_id = true;
                    }
                }
            }
            
            // FALLBACK: If no valid skin IDs found, try to find 4-digit hero IDs
            // This handles cases where only hero ID is present (e.g., img_heroportrait_1044_fullbody)
            if !found_via_skin_id {
                for caps in FILENAME_HERO_ID_REGEX.captures_iter(filename) {
                    if let Some(hero_id_match) = caps.get(1) {
                        let hero_id = hero_id_match.as_str();
                        // Validate it's a real character ID
                        if let Some(name) = character_data::get_character_name_from_id(hero_id) {
                            hero_names.insert(name);
                        }
                    }
                }
            }
        }
    }
    
    // Convert to sorted Vec for consistent ordering
    let mut heroes: Vec<String> = hero_names.into_iter().collect();
    heroes.sort();

    // Determine the pure category (without character name)
    // Priority order: Audio/Movies/UI (pure) > Mesh > Static Mesh > VFX > Audio (mixed) > Retexture
    // Note: Blueprint and Text are now additive categories and handled separately
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
    } else if has_blueprint {
        // Blueprint-only mod (no other primary category detected)
        "Blueprint"
    } else if has_text {
        // Text-only mod (no other primary category detected)
        "Text"
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
    
    // Build additional categories list (Blueprint and Text are additive)
    let mut additional_categories = Vec::new();
    if has_blueprint {
        additional_categories.push("Blueprint".to_string());
    }
    if has_text {
        additional_categories.push("Text".to_string());
    }
    
    // Build the combined mod_type string with " - " separator for easy splitting
    // Include additional categories in the display string
    let base_type = if !display_character_name.is_empty() {
        // Character detected - combine with " - " separator
        format!("{} - {}", display_character_name, category)
    } else if heroes.len() > 1 {
        // Multiple heroes
        format!("Multiple Heroes ({}) - {}", heroes.len(), category)
    } else {
        // No heroes detected - just category
        category.to_string()
    };
    
    // Append additional categories to the mod_type string
    let mod_type = if !additional_categories.is_empty() {
        format!("{} [{}]", base_type, additional_categories.join(", "))
    } else {
        base_type
    };
    
    ModCharacteristics {
        mod_type,
        heroes,
        character_name: display_character_name,
        category: category.to_string(),
        additional_categories,
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
/// Enhanced to check registry and multiple common locations.
fn get_steam_library_paths() -> Vec<PathBuf> {
    let mut vdf_paths_to_check: Vec<PathBuf> = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        // Try to get Steam path from Windows registry first
        if let Some(steam_path) = get_steam_path_from_registry() {
            let vdf = steam_path.join("steamapps/libraryfolders.vdf");
            info!("Found Steam path from registry: {:?}", vdf);
            vdf_paths_to_check.push(vdf);
        }
        
        // Common Steam installation paths to check as fallbacks
        let common_paths = [
            "C:/Program Files (x86)/Steam",
            "C:/Program Files/Steam",
            "D:/Steam",
            "D:/Program Files (x86)/Steam",
            "D:/Program Files/Steam",
            "E:/Steam",
            "E:/SteamLibrary",
            "F:/Steam",
            "F:/SteamLibrary",
        ];
        
        for path in common_paths {
            let vdf = PathBuf::from(path).join("steamapps/libraryfolders.vdf");
            if !vdf_paths_to_check.contains(&vdf) {
                vdf_paths_to_check.push(vdf);
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Expand home directory properly
        if let Some(home) = dirs::home_dir() {
            vdf_paths_to_check.push(home.join(".steam/steam/steamapps/libraryfolders.vdf"));
            vdf_paths_to_check.push(home.join(".local/share/Steam/steamapps/libraryfolders.vdf"));
        }
    }
    
    // Find first existing VDF file
    let vdf_path = vdf_paths_to_check.into_iter().find(|p| p.exists());
    
    let Some(vdf_path) = vdf_path else {
        info!("No Steam libraryfolders.vdf found");
        return vec![];
    };
    
    info!("Using Steam library config: {:?}", vdf_path);
    
    let content = fs::read_to_string(&vdf_path).ok().unwrap_or_default();
    let mut paths = Vec::new();

    for line in content.lines() {
        if line.trim().starts_with("\"path\"") {
            let path = line
                .split("\"")
                .nth(3)
                .map(|s| PathBuf::from(s.replace("\\\\", "\\")));
            info!("Found steam library path: {:?}", path);
            if let Some(p) = path {
                paths.push(p);
            }
        }
    }

    paths
}

/// Get Steam installation path from Windows registry
#[cfg(target_os = "windows")]
fn get_steam_path_from_registry() -> Option<PathBuf> {
    use std::process::Command;
    
    // Query registry for Steam install path
    // reg query "HKCU\Software\Valve\Steam" /v SteamPath
    let output = Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse output: "    SteamPath    REG_SZ    C:\Program Files (x86)\Steam"
    for line in stdout.lines() {
        if line.contains("SteamPath") && line.contains("REG_SZ") {
            // Split by REG_SZ and take the path part
            if let Some(path_part) = line.split("REG_SZ").nth(1) {
                let path = path_part.trim();
                if !path.is_empty() {
                    info!("Found Steam path in registry: {}", path);
                    return Some(PathBuf::from(path));
                }
            }
        }
    }
    
    None
}

#[cfg(not(target_os = "windows"))]
fn get_steam_path_from_registry() -> Option<PathBuf> {
    None
}
