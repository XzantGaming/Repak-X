use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Command as AsyncCommand, Child, ChildStdin, ChildStdout};
use tokio::io::{BufReader, AsyncBufReadExt, AsyncWriteExt, Lines};
use tokio::sync::Mutex;

#[cfg(windows)]
#[allow(unused_imports)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum UAssetRequest {
    #[serde(rename = "detect_texture")]
    DetectTexture { file_path: String },
    #[serde(rename = "set_mip_gen")]
    SetMipGen { file_path: String, mip_gen: String },
    #[serde(rename = "get_texture_info")]
    GetTextureInfo { file_path: String },
    #[serde(rename = "detect_mesh")]
    DetectMesh { file_path: String },
    #[serde(rename = "detect_skeletal_mesh")]
    DetectSkeletalMesh { file_path: String },
    #[serde(rename = "detect_static_mesh")]
    DetectStaticMesh { file_path: String },
    #[serde(rename = "patch_mesh")]
    PatchMesh { file_path: String, uexp_path: String },
    #[serde(rename = "get_mesh_info")]
    GetMeshInfo { file_path: String },
    // Batch detection - sends all files at once, returns first match
    #[serde(rename = "batch_detect_skeletal_mesh")]
    BatchDetectSkeletalMesh { file_paths: Vec<String> },
    #[serde(rename = "batch_detect_static_mesh")]
    BatchDetectStaticMesh { file_paths: Vec<String> },
    #[serde(rename = "batch_detect_texture")]
    BatchDetectTexture { file_paths: Vec<String> },
    #[serde(rename = "batch_detect_blueprint")]
    BatchDetectBlueprint { file_paths: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UAssetResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureInfo {
    pub mip_gen_settings: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshInfo {
    pub material_count: Option<i32>,
    pub vertex_count: Option<i32>,
    pub triangle_count: Option<i32>,
    pub is_skeletal_mesh: Option<bool>,
}

struct ChildProcess {
    _child: Child,
    stdin: ChildStdin,
    reader: Lines<BufReader<ChildStdout>>,
}

pub struct UAssetToolkit {
    bridge_path: String,
    process: Mutex<Option<ChildProcess>>,
}

impl UAssetToolkit {
    /// Create a new UAssetToolkit instance
    /// 
    /// # Arguments
    /// * `bridge_path` - Path to the UAssetBridge executable. If None, will try to find it in target/uassetbridge/
    pub fn new(bridge_path: Option<String>) -> Result<Self> {
        let bridge_path = match bridge_path {
            Some(path) => path,
            None => {
                // Try to find the bridge in the expected location
                let exe_path = std::env::current_exe()?;
                let exe_dir = exe_path.parent().context("Failed to get executable directory")?;
                let bridge_path = exe_dir.join("uassetbridge").join("UAssetBridge.exe");
                
                if !bridge_path.exists() {
                    // Try relative to workspace
                    let workspace_bridge = Path::new("target/uassetbridge/UAssetBridge.exe");
                    if workspace_bridge.exists() {
                        workspace_bridge.to_string_lossy().to_string()
                    } else {
                        // Try looking in the source tools folder as fallback for dev
                        let dev_bridge = Path::new("tools/UAssetBridge/bin/Debug/net8.0/win-x64/UAssetBridge.exe");
                         if dev_bridge.exists() {
                            dev_bridge.to_string_lossy().to_string()
                        } else {
                             // Default assumption
                             bridge_path.to_string_lossy().to_string()
                        }
                    }
                } else {
                    bridge_path.to_string_lossy().to_string()
                }
            }
        };

        Ok(Self { 
            bridge_path,
            process: Mutex::new(None),
        })
    }

    async fn send_request(&self, request: UAssetRequest) -> Result<UAssetResponse> {
        let mut process_guard = self.process.lock().await;

        if process_guard.is_none() {
             if !Path::new(&self.bridge_path).exists() {
                 anyhow::bail!("UAssetBridge executable not found at: {}", self.bridge_path);
             }

            let mut cmd = AsyncCommand::new(&self.bridge_path);
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            
            // Explicitly pass USMAP_PATH to child process
            if let Ok(usmap_path) = std::env::var("USMAP_PATH") {
                cmd.env("USMAP_PATH", &usmap_path);
            }
            
            #[cfg(windows)]
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW flag on Windows
            
            let mut child = cmd.spawn()
                .context("Failed to spawn UAssetBridge process")?;

            let stdin = child.stdin.take().context("Failed to get stdin")?;
            let stdout = child.stdout.take().context("Failed to get stdout")?;
            let reader = BufReader::new(stdout).lines();
            
            *process_guard = Some(ChildProcess { _child: child, stdin, reader });
        }

        let proc = process_guard.as_mut().unwrap();
        let request_json = serde_json::to_string(&request)?;
        
        if let Err(e) = proc.stdin.write_all(request_json.as_bytes()).await {
            *process_guard = None;
            anyhow::bail!("Failed to write to UAssetBridge (process likely died): {}", e);
        }
        
        if let Err(e) = proc.stdin.write_all(b"\n").await {
            *process_guard = None;
            anyhow::bail!("Failed to write newline to UAssetBridge: {}", e);
        }
        
        if let Err(e) = proc.stdin.flush().await {
            *process_guard = None;
            anyhow::bail!("Failed to flush to UAssetBridge: {}", e);
        }

        match proc.reader.next_line().await {
            Ok(Some(line)) => {
                match serde_json::from_str::<UAssetResponse>(&line) {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        *process_guard = None;
                        anyhow::bail!("Failed to parse response from UAssetBridge: {} (Line: {})", e, line);
                    }
                }
            },
            Ok(None) => {
                *process_guard = None;
                anyhow::bail!("UAssetBridge process closed connection (EOF)");
            },
            Err(e) => {
                *process_guard = None;
                anyhow::bail!("Failed to read from UAssetBridge: {}", e);
            }
        }
    }

    /// Check if a uasset file is a texture asset
    pub async fn is_texture_uasset(&self, file_path: &str) -> Result<bool> {
        let request = UAssetRequest::DetectTexture {
            file_path: file_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to detect texture: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Check if a uasset file is a mesh asset
    pub async fn is_mesh_uasset(&self, file_path: &str) -> Result<bool> {
        let request = UAssetRequest::DetectMesh {
            file_path: file_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to detect mesh: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Check if a uasset file is a skeletal mesh asset
    pub async fn is_skeletal_mesh_uasset(&self, file_path: &str) -> Result<bool> {
        let request = UAssetRequest::DetectSkeletalMesh {
            file_path: file_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to detect skeletal mesh: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Check if a uasset file is a static mesh asset
    pub async fn is_static_mesh_uasset(&self, file_path: &str) -> Result<bool> {
        let request = UAssetRequest::DetectStaticMesh {
            file_path: file_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to detect static mesh: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Set mip generation settings to NoMipmaps for a texture uasset
    pub async fn set_no_mipmaps(&self, file_path: &str) -> Result<()> {
        self.set_mip_gen(file_path, "NoMipmaps").await
    }

    /// Set mip generation settings for a texture uasset
    pub async fn set_mip_gen(&self, file_path: &str, mip_gen: &str) -> Result<()> {
        let request = UAssetRequest::SetMipGen {
            file_path: file_path.to_string(),
            mip_gen: mip_gen.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to set mip gen: {}", response.message);
        }
        
        Ok(())
    }

    /// Get texture information from a uasset file
    pub async fn get_texture_info(&self, file_path: &str) -> Result<TextureInfo> {
        let request = UAssetRequest::GetTextureInfo {
            file_path: file_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to get texture info: {}", response.message);
        }
        
        let data = response.data.unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        let texture_info = TextureInfo {
            mip_gen_settings: data.get("MipGenSettings").and_then(|v| v.as_str()).map(|s| s.to_string()),
            width: data.get("Width").and_then(|v| v.as_i64()).map(|i| i as i32),
            height: data.get("Height").and_then(|v| v.as_i64()).map(|i| i as i32),
            format: data.get("Format").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };
        
        Ok(texture_info)
    }

    /// Patch mesh materials in a uasset file
    pub async fn patch_mesh(&self, file_path: &str, uexp_path: &str) -> Result<()> {
        let request = UAssetRequest::PatchMesh {
            file_path: file_path.to_string(),
            uexp_path: uexp_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to patch mesh: {}", response.message);
        }
        
        Ok(())
    }

    /// Get mesh information from a uasset file
    pub async fn get_mesh_info(&self, file_path: &str) -> Result<MeshInfo> {
        let request = UAssetRequest::GetMeshInfo {
            file_path: file_path.to_string(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to get mesh info: {}", response.message);
        }
        
        let data = response.data.unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        let mesh_info = MeshInfo {
            material_count: data.get("MaterialCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            vertex_count: data.get("VertexCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            triangle_count: data.get("TriangleCount").and_then(|v| v.as_i64()).map(|i| i as i32),
            is_skeletal_mesh: data.get("IsSkeletalMesh").and_then(|v| v.as_bool()),
        };
        
        Ok(mesh_info)
    }

    /// Process a uasset file: detect if it's a texture and set NoMipmaps if it is
    /// Returns true if the file was processed (was a texture), false otherwise
    pub async fn process_texture_uasset(&self, file_path: &str) -> Result<bool> {
        if self.is_texture_uasset(file_path).await? {
            self.set_no_mipmaps(file_path).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Process a mesh uasset file: detect if it's a mesh and patch materials if it is
    /// Returns true if the file was processed (was a mesh), false otherwise
    pub async fn process_mesh_uasset(&self, file_path: &str, uexp_path: &str) -> Result<bool> {
        if self.is_mesh_uasset(file_path).await? {
            self.patch_mesh(file_path, uexp_path).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Batch detect skeletal meshes - sends all paths at once, returns true if any match
    pub async fn batch_detect_skeletal_mesh(&self, file_paths: &[String]) -> Result<bool> {
        let request = UAssetRequest::BatchDetectSkeletalMesh {
            file_paths: file_paths.to_vec(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to batch detect skeletal mesh: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Batch detect static meshes - sends all paths at once, returns true if any match
    pub async fn batch_detect_static_mesh(&self, file_paths: &[String]) -> Result<bool> {
        let request = UAssetRequest::BatchDetectStaticMesh {
            file_paths: file_paths.to_vec(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to batch detect static mesh: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Batch detect textures - sends all paths at once, returns true if any match
    pub async fn batch_detect_texture(&self, file_paths: &[String]) -> Result<bool> {
        let request = UAssetRequest::BatchDetectTexture {
            file_paths: file_paths.to_vec(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to batch detect texture: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Batch detect blueprints - sends all paths at once, returns true if any match
    pub async fn batch_detect_blueprint(&self, file_paths: &[String]) -> Result<bool> {
        let request = UAssetRequest::BatchDetectBlueprint {
            file_paths: file_paths.to_vec(),
        };
        
        let response = self.send_request(request).await?;
        
        if !response.success {
            anyhow::bail!("Failed to batch detect blueprint: {}", response.message);
        }
        
        Ok(response.data
            .and_then(|d| d.as_bool())
            .unwrap_or(false))
    }

    /// Batch process multiple uasset files
    /// Returns a vector of (file_path, was_processed, error_message)
    pub async fn batch_process_textures(&self, file_paths: &[String]) -> Vec<(String, bool, Option<String>)> {
        let mut results = Vec::new();
        
        for file_path in file_paths {
            match self.process_texture_uasset(file_path).await {
                Ok(was_processed) => results.push((file_path.clone(), was_processed, None)),
                Err(e) => results.push((file_path.clone(), false, Some(e.to_string()))),
            }
        }
        
        results
    }
}

/// Synchronous wrapper for common operations (blocks on tokio runtime)
/// Handles both cases: when called from within an existing runtime (uses block_in_place)
/// and when called from outside a runtime (creates its own)
pub struct UAssetToolkitSync {
    toolkit: UAssetToolkit,
    runtime: Option<tokio::runtime::Runtime>,
}

impl UAssetToolkitSync {
    pub fn new(bridge_path: Option<String>) -> Result<Self> {
        let toolkit = UAssetToolkit::new(bridge_path)?;
        
        // Check if we're already inside a tokio runtime
        let runtime = if tokio::runtime::Handle::try_current().is_ok() {
            // Already in a runtime, don't create a new one
            None
        } else {
            // Not in a runtime, create one
            Some(tokio::runtime::Runtime::new()?)
        };
        
        Ok(Self { toolkit, runtime })
    }

    fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        if let Some(ref rt) = self.runtime {
            // We have our own runtime, use it
            rt.block_on(future)
        } else {
            // We're inside an existing runtime, use block_in_place
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(future)
            })
        }
    }

    pub fn is_texture_uasset(&self, file_path: &str) -> Result<bool> {
        self.block_on(self.toolkit.is_texture_uasset(file_path))
    }

    pub fn is_mesh_uasset(&self, file_path: &str) -> Result<bool> {
        self.block_on(self.toolkit.is_mesh_uasset(file_path))
    }

    pub fn is_skeletal_mesh_uasset(&self, file_path: &str) -> Result<bool> {
        self.block_on(self.toolkit.is_skeletal_mesh_uasset(file_path))
    }

    pub fn is_static_mesh_uasset(&self, file_path: &str) -> Result<bool> {
        self.block_on(self.toolkit.is_static_mesh_uasset(file_path))
    }

    pub fn set_no_mipmaps(&self, file_path: &str) -> Result<()> {
        self.block_on(self.toolkit.set_no_mipmaps(file_path))
    }

    pub fn patch_mesh(&self, file_path: &str, uexp_path: &str) -> Result<()> {
        self.block_on(self.toolkit.patch_mesh(file_path, uexp_path))
    }

    pub fn process_texture_uasset(&self, file_path: &str) -> Result<bool> {
        self.block_on(self.toolkit.process_texture_uasset(file_path))
    }

    pub fn process_mesh_uasset(&self, file_path: &str, uexp_path: &str) -> Result<bool> {
        self.block_on(self.toolkit.process_mesh_uasset(file_path, uexp_path))
    }

    pub fn get_texture_info(&self, file_path: &str) -> Result<TextureInfo> {
        self.block_on(self.toolkit.get_texture_info(file_path))
    }

    pub fn get_mesh_info(&self, file_path: &str) -> Result<MeshInfo> {
        self.block_on(self.toolkit.get_mesh_info(file_path))
    }
}
