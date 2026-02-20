use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the runtime identifier for the current target platform
fn get_runtime_identifier() -> &'static str {
    // Check CARGO_CFG_TARGET_OS which is set during cross-compilation
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "windows".to_string() // default fallback
        }
    });
    
    match target_os.as_str() {
        "linux" => "linux-x64",
        "macos" => "osx-x64",
        _ => "win-x64",
    }
}

/// Get the executable name for the current target platform
fn get_executable_name() -> &'static str {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    });
    
    if target_os == "windows" {
        "UAssetTool.exe"
    } else {
        "UAssetTool"
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let tool_output_dir: PathBuf = target_dir.join("uassettool");

    // Get the workspace root (two levels up from uasset_app: uasset_app -> uasset_toolkit -> workspace root)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();

    // Use UAssetTool from UassetToolRivals submodule
    let tool_project_dir = workspace_root
        .join("UassetToolRivals")
        .join("src")
        .join("UAssetTool");

    // Watch all C# source files for changes
    let program_cs = tool_project_dir.join("Program.cs");
    let csproj = tool_project_dir.join("UAssetTool.csproj");
    if program_cs.exists() {
        println!("cargo:rerun-if-changed={}", program_cs.display());
    }
    if csproj.exists() {
        println!("cargo:rerun-if-changed={}", csproj.display());
    }

    // Watch UAssetAPI source directory for changes (now in submodule)
    let uasset_api_dir = workspace_root
        .join("UassetToolRivals")
        .join("src")
        .join("UAssetAPI");
    if uasset_api_dir.exists() {
        println!("cargo:rerun-if-changed={}", uasset_api_dir.display());
    }

    // Create output directory
    if let Err(e) = fs::create_dir_all(&tool_output_dir) {
        println!(
            "cargo:warning=failed to create {}: {}",
            tool_output_dir.display(),
            e
        );
    }

    let runtime_id = get_runtime_identifier();
    let exe_name = get_executable_name();
    let dest_exe = tool_output_dir.join(exe_name);

    println!("cargo:warning=Building for runtime: {}, executable: {}", runtime_id, exe_name);

    // Force rebuild if output doesn't exist
    if !dest_exe.exists() {
        println!("cargo:rerun-if-changed=build.rs");
    }

    // Check if we should skip the build (e.g. if build_contributor.ps1 already built it)
    if env::var("SKIP_UASSET_TOOL_BUILD").is_ok() {
        println!("cargo:warning=Skipping UAssetTool build because SKIP_UASSET_TOOL_BUILD is set");
        if dest_exe.exists() {
            return;
        } else {
            println!("cargo:warning=SKIP_UASSET_TOOL_BUILD is set but {} does not exist. Falling back to build.", dest_exe.display());
        }
    }

    // 1) Try to publish via dotnet into target/uassettool
    let mut published = false;
    let dotnet_available = Command::new("dotnet").arg("--version").output().is_ok();
    if dotnet_available {
        let status = Command::new("dotnet")
            .current_dir(&tool_project_dir)
            .args([
                "publish",
                "-c",
                "Release",
                "-r",
                runtime_id,
                "--self-contained",
                "true",
                "-o",
                &tool_output_dir.to_string_lossy(),
            ])
            .status();
        match status {
            Ok(s) if s.success() => {
                if dest_exe.exists() {
                    println!(
                        "cargo:warning=UAssetTool published to {}",
                        dest_exe.display()
                    );
                    published = true;
                } else {
                    println!(
                        "cargo:warning=dotnet publish succeeded but {} not found",
                        dest_exe.display()
                    );
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
        println!("cargo:warning=dotnet not found; attempting to use precompiled UAssetTool");
    }

    // 2) If publish not successful, fallback to existing precompiled build
    if !published {
        let precompiled_paths = [
            tool_project_dir
                .join("bin")
                .join("Release")
                .join("net8.0")
                .join(runtime_id)
                .join("publish")
                .join(exe_name),
            tool_project_dir
                .join("bin")
                .join("Release")
                .join("net8.0")
                .join(runtime_id)
                .join(exe_name),
            tool_project_dir
                .join("bin")
                .join("Debug")
                .join("net8.0")
                .join(runtime_id)
                .join(exe_name),
        ];

        let mut copied = false;
        for precompiled_path in &precompiled_paths {
            if precompiled_path.exists() {
                if let Err(e) = fs::copy(precompiled_path, &dest_exe) {
                    println!(
                        "cargo:warning=Failed to copy precompiled {} to {}: {}",
                        precompiled_path.display(),
                        dest_exe.display(),
                        e
                    );
                    continue;
                }
                println!(
                    "cargo:warning=Using precompiled UAssetTool from: {}",
                    precompiled_path.display()
                );
                println!("cargo:warning=UAssetTool copied to: {}", dest_exe.display());
                copied = true;
                break;
            }
        }

        if !copied {
            let build_cmd = format!("dotnet publish UassetToolRivals/src/UAssetTool -c Release -r {} --self-contained true", runtime_id);
            panic!("UAssetTool is required but was not produced. Ensure .NET SDK is installed or precompile via: '{}'", build_cmd);
        }
    }
}
