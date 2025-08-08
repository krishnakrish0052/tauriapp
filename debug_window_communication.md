# Debug AI Response Window Communication

## Steps to Debug the Window Resize Issue

The issue seems to be that the AI response window is not receiving the streaming data or the JavaScript `updateContent` function is not being called properly. I've added extensive debugging to both the Rust backend and the AI response window JavaScript.

## How to Test:

### 1. Start the Application
```bash
cargo tauri dev
```

### 2. Open DevTools on AI Response Window
1. When the AI response window appears (after starting a streaming response)
2. Right-click on the AI response window and select "Inspect" or "Inspect Element"
3. Go to the Console tab to see detailed logs

### 3. Check the Console Logs

**Look for these logs in the AI response window console:**
- `🎯 AI RESPONSE WINDOW: updateContent called with type: stream`
- `🎯 AI RESPONSE WINDOW: updateContent data: {...}`
- `📝 Stream update received: {...}`
- `🔧 Auto-adjusting window height: {...}`
- `✅ Window auto-resized to XXXpx`

**Look for these logs in the main terminal (Rust logs):**
- `🚀 RUST DEBUG: send_ai_response_data called with message_type: stream`
- `✅ RUST DEBUG: AI response window found, attempting to show and send data`
- `✅ RUST DEBUG: JavaScript evaluation successful`

### 4. Manual Testing with Test Resize Button
1. Click the blue "expand_more" button (Test Resize) in the top-right of the AI response window
2. This should add a lot of test content and trigger a resize
3. Check the console for resize logs

### 5. Expected Log Flow for Streaming

**In Main Terminal (Rust):**
```
🚀 RUST DEBUG: send_ai_response_data called with message_type: "stream"
✅ RUST DEBUG: AI response window found, attempting to show and send data  
✅ RUST DEBUG: AI response window shown successfully
🚀 RUST DEBUG: About to evaluate JavaScript code: if (window.updateContent) { window.updateContent('stream', { text: "..." }); }
✅ RUST DEBUG: JavaScript evaluation successful - AI response data sent successfully
🔧 RESIZE REQUEST: height=XXX, timestamp=...
```

**In AI Response Window Console:**
```
🎯 AI RESPONSE WINDOW: updateContent called with type: stream
🎯 AI RESPONSE WINDOW: updateContent data: {text: "..."}
📝 Stream update received: {fullTextLength: XX, textPreview: "..."}
🔧 Auto-adjusting window height: {contentHeight: XX, finalHeight: XXX, ...}
✅ Window auto-resized to XXXpx
```

## Troubleshooting:

### If you don't see AI response window logs:
- The window may not be created or visible
- Right-click on the AI response window to open DevTools
- If no window appears, check main terminal for window creation errors

### If you see Rust logs but no window logs:
- JavaScript evaluation may be failing
- Check for JavaScript syntax errors in the AI response window console
- The `window.updateContent` function may not be available when called

### If you see window logs but no resize:
- The Tauri resize command may be failing
- Check for resize-related errors in both consoles
- The OS window manager may be preventing resize

### If resize works with Test button but not with streaming:
- The streaming data may not be reaching the window
- Check the format of streaming data being sent
- Timing issues between stream updates and resize calls

## Next Steps:
After running a streaming AI response and collecting these logs, we can determine exactly where the communication breakdown occurs and fix it accordingly.
