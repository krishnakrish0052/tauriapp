# Desktop App Update Instructions - Temporary Token Authentication

## Overview
Your desktop app needs to be updated to support the new secure temporary token authentication system. This will enable automatic login when launched from the web app.

## Changes Required

### 1. Replace Auto Launch Manager
**File**: `dist/auto-launch-manager.js`

Replace the existing file with the new enhanced version:

```bash
# Backup the original
cp dist/auto-launch-manager.js dist/auto-launch-manager-backup.js

# Copy the new version
cp dist/auto-launch-manager-updated.js dist/auto-launch-manager.js
```

### 2. Update HTML File
**File**: `dist/index.html`

Ensure the auto-launch-manager.js is loaded:

```html
<!-- Make sure this script is included -->
<script src="auto-launch-manager.js"></script>
```

### 3. Test the Integration

#### Test Steps:
1. **Build the desktop app**: `npm run build` or `tauri build`
2. **Register protocol handler**: Install and run the desktop app once
3. **Test from web app**: 
   - Go to: `http://localhost:3000/session/38339bd4-94ec-49b8-ba87-b909fe334efd`
   - Click "Launch Desktop App"
   - Desktop should open and automatically authenticate

## How It Works Now

### Before (Manual)
1. User clicks launch button
2. Desktop app opens
3. **User must manually enter session ID and authenticate**

### After (Automatic)
1. User clicks launch button
2. Web app generates temporary token (10-minute expiry)
3. Desktop app launches with: `mockmate://session/{id}?temp_token={token}&auto_fill=true&auto_connect=true`
4. **Desktop app automatically authenticates and starts session**

## New Protocol URL Format

```
mockmate://session/38339bd4-94ec-49b8-ba87-b909fe334efd?temp_token=uuid-token&auto_fill=true&auto_connect=true&source=webapp&timestamp=1692709525000
```

## Backend API Called

The desktop app now calls:

```http
POST http://localhost:5000/api/sessions/{sessionId}/connect-with-temp-token
Content-Type: application/json

{
  "tempToken": "temporary-uuid-token",
  "desktop_version": "1.0.0", 
  "platform": "windows"
}
```

Response includes:
- ✅ User authentication
- ✅ Session activation  
- ✅ Credit deduction
- ✅ Full session data

## Console Output

When working correctly, you should see:

```
🚀 Auto Launch Manager initialized (Enhanced)
🔍 Checking URL for session ID and temp token...
✅ Found session ID from URL: 38339bd4-94ec-49b8-ba87-b909fe334efd
🔑 Found temporary token from URL: 12345678...
🎯 AUTO LAUNCH SUMMARY (Enhanced):
========================================
Session ID: 38339bd4-94ec-49b8-ba87-b909fe334efd
Temp Token: 12345678...
Auto-fill: ✅ Enabled
Auto-connect: ✅ Enabled
Authentication: 🔐 Secure (Temp Token)
========================================
🔐 Auto-connecting with temporary token for session: 38339bd4-...
📡 Calling temp token authentication API...
✅ Temp token authentication successful
✅ Successfully authenticated and activated session with temp token
```

## Error Handling

If temp token authentication fails, the app will:
1. Show error notification
2. Fall back to manual connection UI
3. Auto-fill the session ID for user convenience

## Security Features

- **10-minute token expiry**: Tokens automatically expire
- **Single-use tokens**: Tokens are deleted after use
- **Session-specific**: Each token tied to specific session
- **Secure logging**: Tokens are masked in console output

## Troubleshooting

### Issue: Desktop app opens but doesn't auto-login

**Check:**
1. Console for error messages
2. Backend server is running on `http://localhost:5000`
3. Session exists and is in 'created' status
4. Token hasn't expired (10-minute limit)

**Debug:**
1. Open browser dev tools on session page
2. Look for "Generated temporary desktop token" message
3. Check if protocol URL includes `temp_token` parameter

### Issue: "Authentication failed" error

**Possible causes:**
1. Backend server not running
2. Session already activated by another method
3. Temporary token expired
4. Network connectivity issues

**Solution:**
- Restart backend server
- Create a new session
- Check browser console for errors

## Testing Checklist

- [ ] Desktop app builds without errors
- [ ] Protocol handler registered (mockmate://)
- [ ] Web app generates temp tokens successfully
- [ ] Desktop app launches from web app
- [ ] Desktop app auto-fills session ID
- [ ] Desktop app auto-connects and activates session
- [ ] Session shows as "active" in web app
- [ ] Credits are deducted properly
- [ ] Error handling works (expired tokens, etc.)

## Next Steps

After successful testing:
1. **Commit the changes** to your desktop app repository
2. **Build a release version** if needed
3. **Update any documentation** about the desktop app launch process
4. **Inform users** that desktop app now supports one-click launch

The enhanced auto-authentication significantly improves user experience by eliminating manual login steps while maintaining security through temporary tokens.
