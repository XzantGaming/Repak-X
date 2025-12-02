# Quick P2P Testing Guide

## The Problem You're Seeing

The error "Port 5173 is already in use" happens because you can't run `cargo tauri dev` twice from the same directory - both instances try to use the same development server port.

## Solution: Build Once, Run Twice

### Option 1: Use the Build Script (Recommended)

1. **Close both terminal windows** showing the error
2. **Double-click** `BUILD_AND_TEST.bat`
3. Wait for the build to complete (~2-5 minutes first time)
4. Two app windows will open automatically
5. Test the P2P transfer!

### Option 2: Manual Build & Run

**Step 1: Build the app**
```bash
cd repak-gui
cargo build --release
```

**Step 2: Run two instances**

Open the executable twice:
- `repak-gui\target\release\repak-gui.exe` (Instance 1 - SENDER)
- `repak-gui\target\release\repak-gui.exe` (Instance 2 - RECEIVER)

Or use these shortcuts:
- Double-click `START_SENDER.bat`
- Double-click `START_RECEIVER.bat`

## Testing Steps

### In Instance 1 (SENDER):
1. Open P2P Sharing panel
2. Click "Share Mods"
3. **Copy the connection string** (the long text)

### In Instance 2 (RECEIVER):
1. Open P2P Sharing panel
2. Switch to "Receive Mods" tab
3. **Paste the connection string**
4. Click "Start Receiving"

### Watch for Success:
- Files should start appearing in receiver's game folder
- Check logs: `repak-gui\target\release\Logs\repak-gui.log`

## Why This Works

- **Dev mode** (`cargo tauri dev`): Can only run one instance
- **Release build** (`cargo build --release`): Can run multiple instances
- Each instance gets its own peer ID
- They can connect to each other via localhost

## Troubleshooting

### If build fails:
```bash
# Clean and rebuild
cd repak-gui
cargo clean
cargo build --release
```

### If instances don't connect:
- Make sure both are fully loaded
- Check that connection string was copied completely
- Look at the log file for errors

### If transfer doesn't start:
- Verify sender has mods selected
- Check that both instances show "Peer connected" in logs
- Try closing and reopening both instances

## Expected Timeline

1. **Build**: 2-5 minutes (first time), 30 seconds (subsequent)
2. **App startup**: 5-10 seconds each
3. **Connection**: 1-3 seconds after clicking "Receive"
4. **Transfer**: Depends on file size (very fast on localhost)

## Next Steps

Once this works locally:
1. Build the app: `cargo tauri build`
2. Share the installer with friends
3. Test over the internet!
