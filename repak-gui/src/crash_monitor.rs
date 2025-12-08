// Crash Monitor - Detects Marvel Rivals crashes and provides crash information
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs;
use log::{debug, info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashInfo {
    pub crash_folder: PathBuf,
    pub timestamp: SystemTime,
    pub error_message: Option<String>,
    pub crash_type: Option<String>,
    pub seconds_since_start: Option<u64>,
    pub process_id: Option<u32>,
    pub enabled_mods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecord {
    pub timestamp: String,
    pub error_summary: String,
    pub seconds_in_game: u64,
    pub mods_enabled: Vec<String>,
    pub crash_folder: PathBuf,
}

/// Get the path to Marvel Rivals crash logs directory
pub fn get_crash_log_path() -> PathBuf {
    let local_appdata = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| {
            // Fallback to default Windows path
            let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
            format!("{}\\AppData\\Local", userprofile)
        });
    
    PathBuf::from(local_appdata)
        .join("Marvel")
        .join("Saved")
        .join("Crashes")
}

/// Check for new crash folders created after a specific time
pub fn check_for_new_crashes(since: SystemTime) -> Vec<PathBuf> {
    let crash_dir = get_crash_log_path();
    
    if !crash_dir.exists() {
        debug!("Crash directory does not exist: {:?}", crash_dir);
        return Vec::new();
    }
    
    let mut new_crashes = Vec::new();
    
    match fs::read_dir(&crash_dir) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                // Only check directories
                if !path.is_dir() {
                    continue;
                }
                
                // Check if folder was created after the specified time
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        if created > since {
                            info!("New crash detected: {:?}", path);
                            new_crashes.push(path);
                        }
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to read crash directory: {}", e);
        }
    }
    
    new_crashes
}

/// Parse crash information from a crash folder
pub fn parse_crash_info(crash_folder: &Path, enabled_mods: Vec<String>) -> Option<CrashInfo> {
    let crash_context_path = crash_folder.join("CrashContext.runtime-xml");
    
    if !crash_context_path.exists() {
        warn!("CrashContext.runtime-xml not found in {:?}", crash_folder);
        return None;
    }
    
    let timestamp = fs::metadata(crash_folder)
        .and_then(|m| m.created())
        .unwrap_or_else(|_| SystemTime::now());
    
    // Try to read and parse the XML file
    let xml_content = match fs::read_to_string(&crash_context_path) {
        Ok(content) => {
            // Validate XML content size to prevent memory issues
            if content.len() > 10_000_000 { // 10MB limit
                warn!("CrashContext.runtime-xml file too large ({}B), skipping detailed parsing", content.len());
                return Some(CrashInfo {
                    crash_folder: crash_folder.to_path_buf(),
                    timestamp,
                    error_message: Some("Crash file too large to parse".to_string()),
                    crash_type: None,
                    seconds_since_start: None,
                    process_id: None,
                    enabled_mods,
                });
            }
            content
        }
        Err(e) => {
            warn!("Failed to read CrashContext.runtime-xml: {}", e);
            return Some(CrashInfo {
                crash_folder: crash_folder.to_path_buf(),
                timestamp,
                error_message: None,
                crash_type: None,
                seconds_since_start: None,
                process_id: None,
                enabled_mods,
            });
        }
    };
    
    // Simple XML parsing (extract key values)
    let error_message = extract_xml_tag(&xml_content, "ErrorMessage");
    let crash_type = extract_xml_tag(&xml_content, "CrashType");
    let seconds_since_start = extract_xml_tag(&xml_content, "SecondsSinceStart")
        .and_then(|s| s.parse::<u64>().ok());
    let process_id = extract_xml_tag(&xml_content, "ProcessId")
        .and_then(|s| s.parse::<u32>().ok());
    
    info!("Parsed crash info - Error: {:?}, Type: {:?}, Time: {:?}s", 
          error_message, crash_type, seconds_since_start);
    
    Some(CrashInfo {
        crash_folder: crash_folder.to_path_buf(),
        timestamp,
        error_message,
        crash_type,
        seconds_since_start,
        process_id,
        enabled_mods,
    })
}

