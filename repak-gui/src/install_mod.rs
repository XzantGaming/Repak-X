pub mod install_mod_logic;

use crate::install_mod::install_mod_logic::archives::{extract_zip, extract_rar, extract_7z};
use crate::uasset_detection::{detect_mesh_files, detect_texture_files, detect_static_mesh_files};
use crate::utils::{collect_files, get_current_pak_characteristics};
use crate::utoc_utils::read_utoc;
use log::{debug, error};
use repak::utils::AesKey;
use repak::Compression::Oodle;
use repak::{Compression, PakReader};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::LazyLock;
use tempfile::tempdir;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstallableMod {
    pub mod_name: String,
    pub mod_type: String,
    pub custom_tags: Vec<String>,
    pub custom_tag_input: String,
    pub repak: bool,
    pub fix_mesh: bool,
    pub fix_textures: bool,
    pub fix_serialsize_header: bool,
    pub usmap_path: String,
    pub is_dir: bool,
    pub editing: bool,
    pub path_hash_seed: String,
    pub mount_point: String,
    #[serde(skip)]
    pub compression: Compression,
    #[serde(skip)]
    pub reader: Option<PakReader>,
    pub mod_path: PathBuf,
    pub total_files: usize,
    pub iostore: bool,
    // the only reason we keep this is to filter out the archives during collection
    pub is_archived: bool,
    pub enabled: bool,
    // pub audio_mod: bool,
}

impl Default for InstallableMod {
    fn default() -> Self {
        InstallableMod{
            mod_name: "".to_string(),
            mod_type: "".to_string(),
            custom_tags: Vec::new(),
            custom_tag_input: String::new(),
            repak: false,
            fix_mesh: false,
            fix_textures: false,
            fix_serialsize_header: false,
            usmap_path: String::new(),
            is_dir: false,
            editing: false,
            path_hash_seed: "".to_string(),
            mount_point: "".to_string(),
            compression: Default::default(),
            reader: None,
            mod_path: Default::default(),
            total_files: 0,
            iostore: false,
            is_archived: false,
            enabled: true,
        }
    }
}

pub static AES_KEY: LazyLock<AesKey> = LazyLock::new(|| {
    AesKey::from_str("0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74")
        .expect("Unable to initialise AES_KEY")
});

