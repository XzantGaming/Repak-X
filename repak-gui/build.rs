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
        let tools_dir = workspace_root.join("tools").join("UAssetBridge");
        let fallback_release = tools_dir.join("bin").join("Release").join("net8.0").join("win-x64").join("UAssetBridge.exe");
        let fallback_debug = tools_dir.join("bin").join("Debug").join("net8.0").join("win-x64").join("UAssetBridge.exe");

        // Pick the first existing source
        let source = if primary_src.exists() {
            Some(primary_src)
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
                match fs::copy(&src, &dest_path) {
                    Ok(_) => {
                        println!("cargo:warning=UAssetBridge copied to {}", dest_path.display());
                    }
                    Err(e) => {
                        println!("cargo:warning=failed to copy {} to {}: {}", src.display(), dest_path.display(), e);
                    }
                }
            }
        } else {
            println!("cargo:warning=UAssetBridge.exe not found. To enable texture pipeline, build it via: 'dotnet publish tools/UAssetBridge -c Release -r win-x64 --self-contained false'");
        }
    }
}