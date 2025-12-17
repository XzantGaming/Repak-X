# Game Running Detection - Improvements Made

## Summary

Fixed unreliable game process detection that was causing inconsistent behavior when checking if Marvel Rivals is running.

## Issues Identified

### 1. Missing Process Executable Info
**Problem**: `ProcessRefreshKind::new()` creates an empty refresh configuration that doesn't fetch the executable path.

**Before**:
```rust
let s = System::new_with_specifics(
    RefreshKind::new().with_processes(ProcessRefreshKind::new())
);
```

**After**:
```rust
let s = System::new_with_specifics(
    RefreshKind::new().with_processes(
        ProcessRefreshKind::new()
            .with_exe(UpdateKind::Always)
    )
);
```

### 2. Using `name()` Instead of `exe()`
**Problem**: `process.name()` on Windows can be unreliable - it may return truncated names or different formats depending on how the process was started.

**Solution**: Use `process.exe()` as the primary detection method, which returns the full executable path, then extract the filename from it. Fall back to `name()` as a secondary check.

### 3. Duplicated Detection Logic
**Problem**: Game detection logic was duplicated in multiple places (`check_game_running` and `monitor_game_for_crashes`), making maintenance difficult.

**Solution**: Created a shared `is_game_process_running()` function that both commands use.

## Changes Made

### `main_tauri.rs`
- Added new `is_game_process_running()` function with reliable detection
- Updated `check_game_running` command to use the shared function
- Updated `monitor_game_for_crashes` to use the shared function instead of duplicated logic

### `main.rs` (egui version)
- Updated `is_game_running()` method with the same reliable detection logic

## Detection Algorithm

```rust
fn is_game_process_running() -> bool {
    // 1. Create System with exe path info enabled
    let s = System::new_with_specifics(
        RefreshKind::new().with_processes(
            ProcessRefreshKind::new()
                .with_exe(UpdateKind::Always)
        )
    );
    
    let game_exe_name = "marvel-win64-shipping.exe";
    
    for (_pid, process) in s.processes() {
        // Primary: Check exe() path (most reliable)
        if let Some(exe_path) = process.exe() {
            if let Some(file_name) = exe_path.file_name() {
                if file_name.to_string_lossy().to_lowercase() == game_exe_name {
                    return true;
                }
            }
        }
        
        // Fallback: Check process name() directly
        let process_name = process.name().to_string_lossy().to_lowercase();
        if process_name == game_exe_name {
            return true;
        }
    }
    
    false
}
```

## Frontend Recommendations (Optional)

The current frontend implementation in `App.jsx` is adequate, but consider these optional improvements:

### 1. Increase Polling Frequency During Critical Operations
Currently `checkGame` is called periodically. Consider calling it more frequently (every 1-2 seconds) when the install panel is open.

```javascript
// In useEffect or when install panel opens
useEffect(() => {
  if (showInstallPanel) {
    const interval = setInterval(checkGame, 1000); // 1 second during install
    return () => clearInterval(interval);
  }
}, [showInstallPanel]);
```

### 2. Add Visual Indicator for Game Status
Consider adding a small indicator in the UI showing the current game status (running/not running) so users know the detection is working.

### 3. Debounce Game State Changes
To avoid flickering if the game process briefly disappears during loading screens:

```javascript
const [gameRunningDebounced, setGameRunningDebounced] = useState(false);

useEffect(() => {
  const timeout = setTimeout(() => {
    setGameRunningDebounced(gameRunning);
  }, 500); // 500ms debounce
  return () => clearTimeout(timeout);
}, [gameRunning]);
```

## Testing

To verify the fix works:
1. Start Marvel Rivals
2. Open Repak GUI
3. The game running warning should appear when trying to install mods
4. Close Marvel Rivals
5. The warning should no longer appear

The detection should now be consistent regardless of how the game was launched (Steam, direct exe, etc.).
