# 🎯 Current Window Accessibility Reader

This module provides text extraction from the **currently active window** to detect interview questions in real-time.

## 🔧 How It Works

The system monitors the **currently focused window** (the one the user is actively looking at) and extracts text from it when it matches target applications like:

- Microsoft Teams
- Zoom 
- Google Chrome
- Mozilla Firefox
- Discord
- Slack
- And more...

## 🚀 Available Commands

### 1. Read from Current Window (Main Command)
```javascript
// Get text from the currently active window
const result = await invoke('read_text_from_current_window');

if (result) {
    console.log(`Found text from ${result.source_app}: ${result.text}`);
    console.log(`Is question: ${result.is_potential_question}`);
} else {
    console.log('No relevant text found in current window');
}
```

### 2. Real-time Monitoring
```javascript
// Start monitoring the current window continuously
await invoke('start_realtime_monitoring');

// Listen for questions detected in real-time
window.__TAURI__.event.listen('accessibility-question-detected', (event) => {
    const question = event.payload;
    console.log(`Question detected: ${question.text}`);
    console.log(`From: ${question.source_app} - ${question.window_title}`);
    
    // Display in MockMate UI and generate response
    displayQuestion(question.text);
});

// Stop monitoring
await invoke('stop_realtime_monitoring');
```

### 3. Configuration
```javascript
// Update which apps to monitor
await invoke('update_accessibility_config', {
    target_apps: ['Microsoft Teams', 'Zoom', 'Google Chrome'],
    monitoring_interval_ms: 1000, // Check every second
    min_question_length: 10       // Minimum text length
});
```

## 💡 Perfect for Interview Scenarios

**Scenario**: You're in a Teams interview and MockMate is running

1. **Interviewer asks**: "What is your experience with React?"
2. **Current Window**: Microsoft Teams (where the question appears)
3. **MockMate detects**: Question in the currently focused Teams window
4. **Result**: MockMate captures the question and can generate a response

## ⚙️ Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `target_apps` | Teams, Zoom, Chrome, etc. | Applications to monitor |
| `focused_only` | `true` | Only monitor current window |
| `monitoring_interval_ms` | `1000` | Check frequency (1 second) |
| `min_question_length` | `10` | Minimum text length to consider |
| `max_text_length` | `2000` | Maximum text to process |

## 🎯 Key Benefits

1. **Non-intrusive**: Only reads from the window you're currently using
2. **Fast Detection**: 1-second monitoring interval for real-time response
3. **Smart Filtering**: Only processes text from relevant interview applications
4. **Question Detection**: Uses regex patterns to identify potential questions
5. **Change Detection**: Only triggers on new/changed content

## 🔗 Integration Example

```javascript
// Initialize monitoring when MockMate starts
async function startInterviewMode() {
    // Start monitoring current window
    await invoke('start_realtime_monitoring');
    
    // Set up event listener for questions
    const unlisten = await window.__TAURI__.event.listen(
        'accessibility-question-detected', 
        async (event) => {
            const question = event.payload;
            
            // Show question in UI
            displayQuestion(question.text, question.source_app);
            
            // Generate AI response
            const response = await generateResponse(question.text);
            displayResponse(response);
        }
    );
    
    return unlisten;
}

// Stop monitoring when done
async function stopInterviewMode() {
    await invoke('stop_realtime_monitoring');
}
```

This solution is perfect for your use case because it focuses specifically on the **current active window** - exactly what the user is looking at during an interview!
