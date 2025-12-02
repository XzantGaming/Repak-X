# Local P2P Testing Guide

## Quick Start

### Option 1: Use the Batch Script (Easiest)
1. Double-click `TEST_P2P_LOCALLY.bat`
2. Two app instances will open automatically
3. Follow the on-screen instructions

### Option 2: Manual Setup

#### Terminal 1 (SENDER):
```bash
cd repak-gui
cargo tauri dev
```

#### Terminal 2 (RECEIVER):
Open a NEW terminal window and run:
```bash
cd repak-gui
cargo tauri dev
```

## Testing Steps

### 1. Setup Sender (Instance 1)
1. Wait for the app to fully load
2. Make sure you have some mods in your game folder
3. Open the P2P Sharing panel
4. Click "Share Mods"
5. Enter a name (e.g., "Test Transfer")
6. **Copy the connection string** (the long base64 text in the box)

### 2. Setup Receiver (Instance 2)
1. Wait for the app to fully load
2. Open the P2P Sharing panel
3. Switch to "Receive Mods" tab
4. **Paste the connection string** from Instance 1
5. Click "Start Receiving"

### 3. Watch the Transfer
Look at the **terminal logs** (not the app UI) for both instances:

**Sender Terminal Should Show:**
```
[INFO] Peer connected: 12D3KooW...
[INFO] Received file transfer request from ...
[INFO] Sending pack info to ...: 4 files
[INFO] Sending chunk of VergilMesh_9999999_P.pak: offset=0, size=1048576
[INFO] Sending chunk of VergilMesh_9999999_P.pak: offset=1048576, size=1048576
...
```

**Receiver Terminal Should Show:**
```
[INFO] Dialing peer 12D3KooW... at ...
[INFO] Peer connected: 12D3KooW...
[INFO] Requesting pack info from newly connected peer
[INFO] Received pack info from ...: 4 files, total size: 12345678 bytes
[INFO] Requesting first chunk of VergilMesh_9999999_P.pak
[INFO] Received chunk of VergilMesh_9999999_P.pak: offset=0, size=1048576
[INFO] Requesting next chunk of VergilMesh_9999999_P.pak at offset 1048576
...
[INFO] File complete: VergilMesh_9999999_P.pak
```

## What to Look For

### ✅ Success Indicators:
- "Peer connected" appears in both terminals
- "Sending chunk" appears in sender
- "Received chunk" appears in receiver
- Files appear in the receiver's game folder
- No error messages

### ❌ Problem Indicators:
- "Failed to dial peer" - Connection issue
- "Hash mismatch" - Data corruption
- "File not found" - Sender can't find the file
- No "Peer connected" - Network issue

## Troubleshooting

### Issue: "Failed to dial peer"
**Solution:** Both instances are on the same machine, so they should connect via localhost. Check if:
- Both instances are running
- No firewall blocking local connections
- Connection string was copied correctly

### Issue: No transfer starts
**Solution:** 
- Check if "Peer connected" appears in logs
- Verify connection string is complete (not truncated)
- Make sure sender has mods selected

### Issue: Transfer stops mid-way
**Solution:**
- Check terminal for error messages
- Verify disk space on receiver
- Check file permissions

## Expected Behavior

### Connection:
- **Time:** 1-3 seconds after clicking "Start Receiving"
- **Log:** "Peer connected: 12D3KooW..."

### Pack Info Exchange:
- **Time:** Immediately after connection
- **Log:** "Received pack info from ...: X files"

### File Transfer:
- **Speed:** Depends on file size, typically very fast on localhost
- **Chunks:** 1MB per chunk
- **Log:** Continuous "Sending/Received chunk" messages

### Completion:
- **Log:** "File complete: [filename]"
- **Result:** Files appear in receiver's game folder

## Advanced Testing

### Test Different Scenarios:

1. **Small Files:** Transfer a single small mod (<10MB)
2. **Large Files:** Transfer a large mod (>100MB)
3. **Multiple Files:** Transfer a pack with 4+ mods
4. **Interruption:** Stop receiver mid-transfer (tests error handling)
5. **Reconnection:** Stop and restart receiver (tests if it can resume)

### Monitor Performance:

Watch the logs for:
- Transfer speed (chunks per second)
- Memory usage (Task Manager)
- CPU usage during transfer
- Any warnings or errors

## Cleanup

After testing:
1. Stop both instances (Ctrl+C in terminals)
2. Check receiver's game folder for transferred files
3. Delete test files if needed
4. Check logs for any errors: `target/debug/Logs/repak-gui.log`

## Notes

- **Both instances use the same peer ID generation**, so they'll have different IDs
- **Localhost transfers are FAST** - much faster than internet transfers
- **No IP addresses are exposed** - even locally, it uses Peer IDs
- **Files are verified** - SHA256 hash checked for each chunk

## Next Steps

Once local testing works:
1. Test between two different computers on same network
2. Test over the internet (different networks)
3. Test with NAT traversal (behind routers)
4. Test with relay servers (strict NATs)
