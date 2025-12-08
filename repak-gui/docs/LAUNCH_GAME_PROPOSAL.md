# Launch Game Feature Proposal

## Overview
The user requested a "Play" button to launch Marvel Rivals directly from the Repak GUI.
Since this requires executing a system command (opening a `steam://` URL), it involves backend configuration changes in Tauri that are outside the scope of a purely frontend update.

## Implementation Status

### ✅ Backend Configuration - COMPLETED
The `capabilities/default.json` has been updated with the required permission:
- Added `"shell:allow-open"` to the permissions array
- This enables the app to open external URLs via the shell plugin

### ✅ Rust Backend Command - COMPLETED
A new `launch_game` Tauri command has been implemented in `main_tauri.rs`:
- **Toggleable launcher skip** - Only skips launcher when launched through our app
- **Preserves user settings** - Steam manual launches use user's configured settings
- Cross-platform support (Windows, macOS, Linux)
- Uses `launch_record` file modification (temporary, auto-restores)
- Proper error handling and logging
- Registered in the Tauri invoke_handler

### ⏳ Frontend Implementation - PENDING

You have **TWO OPTIONS** to implement the frontend:

## Option 1: Use Rust Backend Command (Recommended)

This approach uses the new `launch_game` Rust command. **Only requires minimal JSX changes.**

### Required Changes:

**In `App.jsx` line ~1210, replace the onClick handler:**

```javascript
onClick={async () => {
  try {
    await invoke('launch_game')
    setGameRunning(true)
  } catch (error) {
    console.error('Failed to launch game:', error)
    alert(error)
  }
}}
```

**That's it!** No import changes needed since `invoke` is already imported.

---

## Option 2: Use Tauri Shell Plugin (Original Approach)

The frontend changes are ready to be applied when needed. Here's what needs to be done:

## Required Frontend Changes (`App.jsx`)

### 1. Update Imports (Line 3)

**Current:**
```javascript
import { open } from '@tauri-apps/plugin-dialog'
```

**Change to:**
```javascript
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { open as openUrl } from '@tauri-apps/plugin-shell'
```

*Note: We rename `open` from dialog to `openDialog` to avoid naming conflicts, since both plugins export an `open` function.*

### 2. Update All Dialog References

Find and replace all instances of `open(` with `openDialog(` throughout the file where it's used for file/folder dialogs.

**Examples to update:**
- Line ~145: `const result = await open({...})`  → `const result = await openDialog({...})`
- Line ~160: `const result = await open({...})`  → `const result = await openDialog({...})`
- Line ~175: `const result = await open({...})`  → `const result = await openDialog({...})`
- Any other dialog open calls

### 3. Update Play Button (Lines 1198-1213)

**Current:**
```javascript
<button 
  className="btn-settings"
  title="Launch Marvel Rivals (Coming Soon)"
  style={{ 
    background: 'rgba(74, 158, 255, 0.1)', 
    color: '#4a9eff', 
    border: '1px solid rgba(74, 158, 255, 0.3)',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    fontWeight: 600
  }}
  onClick={() => alert('Launch Game feature requires backend configuration. See docs/LAUNCH_GAME_PROPOSAL.md')}
>
  <PlayArrowIcon /> Play
</button>
```

**Change to:**
```javascript
<button 
  className="btn-settings"
  title="Launch Marvel Rivals"
  style={{ 
    background: 'rgba(74, 158, 255, 0.1)', 
    color: '#4a9eff', 
    border: '1px solid rgba(74, 158, 255, 0.3)',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    fontWeight: 600
  }}
  onClick={async () => {
    try {
      await openUrl('steam://run/2767030')
      setGameRunning(true)
    } catch (error) {
      console.error('Failed to launch game:', error)
      alert('Failed to launch game. Please ensure Steam is installed.')
    }
  }}
>
  <PlayArrowIcon /> Play
</button>
```

## Technical Details

### Steam App ID
- Marvel Rivals Steam App ID: `2767030`
- Steam protocol URL: `steam://run/2767030`

### How It Works
1. User clicks the "Play" button
2. `openUrl()` or `launch_game()` calls the system's default handler for `steam://` URLs
3. Steam client intercepts the URL and launches the game
4. The "Game Running" checkbox is automatically enabled
5. If Steam is not installed or the URL fails, an error alert is shown

### How Launcher Skip Works (Implemented!)
Our implementation **automatically skips the launcher** when you launch through our app, while preserving your Steam settings for manual launches.

**Technical Implementation:**
1. **Backup** - Reads current `launch_record` value (usually "6")
2. **Modify** - Temporarily sets it to "0" (skip launcher)
3. **Launch** - Starts game via Steam protocol
4. **Restore** - Returns `launch_record` to original value after 500ms

**Result:**
- ✅ **Our app** → Game launches directly (no launcher screen)
- ✅ **Steam manual launch** → Uses your configured Steam launch options
- ✅ **Non-persistent** → Doesn't permanently change your settings
- ✅ **Safe** → Auto-restores even if something goes wrong

**Optional: Steam Launch Options for Manual Launches**
If you want to skip the launcher even when launching manually through Steam:

1. Right-click Marvel Rivals in Steam → Properties
2. In the "Launch Options" field, add:
   ```
   cmd /min /C "set __COMPAT_LAYER=RUNASINVOKER && start "" %command%"
   ```
3. This will also run without Administrator privileges (better security)

## Security Implications
- ✅ `shell:allow-open` permission is now enabled in capabilities
- ✅ The shell plugin will open URLs using the system's default handler
- ✅ Steam protocol URLs are safe and handled by the Steam client
- ✅ No arbitrary command execution - only URL opening

## Testing Checklist
- [ ] Click "Play" button with Steam installed → Game launches
- [ ] Click "Play" button without Steam → Error message shown
- [ ] "Game Running" checkbox auto-enables after launch
- [ ] All file/folder dialogs still work (after renaming `open` to `openDialog`)

## Notes
- The `@tauri-apps/plugin-shell` package is already installed in `package.json`
- Backend configuration is complete and ready to use
- Frontend changes are minimal and low-risk
- Feature will work on Windows, macOS, and Linux (wherever Steam is installed)

---

## Summary

### What's Been Done (Rust Backend):
✅ Added `shell:allow-open` permission to `capabilities/default.json`  
✅ Implemented `launch_game()` command in `main_tauri.rs` (lines 1739-1820)  
✅ **Toggleable launcher skip** - Temporary `launch_record` modification  
✅ **Preserves user settings** - Auto-restores original value after launch  
✅ Registered command in invoke_handler (line 2787)  
✅ Cross-platform support (Windows/macOS/Linux)  
✅ Proper error handling and logging  
✅ **Build successful** - Ready to use

### What's Left (Frontend - Your Choice):
**Option 1 (Recommended):** Change 1 line in `App.jsx` to call `invoke('launch_game')`  
**Option 2 (Alternative):** Import shell plugin and use `openUrl()` directly (requires more changes)

### Recommendation:
**Use Option 1** - It's simpler, requires minimal JSX changes, and leverages the Rust backend which:
- Automatically skips the launcher when launched through our app
- Preserves Steam settings for manual launches
- Provides better error handling and logging
