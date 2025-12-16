# Repak X Protocol Handler Implementation

## Overview

The Repak X Chrome extension sends mod files to the Repak X desktop app via a custom `repakx://` protocol URL. The Tauri backend now handles this protocol.

## Current Flow (Working ✅)

1. User clicks **"To Repak X"** button on Nexus Mods
2. Extension clicks "Manual download" → navigates to file_id page
3. Extension auto-clicks "Slow download" button
4. Download completes
5. Extension opens: `repakx://install?file=C:\path\to\downloaded\mod.rar`
6. **Tauri app receives the URL and emits `extension-mod-received` event**

---

## Implementation Status

### ✅ Completed - Rust Backend

#### Dependencies Added (`Cargo.toml`)

```toml
tauri-plugin-deep-link = "2"
tauri-plugin-single-instance = "2"  # CRITICAL: Prevents opening new instance
url = "2"
urlencoding = "2"
```

---

### ✅ Single-Instance Support (Implemented)

**Problem:** When `repakx://` URL is opened while the app is already running, Windows launches a **new instance** instead of sending the URL to the existing instance.

**Solution:** Use `tauri-plugin-single-instance` to:
1. Detect if app is already running
2. Forward the URL to the running instance
3. Focus the existing window

#### Required Setup in `main_tauri.rs`

```rust
use tauri_plugin_single_instance::init as single_instance_init;

fn main() {
    tauri::Builder::default()
        .plugin(single_instance_init(|app, args, _cwd| {
            // This closure is called when a second instance is launched
            // `args` contains command line arguments including the deep-link URL
            
            // Focus the main window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
            
            // Check if args contains a repakx:// URL
            for arg in args.iter() {
                if arg.starts_with("repakx://") {
                    handle_deep_link_url(arg, app);
                }
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        // ... rest of plugins
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

#### How It Works

1. User has app running
2. Browser extension opens `repakx://install?file=...`
3. Windows tries to launch the exe with the URL as an argument
4. `single-instance` plugin detects existing instance
5. Plugin forwards `args` (including the URL) to the running instance
6. Closure receives the URL and calls `handle_deep_link_url()`
7. Existing app receives the event and shows the overlay

#### Configuration (`tauri.conf.json`)

```json
{
  "plugins": {
    "deep-link": {
      "desktop": {
        "schemes": ["repakx"]
      }
    }
  }
}
```

#### Capabilities (`capabilities/default.json`)

```json
{
  "permissions": [
    "deep-link:default"
  ]
}
```

#### Protocol Handler (`main_tauri.rs`)

The `handle_deep_link_url()` function:
- Parses `repakx://install?file=...` URLs
- URL-decodes the file path
- Validates the file exists
- Emits `extension-mod-received` event with the file path
- Emits `extension-mod-error` if file not found

```rust
fn handle_deep_link_url(url: &str, app_handle: &tauri::AppHandle) {
    if let Ok(parsed) = url::Url::parse(url) {
        if parsed.scheme() == "repakx" && parsed.host_str() == Some("install") {
            if let Some(file_path) = parsed.query_pairs()
                .find(|(key, _)| key == "file")
                .map(|(_, value)| value.to_string()) 
            {
                let decoded_path = urlencoding::decode(&file_path)
                    .unwrap_or(file_path.clone().into())
                    .to_string();
                
                if Path::new(&decoded_path).exists() {
                    app_handle.emit("extension-mod-received", &decoded_path).ok();
                } else {
                    app_handle.emit("extension-mod-error", format!("File not found: {}", decoded_path)).ok();
                }
            }
        }
    }
}
```

---

### ✅ Completed - Frontend Integration

Frontend integration is implemented in `App.jsx` with:
- Event listeners for `extension-mod-received` and `extension-mod-error`
- `ExtensionModOverlay` component for folder selection
- `handleExtensionModInstall()` function using `quick_organize` command

See `src/components/ExtensionModOverlay.jsx` for the overlay implementation.

---

## URL Format Reference

| Field         | Value                              |
| ------------- | ---------------------------------- |
| **Protocol**  | `repakx://`                        |
| **Action**    | `install`                          |
| **Parameter** | `file` (URL-encoded absolute path) |

**Example:**
```
repakx://install?file=C%3A%5CUsers%5Cfranc%5CDownloads%5Cmod.rar
```
**Decoded:** `C:\Users\franc\Downloads\mod.rar`

---

## Testing Steps

1. Build and install the app (registers protocol automatically)
2. Open browser console and run:
   ```javascript
   window.location.href = 'repakx://install?file=C%3A%5Ctest%5Cmod.zip';
   ```
