use std::path::Path;

// Simplified file entry for mod file table display
#[derive(Clone, Debug)]
#[allow(dead_code)] // Fields are read via file_table.rs
pub struct UtocFileEntry {
    pub file_path: String,
    pub bulkdata_chunks: usize,
    pub packagedata_chunks: usize,
}

/// Read UTOC file list using UAssetTool (replaces retoc crate)
pub fn read_utoc(utoc_path: &Path) -> Vec<UtocFileEntry> {
    match try_read_utoc(utoc_path) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to read utoc {}: {}", utoc_path.display(), e);
            Vec::new()
        }
    }
}

pub fn try_read_utoc(utoc_path: &Path) -> Result<Vec<UtocFileEntry>, String> {
    // Use UAssetTool via uasset_toolkit to list IoStore files
    let result = uasset_toolkit::list_iostore_files(
        utoc_path.to_string_lossy().as_ref(),
        None, // Use default AES key
    ).map_err(|e| format!("Failed to read utoc: {}", e))?;
    
    // Convert to UtocFileEntry format
    let entries = result.files.iter().map(|file_path| {
        UtocFileEntry {
            file_path: file_path.clone(),
            bulkdata_chunks: 0, // Not available from new API
            packagedata_chunks: 0, // Not available from new API
        }
    }).collect();
    
    Ok(entries)
}