/// Extract a value from an XML tag (simple parser, not full XML)
/// Includes validation to prevent crashes from malformed XML
fn extract_xml_tag(xml: &str, tag_name: &str) -> Option<String> {
    // Validate input parameters
    if xml.is_empty() || tag_name.is_empty() {
        return None;
    }
    
    let start_tag = format!("<{}>", tag_name);
    let end_tag = format!("</{}>", tag_name);
    
    if let Some(start_pos) = xml.find(&start_tag) {
        let content_start = start_pos + start_tag.len();
        
        // Validate bounds to prevent panics
        if content_start >= xml.len() {
            warn!("XML parsing error: invalid content start position for tag '{}'", tag_name);
            return None;
        }
        
        if let Some(end_pos) = xml[content_start..].find(&end_tag) {
            let content_end = content_start + end_pos;
            
            // Additional bounds check
            if content_end > xml.len() {
                warn!("XML parsing error: invalid content end position for tag '{}'", tag_name);
                return None;
            }
            
            let value = xml[content_start..content_end].trim();
            
            // Don't return empty values
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    
    None
}

/// Extract specific crash details from error message
/// Returns (asset_path, error_type, details)
pub fn parse_error_details(error_message: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut asset_path = None;
    let mut error_type = None;
    let mut details = None;
    
    // Extract asset path (e.g., /Game/Marvel/Characters/1033/1033001/...)
    if let Some(game_pos) = error_message.find("/Game/") {
        if let Some(space_pos) = error_message[game_pos..].find(' ') {
            asset_path = Some(error_message[game_pos..game_pos + space_pos].to_string());
        }
    }
    
    // Extract error type (e.g., "ObjectSerializationError", "Serial size mismatch")
    if error_message.contains("ObjectSerializationError") {
        error_type = Some("ObjectSerializationError".to_string());
    } else if error_message.contains("EXCEPTION_ACCESS_VIOLATION") {
        error_type = Some("Access Violation".to_string());
    } else if error_message.contains("Assertion failed") {
        error_type = Some("Assertion Failed".to_string());
    }
    
    // Extract serial size mismatch details
    if let Some(mismatch_pos) = error_message.find("Serial size mismatch:") {
        if let Some(end_pos) = error_message[mismatch_pos..].find('\n').or(Some(error_message.len() - mismatch_pos)) {
            details = Some(error_message[mismatch_pos..mismatch_pos + end_pos].to_string());
        }
    }
    
    (asset_path, error_type, details)
}

/// Check if crash is related to mesh/asset loading
pub fn is_mesh_related_crash(error_message: &str) -> bool {
    error_message.contains("StaticMesh") 
        || error_message.contains("SM_") 
        || error_message.contains("Serial size mismatch")
        || error_message.contains("ObjectSerializationError")
}

/// Extract character ID from error message (e.g., "1033" from path)
pub fn extract_character_id(error_message: &str) -> Option<String> {
    // Look for pattern like /Characters/1033/ or /1033/
    if let Some(chars_pos) = error_message.find("/Characters/") {
        let after_chars = &error_message[chars_pos + 12..]; // Skip "/Characters/"
        if let Some(slash_pos) = after_chars.find('/') {
            let potential_id = &after_chars[..slash_pos];
            // Check if it's a 4-digit number starting with 10
            if potential_id.len() == 4 && potential_id.starts_with("10") {
                return Some(potential_id.to_string());
            }
        }
    }
    None
}

/// Count total number of crash folders
pub fn count_total_crashes() -> usize {
    let crash_dir = get_crash_log_path();
    
    if !crash_dir.exists() {
        return 0;
    }
    
    match fs::read_dir(&crash_dir) {
        Ok(entries) => {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_dir())
                .count()
        }
        Err(e) => {
            warn!("Failed to count crash folders: {}", e);
            0
        }
    }
}

/// Format SystemTime to human readable string
fn format_system_time(time: SystemTime) -> String {
    match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => {
            // Simple timestamp formatting
            let secs = duration.as_secs();
            let minutes = (secs / 60) % 60;
            let hours = (secs / 3600) % 24;
            format!("{}h {}m ago", hours, minutes)
        }
        Err(_) => "Unknown time".to_string(),
    }
}

/// Convert CrashInfo to CrashRecord for persistent storage
pub fn crash_info_to_record(info: &CrashInfo) -> CrashRecord {
    let timestamp_str = format_system_time(info.timestamp);
    let error_summary = info.error_message
        .clone()
        .unwrap_or_else(|| "Unknown crash".to_string());
    let seconds_in_game = info.seconds_since_start.unwrap_or(0);
    
    CrashRecord {
        timestamp: timestamp_str,
        error_summary,
        seconds_in_game,
        mods_enabled: info.enabled_mods.clone(),
        crash_folder: info.crash_folder.clone(),
    }
}


/// Delete all crash folders (cleanup)
pub fn clear_all_crashes() -> Result<usize, String> {
    let crash_dir = get_crash_log_path();
    
    if !crash_dir.exists() {
        return Ok(0);
    }
    
    let mut deleted_count = 0;
    
    match fs::read_dir(&crash_dir) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                if path.is_dir() {
                    match fs::remove_dir_all(&path) {
                        Ok(_) => {
                            deleted_count += 1;
                            info!("Deleted crash folder: {:?}", path);
                        }
                        Err(e) => {
                            error!("Failed to delete crash folder {:?}: {}", path, e);
                        }
                    }
                }
            }
            Ok(deleted_count)
        }
        Err(e) => Err(format!("Failed to read crash directory: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_xml_tag() {
        let xml = r#"<Root><ErrorMessage>Test Error</ErrorMessage></Root>"#;
        assert_eq!(extract_xml_tag(xml, "ErrorMessage"), Some("Test Error".to_string()));
    }
    
    #[test]
    fn test_crash_log_path() {
        let path = get_crash_log_path();
        assert!(path.to_string_lossy().contains("Marvel"));
        assert!(path.to_string_lossy().contains("Crashes"));
    }
}