3. App should launch/focus and log the received URL
4. Check app logs for "Received mod file from extension: ..."

## Windows Registry

### Automatic Registration (Installer)

The `deep-link` plugin automatically registers the protocol on Windows via NSIS/MSI installer.

### Manual Registration (Development)

Use the PowerShell script at `scripts/register-protocol.ps1`:

```powershell
# Register for current user (no admin required)
.\scripts\register-protocol.ps1 -CurrentUser

# Unregister
.\scripts\register-protocol.ps1 -Unregister -CurrentUser
```

Manual registry entry format:
```
HKEY_CURRENT_USER\Software\Classes\repakx
    (Default) = URL:Repak X Protocol
    URL Protocol = ""
    
HKEY_CURRENT_USER\Software\Classes\repakx\shell\open\command
    (Default) = "C:\path\to\Repak Gui Revamped.exe" "%1"
```

---

## ✅ Portable App Support - Self-Registration (Implemented)

For portable apps (no installer), the protocol must be registered on first launch. This ensures the `repakx://` protocol works without requiring users to run a separate script.

### Required: Add `winreg` Dependency

```toml
# Cargo.toml
[target.'cfg(windows)'.dependencies]
winreg = "0.52"
```

### Proposed Implementation

Add this function to register the protocol on app startup:

```rust
/// Registers the repakx:// protocol handler in Windows Registry (HKCU)
/// This enables the browser extension to communicate with the app.
/// Safe to call on every startup - it will just update the path if needed.
#[cfg(target_os = "windows")]
fn register_protocol_handler() -> Result<(), Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    // Create or open the protocol key
    let (protocol_key, _) = hkcu.create_subkey(r"Software\Classes\repakx")?;
    protocol_key.set_value("", &"URL:Repak X Protocol")?;
    protocol_key.set_value("URL Protocol", &"")?;
    
    // Create the DefaultIcon key (optional, for nice icon in Windows)
    let (icon_key, _) = hkcu.create_subkey(r"Software\Classes\repakx\DefaultIcon")?;
    icon_key.set_value("", &format!("\"{}\",0", exe_path_str))?;
    
    // Create the shell\open\command key
    let (command_key, _) = hkcu.create_subkey(r"Software\Classes\repakx\shell\open\command")?;
    let command = format!("\"{}\" \"%1\"", exe_path_str);
    command_key.set_value("", &command)?;
    
    log::info!("Registered repakx:// protocol handler for: {}", exe_path_str);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn register_protocol_handler() -> Result<(), Box<dyn std::error::Error>> {
    // No-op on non-Windows platforms
    Ok(())
}
```

### Where to Call It

Call this early in `main()` or during Tauri setup:

```rust
fn main() {
    // Register protocol handler for portable app support
    if let Err(e) = register_protocol_handler() {
        eprintln!("Failed to register protocol handler: {}", e);
        // Non-fatal - app can still work, just browser extension won't
    }
    
    tauri::Builder::default()
        // ... rest of setup
}
```

### Benefits of This Approach

1. **No admin required** - Uses `HKEY_CURRENT_USER`, not `HKEY_LOCAL_MACHINE`
2. **Idempotent** - Safe to call every startup, just updates the path
3. **Portable-friendly** - Works when exe is moved to a different location
4. **Silent** - No user interaction required
5. **Self-healing** - If user moves the app, next launch fixes the registry

### Notes

- The registry update is instant and doesn't require a restart
- Only affects the current Windows user
- If app is run from different locations, the most recent one "wins"

---

## Summary

| Component                      | Status              |
| ------------------------------ | ------------------- |
| Chrome Extension               | ✅ Complete          |
| Button Injection               | ✅ Working           |
| Auto-click Slow Download       | ✅ Working           |
| Download Detection             | ✅ Working           |
| Protocol URL Generation        | ✅ Working           |
| **Protocol Handler in Tauri**  | ✅ **Implemented**   |
| **Frontend Integration**       | ✅ **Implemented**   |
| **Single-Instance Support**    | ✅ **Implemented**   |
| **Portable Self-Registration** | ✅ **Implemented**   |

### ✅ All Backend Work Complete

The protocol handler system is fully implemented:

1. **Single-Instance Support** - Uses `tauri-plugin-single-instance` to forward deep-link URLs to the running instance
2. **Portable Self-Registration** - Uses `winreg` crate to register the `repakx://` protocol on every app startup (HKCU, no admin required)
3. **Deep-Link Handling** - Parses URLs, validates files, and emits events to frontend

### Testing

1. Run the app (registers protocol automatically)
2. Open browser and navigate to: `repakx://install?file=C%3A%5Ctest%5Cmod.zip`
3. App should focus and show the extension mod overlay


