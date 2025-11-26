use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).parent().unwrap().parent().unwrap().parent().unwrap();
    let bridge_output_dir: PathBuf = target_dir.join("uassetbridge");

    // Get the workspace root (two levels up from uasset_app: uasset_app -> uasset_toolkit -> workspace root)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let bridge_project_dir = workspace_root.join("uasset_toolkit").join("tools").join("UAssetBridge");

    // Only emit rerun-if-changed if the files actually exist
    let program_cs = bridge_project_dir.join("Program.cs");
    let csproj = bridge_project_dir.join("UAssetBridge.csproj");
    if program_cs.exists() {
        println!("cargo:rerun-if-changed={}", program_cs.display());
    }
    if csproj.exists() {
        println!("cargo:rerun-if-changed={}", csproj.display());
    }

    // Create output directory
    if let Err(e) = fs::create_dir_all(&bridge_output_dir) {
        println!("cargo:warning=failed to create {}: {}", bridge_output_dir.display(), e);
    }

    let dest_exe = bridge_output_dir.join("UAssetBridge.exe");

    // 1) Try to publish via dotnet into target/uassetbridge
    let mut published = false;
    let dotnet_available = Command::new("dotnet").arg("--version").output().is_ok();
    if dotnet_available {
        let status = Command::new("dotnet")
            .current_dir(&bridge_project_dir)
            .args([
                "publish",
                "-c",
                "Release",
                "-r",
                "win-x64",
                "--self-contained",
                "false",
                "-o",
                &bridge_output_dir.to_string_lossy(),
            ])
            .status();
        match status {
            Ok(s) if s.success() => {
                if dest_exe.exists() {
                    println!("cargo:warning=UAssetBridge published to {}", dest_exe.display());
                    published = true;
                } else {
                    println!("cargo:warning=dotnet publish succeeded but {} not found", dest_exe.display());
                }
            }
            Ok(s) => {
                println!("cargo:warning=dotnet publish failed with status: {}", s);
            }
            Err(e) => {
                println!("cargo:warning=failed to run dotnet publish: {}", e);
            }
        }
    } else {
        println!("cargo:warning=dotnet not found; attempting to use precompiled UAssetBridge.exe");
    }

    // 2) If publish not successful, fallback to existing precompiled build
    if !published {
        let precompiled_paths = [
            bridge_project_dir.join("bin").join("Release").join("net8.0").join("win-x64").join("UAssetBridge.exe"),
            bridge_project_dir.join("bin").join("Debug").join("net8.0").join("win-x64").join("UAssetBridge.exe"),
        ];

        let mut copied = false;
        for precompiled_path in &precompiled_paths {
            if precompiled_path.exists() {
                if let Err(e) = fs::copy(precompiled_path, &dest_exe) {
                    println!("cargo:warning=Failed to copy precompiled {} to {}: {}", precompiled_path.display(), dest_exe.display(), e);
                    continue;
                }
                println!("cargo:warning=Using precompiled UAssetBridge from: {}", precompiled_path.display());
                println!("cargo:warning=UAssetBridge copied to: {}", dest_exe.display());
                copied = true;
                break;
            }
        }

        if !copied {
            panic!("UAssetBridge.exe is required but was not produced. Ensure .NET SDK is installed or precompile via: 'dotnet publish tools/UAssetBridge -c Release -r win-x64 --self-contained false'");
        }
    }
}
