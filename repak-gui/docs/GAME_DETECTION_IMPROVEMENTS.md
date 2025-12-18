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

## Frontend Changes Required (App.jsx)

**Issue**: `checkGame()` is only called once during app initialization. There is no periodic polling, so the game status never updates until the app is reloaded.

### Required Fix: Add Periodic Polling

Add a `useEffect` hook to poll the game status every 3 seconds:

```javascript
// Add this useEffect after the initialization useEffect (around line 440)
// Periodic game status polling
useEffect(() => {
  // Initial check
  checkGame();
  
  // Poll every 3 seconds
  const interval = setInterval(() => {
    checkGame();
  }, 3000);
  
  return () => clearInterval(interval);
}, []);
```

### Alternative: More Aggressive Polling When Install Panel is Open

If you want faster updates when the install panel is visible:

```javascript
// Poll game status - faster when install panel is open
useEffect(() => {
  const pollInterval = showInstallPanel ? 1000 : 5000; // 1s when installing, 5s otherwise
  
  const interval = setInterval(() => {
    checkGame();
  }, pollInterval);
  
  return () => clearInterval(interval);
}, [showInstallPanel]);
```

### Location in App.jsx

Add the polling `useEffect` after line ~440 (after the initialization effect that calls `checkGame()` once).

The `checkGame` function already exists at line 571:
```javascript
const checkGame = async () => {
  try {
    const running = await invoke('check_game_running')
    setGameRunning(running)
  } catch (error) {
    console.error('Failed to check game status:', error)
  }
}
```

## Testing

To verify the fix works:
1. Start Repak GUI
2. Start Marvel Rivals
3. The "Game Running" indicator should appear within 3-5 seconds
4. Close Marvel Rivals
5. The indicator should disappear within 3-5 seconds

The detection should now be consistent regardless of how the game was launched (Steam, direct exe, etc.).
