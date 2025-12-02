# Subfolder Support Implementation Proposal

This document outlines the necessary changes to enable full nested folder support in the Repak GUI.

## 1. Backend Changes (`repak-gui/src/main_tauri.rs`)

### A. Update `ModFolder` Struct
Ensure the `ModFolder` struct supports hierarchy metadata.

```rust
#[derive(Clone, Serialize, Deserialize)]
struct ModFolder {
    id: String,
    name: String,
    enabled: bool,
    expanded: bool,
    color: Option<[u8; 3]>,
    /// Depth in folder hierarchy (0 = root, 1 = direct child, etc.)
    #[serde(default)]
    depth: usize,
    /// Parent folder ID (None = root folder, "_root" for root's direct children)
    #[serde(default)]
    parent_id: Option<String>,
    /// Is this the root folder (the ~mods directory itself)
    #[serde(default)]
    is_root: bool,
    /// Number of mods directly in this folder
    #[serde(default)]
    mod_count: usize,
}
```

### B. Update `create_folder`
Allow creating nested folders (e.g., "Category/Subcategory") by using `create_dir_all`.

```rust
#[tauri::command]
async fn create_folder(name: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    // Create physical directory in ~mods
    let folder_path = game_path.join(&name);
    
    if folder_path.exists() {
        return Err("Folder already exists".to_string());
    }
    
    // Use create_dir_all to support nested paths like "Category/Subcategory"
    std::fs::create_dir_all(&folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;
    
    Ok(name)
}
```

### C. Update `get_folders`
Recursively scan the directory structure using `WalkDir` to build the folder tree.

```rust
#[tauri::command]
async fn get_folders(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<ModFolder>, String> {
    let state = state.lock().unwrap();
    let game_path = &state.game_path;
    
    if !game_path.exists() {
        return Ok(Vec::new());
    }
    
    let mut folders = Vec::new();
    
    // Get root folder name (e.g., "~mods")
    let root_name = game_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Mods")
        .to_string();
    
    // Count mods directly in root
    let root_mod_count = std::fs::read_dir(game_path)
        .map(|entries| {
            entries.filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|s| s.to_str());
                        ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled")
                    } else {
                        false
                    }
                })
                .count()
        })
        .unwrap_or(0);
    
    // Add root folder
    folders.push(ModFolder {
        id: root_name.clone(),
        name: root_name.clone(),
        enabled: true,
        expanded: true,
        color: None,
        depth: 0,
        parent_id: None,
        is_root: true,
        mod_count: root_mod_count,
    });
    
    // Recursively scan for subdirectories
    for entry in WalkDir::new(game_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        let path = entry.path();
        
        if path.is_dir() {
            // Calculate relative path from game_path to get ID
            let relative_path = path.strip_prefix(game_path)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| "Unknown".to_string());
                
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            // Calculate depth
            let depth = relative_path.split('/').count();
            
            // Calculate parent ID
            let parent_id = if depth > 1 {
                // If depth > 1, parent is the directory containing this one
                // e.g. "A/B" -> parent is "A"
                let parent_rel = std::path::Path::new(&relative_path)
                    .parent()
                    .map(|p| p.to_string_lossy().replace('\\', "/"));
                parent_rel
            } else {
                // If depth is 1, parent is the root folder
                Some(root_name.clone())
            };

            // Count mods in this folder
            let mod_count = std::fs::read_dir(&path)
                .map(|entries| {
                    entries.filter_map(|e| e.ok())
                        .filter(|e| {
                            let p = e.path();
                            if p.is_file() {
                                let ext = p.extension().and_then(|s| s.to_str());
                                ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled")
                            } else {
                                false
                            }
                        })
                        .count()
                })
                .unwrap_or(0);
            
            folders.push(ModFolder {
                id: relative_path, // ID is the relative path (e.g. "Category/Subcategory")
                name,
                enabled: true,
                expanded: true,
                color: None,
                depth,
                parent_id,
                is_root: false,
                mod_count,
            });
        }
    }
    
    Ok(folders)
}
```

### D. Update `get_pak_files`
Update logic to correctly assign mods to nested folders by using relative paths.

```rust
async fn get_pak_files(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<ModEntry>, String> {
    // ... (setup code) ...

    // Scan root ~mods directory and all subdirectories recursively
    for entry in WalkDir::new(&game_path)
        // Remove .max_depth(2) to allow infinite recursion
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        if path.is_dir() { continue; }
        
        let ext = path.extension().and_then(|s| s.to_str());
        
        if ext == Some("pak") || ext == Some("bak_repak") || ext == Some("pak_disabled") {
            let is_enabled = ext == Some("pak");
            
            let root_folder_name = game_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("~mods")
                .to_string();
            
            // Determine folder_id based on relative path
            let folder_id = if let Some(parent) = path.parent() {
                if parent == game_path {
                    // Directly in root
                    Some(root_folder_name)
                } else {
                    // In a subfolder - use relative path from game_path as ID
                    parent.strip_prefix(game_path)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .ok()
                }
            } else {
                Some(root_folder_name)
            };
            
            // ... (rest of the function) ...
        }
    }
    // ...
}
```

## 2. Frontend Changes (`repak-gui/src/App.jsx`)

### Update Folder List Rendering
Modify the folder list mapping to apply indentation based on `folder.depth`.

```jsx
{/* Folder List */}
<div className="folder-list">
  {folders
    .filter(f => {
      // Show if root, or if parent is expanded (logic can be enhanced for full tree collapse)
      // For now, simple list with indentation
      return true; 
    })
    .map(folder => (
      <div
        key={folder.id}
        className={`folder-item ${selectedFolder === folder.id ? 'active' : ''}`}
        onClick={() => handleFolderSelect(folder.id)}
        onContextMenu={(e) => handleFolderContextMenu(e, folder)}
        // Add indentation based on depth
        style={{ paddingLeft: `${12 + (folder.depth || 0) * 12}px` }}
      >
        <div className="folder-icon">
          {folder.is_root ? <FolderIcon /> : <FolderIcon style={{ opacity: 0.7 }} />}
        </div>
        <div className="folder-name">
          {folder.name}
          <span className="folder-count">({folder.mod_count})</span>
        </div>
      </div>
  ))}
</div>
```

## 3. Context Menu Changes (`repak-gui/src/components/ContextMenu.jsx`)

### Update "Move to..." Menu
Show hierarchy in the move menu.

```jsx
<div className="context-menu-item submenu-trigger">
  Move to...
  <div className="submenu">
     <div className="context-menu-item" onClick={() => { onCreateFolder(); onClose(); }}>
       + New Folder...
     </div>
     <div className="context-menu-separator" />
     {folders.filter(f => !f.is_root).map(f => (
       <div 
         key={f.id} 
         className="context-menu-item" 
         // Indent subfolders
         style={{ paddingLeft: `${12 + (f.depth || 0) * 12}px` }} 
         onClick={() => { onMoveTo(f.id); onClose(); }}
       >
         {f.name}
       </div>
     ))}
     <div className="context-menu-separator" />
     <div className="context-menu-item" onClick={() => { onMoveTo(null); onClose(); }}>
       Root
     </div>
  </div>
</div>
```