fn find_mods_from_archive(path: &str) -> Vec<InstallableMod> {
    let mut new_mods = Vec::<InstallableMod>::new();
    let mut processed_mods = std::collections::HashSet::new();
    let mut found_pak_files = false;
    
    // First pass: look for .pak files (existing behavior)
    for entry in WalkDir::new(path) {
        let entry = entry.expect("Failed to read directory entry");
        let file_path = entry.path();
        
        // Only process .pak files
        if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("pak") {
            found_pak_files = true;
            let mod_base_name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
            
            // Skip if we've already processed this mod
            if processed_mods.contains(&mod_base_name) {
                continue;
            }
            processed_mods.insert(mod_base_name.clone());
            
            let utoc_path = file_path.with_extension("utoc");
            let ucas_path = file_path.with_extension("ucas");

            // Check if this is an iostore mod (has all three files: pak, utoc, ucas)
            if utoc_path.exists() && ucas_path.exists() {
                // This is an iostore mod - just copy the files, don't go through repak workflow
                let builder = repak::PakBuilder::new()
                    .key(AES_KEY.clone().0)
                    .reader(&mut BufReader::new(File::open(&file_path).unwrap()));

                if let Ok(builder) = builder {
                    let files = read_utoc(&utoc_path, &builder, &file_path);
                    let files = files
                        .iter()
                        .map(|x| x.file_path.clone())
                        .collect::<Vec<_>>();
                    let len = files.len();
                    let modtype = get_current_pak_characteristics(files);

                    let installable_mod = InstallableMod {
                        mod_name: mod_base_name,
                        mod_type: modtype.to_string(),
                        repak: false,  // Don't use repak workflow for iostore mods
                        fix_mesh: false,
                        is_dir: false,
                        reader: Some(builder),
                        mod_path: file_path.to_path_buf(),
                        mount_point: "../../../".to_string(),
                        path_hash_seed: "00000000".to_string(),
                        total_files: len,
                        iostore: true,  // Mark as iostore so it gets copied directly
                        is_archived: false,
                        editing: false,
                        compression: Oodle,
                        ..Default::default()
                    };

                    new_mods.push(installable_mod);
                }
            }
            // This is a standalone .pak file
            else {
                let builder = repak::PakBuilder::new()
                    .key(AES_KEY.clone().0)
                    .reader(&mut BufReader::new(File::open(file_path).unwrap()));

                if let Ok(builder) = builder {
                    let files = builder.files();
                    let len = files.len();
                    let modtype = get_current_pak_characteristics(files.clone());
                    
                    // Check if this is an Audio or Movies mod (these should skip repak workflow)
                    let is_audio_or_movie = modtype.contains("Audio") || modtype.contains("Movies");
                    
                    // Auto-detect mesh and texture files
                    let auto_fix_mesh = detect_mesh_files(&files);
                    let auto_fix_textures = detect_texture_files(&files);
                    let auto_fix_static_mesh = detect_static_mesh_files(&files);

                    let installable_mod = InstallableMod {
                        mod_name: mod_base_name,
                        mod_type: modtype.to_string(),
                        repak: !is_audio_or_movie,  // Only use repak if NOT Audio/Movies
                        fix_mesh: auto_fix_mesh,
                        fix_textures: auto_fix_textures,
                        fix_serialsize_header: auto_fix_static_mesh,
                        is_dir: false,
                        reader: Some(builder),
                        mod_path: file_path.to_path_buf(),
                        mount_point: "../../../".to_string(),
                        path_hash_seed: "00000000".to_string(),
                        total_files: len,
                        iostore: false,
                        is_archived: false,
                        editing: false,
                        compression: Oodle,
                        ..Default::default()
                    };

                    new_mods.push(installable_mod);
                }
            }
        }
    }

    // Second pass: if no .pak files found, look for content folders with .uasset files
    // This handles archives that contain loose mod files (folders) instead of pre-packed .pak files
    if !found_pak_files {
        debug!("No .pak files found in archive, looking for content folders...");
        
        // Find directories that contain .uasset files (these are mod content folders)
        let archive_root = std::path::Path::new(path);
        
        // Check immediate subdirectories of the archive root
        if let Ok(entries) = std::fs::read_dir(archive_root) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                
                // Check if this is a directory that contains content
                if entry_path.is_dir() {
                    let mut has_content = false;
                    let mut content_files = Vec::new();
                    
                    // Recursively collect files and check for .uasset content
                    if collect_files(&mut content_files, &entry_path).is_ok() {
                        for file in &content_files {
                            if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
                                if ext == "uasset" || ext == "uexp" || ext == "ubulk" || ext == "bnk" || ext == "wem" {
                                    has_content = true;
                                    break;
                                }
                            }
                        }
                    }
                    
                    if has_content {
                        let mod_name = entry_path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        debug!("Found content folder in archive: {} ({} files)", mod_name, content_files.len());
                        
                        // Get file paths as strings for mod type detection
                        let file_strings: Vec<String> = content_files
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        
                        let modtype = get_current_pak_characteristics(file_strings.clone());
                        let is_audio_or_movies = modtype.contains("Audio") || modtype.contains("Movies");
                        
                        // Auto-detect mesh and texture files
                        let auto_fix_mesh = detect_mesh_files(&file_strings);
                        let auto_fix_textures = detect_texture_files(&file_strings);
                        let auto_fix_static_mesh = detect_static_mesh_files(&file_strings);
                        
                        let installable_mod = InstallableMod {
                            mod_name,
                            mod_type: modtype.to_string(),
                            repak: !is_audio_or_movies,  // Will go through convert_to_iostore_directory
                            fix_mesh: auto_fix_mesh,
                            fix_textures: auto_fix_textures,
                            fix_serialsize_header: auto_fix_static_mesh,
                            is_dir: true,  // Mark as directory so it uses convert_to_iostore_directory
                            reader: None,
                            mod_path: entry_path,
                            mount_point: "../../../".to_string(),
                            path_hash_seed: "00000000".to_string(),
                            total_files: content_files.len(),
                            iostore: false,
                            is_archived: false,
                            editing: false,
                            compression: Oodle,
                            ..Default::default()
                        };
                        
                        new_mods.push(installable_mod);
                    }
                }
            }
        }
        
        // If still no mods found, check if the archive root itself contains content files directly
        if new_mods.is_empty() {
            let mut content_files = Vec::new();
            if collect_files(&mut content_files, archive_root).is_ok() {
                let has_content = content_files.iter().any(|f| {
                    f.extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| ext == "uasset" || ext == "uexp" || ext == "ubulk" || ext == "bnk" || ext == "wem")
                        .unwrap_or(false)
                });
                
                if has_content {
                    // Use the archive folder name as mod name
                    let mod_name = archive_root.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("ExtractedMod")
                        .to_string();
                    
                    debug!("Archive root contains content files: {} ({} files)", mod_name, content_files.len());
                    
                    let file_strings: Vec<String> = content_files
                        .iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect();
                    
                    let modtype = get_current_pak_characteristics(file_strings.clone());
                    let is_audio_or_movies = modtype.contains("Audio") || modtype.contains("Movies");
                    let auto_fix_mesh = detect_mesh_files(&file_strings);
                    let auto_fix_textures = detect_texture_files(&file_strings);
                    let auto_fix_static_mesh = detect_static_mesh_files(&file_strings);
                    
                    let installable_mod = InstallableMod {
                        mod_name,
                        mod_type: modtype.to_string(),
                        repak: !is_audio_or_movies,
                        fix_mesh: auto_fix_mesh,
                        fix_textures: auto_fix_textures,
                        fix_serialsize_header: auto_fix_static_mesh,
                        is_dir: true,
                        reader: None,
                        mod_path: archive_root.to_path_buf(),
                        mount_point: "../../../".to_string(),
                        path_hash_seed: "00000000".to_string(),
                        total_files: content_files.len(),
                        iostore: false,
                        is_archived: false,
                        editing: false,
                        compression: Oodle,
                        ..Default::default()
                    };
                    
                    new_mods.push(installable_mod);
                }
            }
        }
    }

    new_mods
}

