# Folder Refresh Fix - Required Frontend Changes

## Problem
When creating a new physical subfolder in the mod folder directory, the GUI does not update and display the new folder.

## Root Causes Identified

### 1. File Watcher Not Initialized
**Location:** `App.jsx` - `loadInitialData()` function (around line 312)

**Issue:** The `start_file_watcher` Tauri command is never called from the frontend.

**Fix Required:**
```jsx
const loadInitialData = async () => {
  try {
    const path = await invoke('get_game_path')
    setGamePath(path)
    
    const ver = await invoke('get_app_version')
    setVersion(ver)
    
    await loadMods()
    await loadFolders()
    await checkGame()
    
    // ADD THIS: Start the file watcher
    await invoke('start_file_watcher')
  } catch (error) {
    console.error('Failed to load initial data:', error)
  }
}
```

### 2. Missing Event Listener
**Location:** `App.jsx` - First `useEffect` hook (around line 211)

**Issue:** The frontend doesn't listen for the `mods_dir_changed` event that the backend emits when directory changes occur.

**Fix Required:**
Add this listener inside the first `useEffect` hook (after line 233):

```jsx
// Listen for directory changes (new folders, deleted folders, etc.)
const unlistenDirChanged = listen('mods_dir_changed', () => {
  console.log('Directory changed, reloading mods and folders...')
  loadMods()
  loadFolders()
})
```

And add cleanup in the return statement (after line 284):

```jsx
return () => {
  // Cleanup listeners
  unlisten.then(f => f())
  unlistenComplete.then(f => f())
  unlistenCharUpdate.then(f => f())
  unlistenDragDrop.then(f => f())
  unlistenFileDrop.then(f => f())
  unlistenLogs.then(f => f())
  unlistenDirChanged.then(f => f())  // ADD THIS LINE
  document.removeEventListener('dragover', preventDefault)
  document.removeEventListener('drop', preventDefault)
}
```

## Backend Status
✅ **No backend changes needed** - The backend implementation is correct:
- File watcher properly configured in `main_tauri.rs` (line 149-200)
- Uses recursive watching mode (`RecursiveMode::Recursive` on line 187)
- Emits `mods_dir_changed` events for Create, Remove, and Modify operations (line 171)
- `EventKind::Create(_)` includes both file AND directory creation events
- `get_folders` function correctly scans directories using `std::fs::read_dir` (line 1053)

**Note:** The comment on line 169 says "on files" but the code actually handles both files and directories correctly. The `EventKind::Create(_)` enum variant matches all creation events regardless of whether they're files or directories.

## Testing Steps
After implementing the frontend changes:

1. Start the application
2. Navigate to the mod folder directory in File Explorer
3. Create a new subfolder (e.g., "TestFolder")
4. Observe that the GUI automatically updates and shows the new folder
5. Delete the folder and verify it disappears from the GUI
6. Rename a folder and verify the change is reflected

## Additional Notes
- The file watcher uses `RecursiveMode::Recursive`, so it monitors all subdirectories
- Events are emitted for Create, Remove, and Modify operations
- The watcher is set up to watch the `game_path` which should be the `~mods` directory
- Consider adding debouncing if multiple rapid changes cause performance issues

## Optional Enhancement: Manual Refresh Button
Consider adding a manual refresh button as a fallback in case the file watcher fails or for user preference.

**Location:** `App.jsx` - In the folder sidebar header (around line 873)

**Suggested Addition:**
```jsx
<div className="sidebar-header">
  <h3>Folders</h3>
  <div style={{ display: 'flex', gap: '4px' }}>
    <button 
      onClick={async () => {
        await loadFolders()
        await loadMods()
        setStatus('Folders refreshed')
      }} 
      className="btn-icon" 
      title="Refresh Folders"
    >
      <RefreshIcon fontSize="small" />
    </button>
    <button onClick={handleCreateFolder} className="btn-icon" title="New Folder">
      <CreateNewFolderIcon fontSize="small" />
    </button>
  </div>
</div>
```

This provides users with a manual way to refresh the folder list if needed.
