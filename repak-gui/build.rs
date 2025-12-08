// extern crate winres; // Disabled for Tauri - Tauri handles icons
fn main() {
    // Tauri build - handles icons and resources
    tauri_build::build();
    #[cfg(windows)]
    {
        use std::{env, fs, path::Path, path::PathBuf};

        // Winres disabled for Tauri to avoid duplicate resources
        // Tauri handles icon embedding via tauri.conf.json
        // let mut res = winres::WindowsResource::new();
        // res.set_icon("icons/RepakIcon.ico");
        // if let Err(e) = res.compile() {
        //     println!("cargo:warning=winres: failed to compile resources: {}", e);
        // }

        // 2) Ensure UAssetBridge.exe is placed next to the built repak-gui.exe under
        //    target/<profile>/uassetbridge/UAssetBridge.exe so runtime lookup succeeds.
        //    Primary source: target/uassetbridge/UAssetBridge.exe (produced by uasset_app build.rs)
        //    Fallback: tools/UAssetBridge/bin/{Release,Debug}/net8.0/win-x64/UAssetBridge.exe

        // Compute key paths from OUT_DIR (…/target/<profile>/build/<crate>…/out)
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let target_dir = out_dir
            .parent().and_then(Path::parent) // …/target/<profile>/build
            .and_then(Path::parent)          // …/target/<profile>
            .and_then(Path::parent)          // …/target
            .map(|p| p.to_path_buf())
            .expect("Failed to derive target directory from OUT_DIR");

        // Determine profile directory (debug/release) to mirror exe location
        let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        let exe_dir = target_dir.join(&profile);
        let dest_dir = exe_dir.join("uassetbridge");
        let dest_path = dest_dir.join("UAssetBridge.exe");

        // Source candidates
        let primary_src = target_dir.join("uassetbridge").join("UAssetBridge.exe");

        // Workspace root: …/Repak_Gui-Revamped/repak-gui -> parent is workspace root
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
        // Update path to match actual location: uasset_toolkit/tools/UAssetBridge
        // Prefer net9.0 exe (old working version from Nov 27) with net8.0 DLLs
        let tools_dir = workspace_root.join("uasset_toolkit").join("tools").join("UAssetBridge");
        let fallback_release_net9 = tools_dir.join("bin").join("Release").join("net9.0").join("win-x64").join("UAssetBridge.exe");
        let fallback_release_publish = tools_dir.join("bin").join("Release").join("net8.0").join("win-x64").join("publish").join("UAssetBridge.exe");
        let fallback_release = tools_dir.join("bin").join("Release").join("net8.0").join("win-x64").join("UAssetBridge.exe");
        let fallback_debug = tools_dir.join("bin").join("Debug").join("net8.0").join("win-x64").join("UAssetBridge.exe");

        // Pick the first existing source
        let source = if primary_src.exists() {
            Some(primary_src)
        } else if fallback_release_net9.exists() {
            Some(fallback_release_net9)
        } else if fallback_release_publish.exists() {
            Some(fallback_release_publish)
        } else if fallback_release.exists() {
            Some(fallback_release)
        } else if fallback_debug.exists() {
            Some(fallback_debug)
        } else {
            None
        };

        if let Some(src) = source {
            if let Err(e) = fs::create_dir_all(&dest_dir) {
                println!("cargo:warning=failed to create {}: {}", dest_dir.display(), e);
            } else {
                // Copy UAssetBridge.exe
                match fs::copy(&src, &dest_path) {
                    Ok(_) => {
                        println!("cargo:warning=UAssetBridge copied to {}", dest_path.display());
                    }
                    Err(e) => {
                        println!("cargo:warning=failed to copy {} to {}: {}", src.display(), dest_path.display(), e);
                    }
                }
                
                // Copy required DLL dependencies and config files
                // For net9.0 exe, get DLLs from net8.0 publish folder since net9.0 lacks them
                let src_dir = src.parent().unwrap();
                let dll_fallback_dir = tools_dir.join("bin").join("Release").join("net8.0").join("win-x64").join("publish");
                
                let required_files = vec![
                    "UAssetBridge.dll", 
                    "UAssetAPI.dll", 
                    "Newtonsoft.Json.dll", 
                    "ZstdSharp.dll",
                    "UAssetBridge.runtimeconfig.json",
                    "UAssetBridge.deps.json"
                ];
                
                for dll_name in required_files {
                    let mut dll_src = src_dir.join(dll_name);
                    
                    // If file doesn't exist in source dir, try fallback (net8.0 publish)
                    if !dll_src.exists() && dll_fallback_dir.exists() {
                        dll_src = dll_fallback_dir.join(dll_name);
                    }
                    
                    let dll_dest = dest_dir.join(dll_name);
                    
                    if dll_src.exists() {
                        match fs::copy(&dll_src, &dll_dest) {
                            Ok(_) => {
                                println!("cargo:warning={} copied to {}", dll_name, dll_dest.display());
                            }
                            Err(e) => {
                                println!("cargo:warning=failed to copy {} to {}: {}", dll_src.display(), dll_dest.display(), e);
                            }
                        }
                    } else {
                        println!("cargo:warning={} not found at {} or fallback", dll_name, src_dir.display());
                    }
                }
            }
        } else {
            println!("cargo:warning=UAssetBridge.exe not found. To enable texture pipeline, build it via: 'dotnet publish tools/UAssetBridge -c Release -r win-x64 --self-contained false'");
        }

        // 3) Copy oo2core_9_win64.dll to the same directory as the executable
        //    Required for repak (oodle compression) to work
        let dll_name = "oo2core_9_win64.dll";
        let dll_src = workspace_root.join(dll_name);
        let dll_dest = exe_dir.join(dll_name);
        
        if dll_src.exists() {
            match fs::copy(&dll_src, &dll_dest) {
                Ok(_) => {
                    println!("cargo:warning={} copied to {}", dll_name, dll_dest.display());
                }
                Err(e) => {
                    println!("cargo:warning=failed to copy {} to {}: {}", dll_name, dll_dest.display(), e);
                }
            }
        } else {
             println!("cargo:warning={} not found at {}", dll_name, dll_src.display());
        }

        // 4) Copy character_data.json to data folder next to the executable
        //    Required for runtime character data lookup
        let char_data_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("data").join("character_data.json");
        let char_data_dest_dir = exe_dir.join("data");
        let char_data_dest = char_data_dest_dir.join("character_data.json");
        
        if char_data_src.exists() {
            if let Err(e) = fs::create_dir_all(&char_data_dest_dir) {
                println!("cargo:warning=failed to create data directory {}: {}", char_data_dest_dir.display(), e);
            } else {
                match fs::copy(&char_data_src, &char_data_dest) {
                    Ok(_) => {
                        println!("cargo:warning=character_data.json copied to {}", char_data_dest.display());
                    }
                    Err(e) => {
                        println!("cargo:warning=failed to copy character_data.json to {}: {}", char_data_dest.display(), e);
                    }
                }
            }
        } else {
            println!("cargo:warning=character_data.json not found at {}", char_data_src.display());
        }
    }
}