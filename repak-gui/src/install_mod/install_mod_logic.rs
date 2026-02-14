pub mod archives;
pub mod iotoc;
pub mod pak_files;

use crate::install_mod::InstallableMod;
use iotoc::convert_to_iostore_directory;
use log::{error, info, warn};
use pak_files::create_repak_from_pak;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::collections::BTreeMap;
use std::collections::HashMap;
use dirs;
use serde_json;
use regex_lite::Regex;

/// Recursively copy a directory and all its contents to a destination
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

pub fn normalize_mod_base_name(name: &str, min_nines: usize) -> String {
    // Regex to find existing 9s suffix
    // Looking for pattern: any characters, then underscore, then 7 or more 9s, then _P
    
    // First, strip _P if present to work with the base
    let base = if name.ends_with("_P") {
        name.strip_suffix("_P").unwrap()
    } else {
        name
    };

    // Now check if base ends with _999...
    let re = Regex::new(r"^(.*)_(\d+)$").unwrap();
    
    if let Some(caps) = re.captures(base) {
        let prefix = &caps[1];
        let numbers = &caps[2];
        
        // Check if numbers are all 9s
        if numbers.chars().all(|c| c == '9') {
             let num_nines = numbers.len();
             if num_nines >= min_nines {
                 // Already has enough 9s.
                 // Re-append _P
                 return format!("{}_{}_P", prefix, numbers);
             } else {
                 // Has 9s but not enough. Replace with min_nines 9s.
                 let new_nines = "9".repeat(min_nines);
                 return format!("{}_{}_P", prefix, new_nines);
             }
        }
    }
    
    // If we are here, it doesn't have a valid 9s suffix.
    // Append suffix.
    let new_nines = "9".repeat(min_nines);
    format!("{}_{}_P", base, new_nines)
}

pub fn record_installed_tags(base_name: &str, tags: &Vec<String>) {
    if tags.is_empty() { return; }
    let mut cfg_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    cfg_dir.push("repak_manager");
    let _ = fs::create_dir_all(&cfg_dir);
    let mut path = cfg_dir.clone();
    path.push("pending_custom_tags.json");

    let mut map: BTreeMap<String, Vec<String>> = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<BTreeMap<String, Vec<String>>>(&s).ok())
            .unwrap_or_default()
    } else { BTreeMap::new() };

    let entry = map.entry(base_name.to_string()).or_default();
    for t in tags {
        if !entry.contains(t) { entry.push(t.clone()); }
    }
    entry.sort();
    entry.dedup();
    let _ = fs::write(&path, serde_json::to_string_pretty(&map).unwrap());
}

pub fn install_mods_in_viewport(
    mods: &mut [InstallableMod],
    mod_directory: &Path,
    installed_mods_ptr: &AtomicI32,
    stop_thread: &AtomicBool,
) {
    let mut type_tracker: HashMap<String, usize> = HashMap::new();

    for installable_mod in mods.iter_mut() {
        let min_nines = if installable_mod.enabled {
             let count = type_tracker.entry(installable_mod.mod_type.clone()).or_insert(0);
             let n = 7 + *count;
             *count += 1;
             n
        } else {
             7
        };

        // Ensure naming suffix consistency up-front for all flows
        installable_mod.mod_name = normalize_mod_base_name(&installable_mod.mod_name, min_nines);
        
        if !installable_mod.enabled {
            continue;
        }
        
        if stop_thread.load(Ordering::SeqCst) {
            warn!("Stopping thread");
            break;
        }

        // Determine the actual output directory (base + subfolder if specified)
        let output_directory = if installable_mod.install_subfolder.is_empty() {
            mod_directory.to_path_buf()
        } else {
            let subfolder_path = mod_directory.join(&installable_mod.install_subfolder);
            // Create the subfolder if it doesn't exist
            if !subfolder_path.exists() {
                if let Err(e) = fs::create_dir_all(&subfolder_path) {
                    error!("Failed to create subfolder '{}': {}", installable_mod.install_subfolder, e);
                    continue;
                }
                info!("Created install subfolder: {}", subfolder_path.display());
            }
            subfolder_path
        };

        if installable_mod.iostore {
            // copy the iostore files
            let pak_path = installable_mod.mod_path.with_extension("pak");
            let utoc_path = installable_mod.mod_path.with_extension("utoc");
            let ucas_path = installable_mod.mod_path.with_extension("ucas");

            // Ensure output names follow suffix rule
            let base = normalize_mod_base_name(&installable_mod.mod_name, 7);
            let dests = vec![
                (pak_path, format!("{}.pak", base)),
                (utoc_path, format!("{}.utoc", base)),
                (ucas_path, format!("{}.ucas", base)),
            ];

            for (src, dest_name) in dests {
                if let Err(e) = std::fs::copy(&src, output_directory.join(&dest_name)) {
                    error!("Unable to copy file {:?}: {:?}", src, e);
                }
            }
            // Record tags for pickup by main app
            record_installed_tags(&base, &installable_mod.custom_tags);
            continue;
        }

        if installable_mod.repak {
            if let Err(e) = create_repak_from_pak(
                installable_mod,
                output_directory.clone(),
                installed_mods_ptr,
            ) {
                error!("Failed to create repak from pak: {}", e);
            } else {
                let base = normalize_mod_base_name(&installable_mod.mod_name, 7);
                record_installed_tags(&base, &installable_mod.custom_tags);
            }
        }

        // This shit shouldnt even be possible why do I still have this in the codebase???
        if !installable_mod.repak && !installable_mod.is_dir {
            // just move files to the correct location
            info!(
                "Copying mod instead of repacking: {}",
                installable_mod.mod_name
            );
            let base = normalize_mod_base_name(&installable_mod.mod_name, 7);
            std::fs::copy(&installable_mod.mod_path, output_directory.join(format!("{}.pak", &base)))
            .unwrap();
            record_installed_tags(&base, &installable_mod.custom_tags);
            installed_mods_ptr.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            continue;
        }

        if installable_mod.is_dir {
            // Copy source directory to temp dir to avoid modifying original files
            let temp_dir = match tempfile::tempdir() {
                Ok(dir) => dir,
                Err(e) => {
                    error!("Failed to create temp directory: {}", e);
                    continue;
                }
            };
            let temp_path = temp_dir.path().to_path_buf();
            
            // Copy all files from source to temp
            let source_path = PathBuf::from(&installable_mod.mod_path);
            if let Err(e) = copy_dir_recursive(&source_path, &temp_path) {
                error!("Failed to copy mod files to temp directory: {}", e);
                continue;
            }
            info!("Copied mod files to temp directory for processing");
            
            let res = convert_to_iostore_directory(
                installable_mod,
                output_directory.clone(),
                temp_path,
                installed_mods_ptr,
            );
            // temp_dir is automatically cleaned up when it goes out of scope
            if let Err(e) = res {
                error!("Failed to create repak from pak: {}", e);
            } else {
                info!("Installed mod: {}", installable_mod.mod_name);
            }
        }
    }
    // set i32 to -255 magic value to indicate mod installation is done
    AtomicI32::store(installed_mods_ptr, -255, Ordering::SeqCst);
}