fn map_to_mods_internal(paths: &[PathBuf]) -> Vec<InstallableMod> {
    let mut extensible_vec: Vec<InstallableMod> = Vec::new();
    let mut installable_mods = paths
        .iter()
        .map(|path| {
            let is_dir = path.clone().is_dir();
            let extension = path.extension().unwrap_or_default();
            let is_archive = extension == "zip" || extension == "rar" || extension == "7z";
            
            // Check if this is an IoStore package (has .utoc and .ucas companions)
            let is_iostore = if extension == "pak" {
                let utoc_path = path.with_extension("utoc");
                let ucas_path = path.with_extension("ucas");
                utoc_path.exists() && ucas_path.exists()
            } else {
                false
            };

            let mut modtype = "Unknown".to_string();
            let mut pak = None;
            let mut len = 1;
            let mut auto_fix_mesh = false;
            let mut auto_fix_textures = false;
            let mut auto_fix_static_mesh = false;

            if !is_dir && !is_archive {
                let builder = repak::PakBuilder::new()
                    .key(AES_KEY.clone().0)
                    .reader(&mut BufReader::new(File::open(path.clone()).unwrap()));
                match builder {
                    Ok(builder) => {
                        pak = Some(builder.clone());
                        
                        // For IoStore packages, read from .utoc file
                        let files = if is_iostore {
                            let utoc_path = path.with_extension("utoc");
                            let utoc_files = read_utoc(&utoc_path, &builder, path);
                            len = utoc_files.len();
                            utoc_files.iter().map(|f| f.file_path.clone()).collect()
                        } else {
                            let files = builder.files();
                            len = files.len();
                            files
                        };
                        
                        modtype = get_current_pak_characteristics(files.clone());
                        
                        // IoStore packages should not have auto-fixes enabled
                        if !is_iostore {
                            // Auto-detect mesh and texture files in pak files
                            auto_fix_mesh = detect_mesh_files(&files);
                            auto_fix_textures = detect_texture_files(&files);
                            auto_fix_static_mesh = detect_static_mesh_files(&files);
                        }
                    }
                    Err(e) => {
                        error!("Error reading pak file: {}", e);
                        return Err(e);
                    }
                }
            }

            if is_dir {
                let mut files = vec![];
                collect_files(&mut files, path)?;
                let files = files
                    .iter()
                    .map(|s| s.to_str().unwrap().to_string())
                    .collect::<Vec<_>>();
                len = files.len();
                modtype = get_current_pak_characteristics(files.clone());
                
                // Auto-detect mesh and texture files
                auto_fix_mesh = detect_mesh_files(&files);
                auto_fix_textures = detect_texture_files(&files);
                auto_fix_static_mesh = detect_static_mesh_files(&files);
            }

            if is_archive {
                modtype = "Archive".to_string();
                let tempdir = tempdir()
                    .unwrap()
                    .path()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();

                if extension == "zip" {
                    extract_zip(path.to_str().unwrap(), &tempdir).expect("Unable to extract zip archive")
                } else if extension == "rar" {
                    extract_rar(path.to_str().unwrap(), &tempdir).expect("Unable to extract rar archive")
                } else if extension == "7z" {
                    extract_7z(path.to_str().unwrap(), &tempdir).expect("Unable to extract 7z archive")
                }

                // Now find pak files / iostore mods and turn them into installable mods
                let mut new_mods = find_mods_from_archive(&tempdir);
                extensible_vec.append(&mut new_mods);
            }

            // Determine if we should repak this mod
            // Don't repak if: it's a directory, IoStore package, or Audio/Movies mod
            let is_audio_or_movies = modtype.contains("Audio") || modtype.contains("Movies");
            let should_repak = !is_dir && !is_iostore && !is_audio_or_movies;
            
            Ok(InstallableMod {
                mod_name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                mod_type: modtype,
                repak: should_repak,
                fix_mesh: auto_fix_mesh,
                fix_textures: auto_fix_textures,
                fix_serialsize_header: auto_fix_static_mesh,
                is_dir,
                reader: pak,
                mod_path: path.clone(),
                mount_point: "../../../".to_string(),
                path_hash_seed: "00000000".to_string(),
                total_files: len,
                iostore: is_iostore,  // Mark as IoStore package
                is_archived: is_archive,
                ..Default::default()
            })
        })
        .filter_map(|x: Result<InstallableMod, repak::Error>| x.ok())
        .filter(|x| !x.is_archived)
        .collect::<Vec<_>>();

    installable_mods.extend(extensible_vec);

    debug!("Install mods: {:?}", installable_mods);
    installable_mods
}

pub fn map_paths_to_mods(paths: &[PathBuf]) -> Vec<InstallableMod> {
    let installable_mods = map_to_mods_internal(paths);
    installable_mods
}

// Egui-specific function - stubbed out for Tauri
#[allow(dead_code)]
pub fn map_dropped_file_to_mods(_dropped_files: &[PathBuf]) -> Vec<InstallableMod> {
    // This function signature is changed for Tauri compatibility
    // Original egui version uses egui::DroppedFile
    unimplemented!("This function is only available in the egui version")
}
