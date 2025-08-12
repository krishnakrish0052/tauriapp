# Session Started Tracking

This document explains how to use the new `session_started_at` field that tracks when an interview session actually begins.

## Overview

The `session_started_at` field is different from the existing timestamps:
- `created_at`: When the session was initially created
- `desktop_connected_at`: When the desktop app connected to the session
- `session_started_at`: **NEW** - When the interview actually began (first question asked)

## Database Schema

The `sessions` table now includes:
```sql
session_started_at TIMESTAMPTZ NULL
```

## Usage

### 1. Mark Session as Started (Backend/Rust)

When the interview begins (e.g., first question is asked), call:

```rust
use crate::database::postgres::mark_session_started;

// Mark the session as started
match mark_session_started(session_id).await {
    Ok(_) => info!("Session marked as started successfully"),
    Err(e) => error!("Failed to mark session as started: {}", e)
}
```

### 2. Mark Session as Started (Frontend/JavaScript)

From the frontend, you can call the Tauri command:

```javascript
import { invoke } from '@tauri-apps/api/tauri';

try {
    const result = await invoke('mark_session_started', {
        sessionId: 'your-session-id-here'
    });
    console.log('Session marked as started:', result);
} catch (error) {
    console.error('Failed to mark session as started:', error);
}
```

### 3. Query Session with Started Time

When querying session data, the `session_started_at` field will be included:

```javascript
const sessionReport = await invoke('get_interview_report', {
    sessionId: 'your-session-id-here'
});

console.log('Session created:', sessionReport.session.created_at);
console.log('Desktop connected:', sessionReport.session.desktop_connected_at);
console.log('Interview started:', sessionReport.session.session_started_at);
```

## Integration Points

### Recommended Integration

1. **On First Question**: When the first interview question is displayed or asked
```javascript
// When displaying the first question
await invoke('mark_session_started', { sessionId });
```

2. **On Interview Timer Start**: When the interview timer begins
```javascript
// When starting the interview timer
await Promise.all([
    startInterviewTimer(),
    invoke('mark_session_started', { sessionId })
]);
```

3. **On Active Status Change**: When session status changes from 'created' to 'active'
```javascript
// This is handled automatically in the database layer
// session_started_at is set when status becomes 'active'
```

## Database Queries

### Get Sessions Started Today
```sql
SELECT * FROM sessions 
WHERE DATE(session_started_at) = CURRENT_DATE;
```

### Get Average Time from Connection to Start
```sql
SELECT 
    AVG(EXTRACT(EPOCH FROM (session_started_at - desktop_connected_at))) as avg_seconds
FROM sessions 
WHERE session_started_at IS NOT NULL 
  AND desktop_connected_at IS NOT NULL;
```

### Get Sessions by Start Time Range
```sql
SELECT * FROM sessions 
WHERE session_started_at BETWEEN '2024-01-01' AND '2024-01-31';
```

## Migration

To add this field to an existing database, run the migration script:

```sql
-- See: src-tauri/migrations/add_session_started_at.sql
ALTER TABLE sessions ADD COLUMN session_started_at TIMESTAMPTZ;
```

## Best Practices

1. **Call Once**: Only mark a session as started once. The database method is idempotent.
2. **Early Timing**: Call as soon as the interview actually begins, not when preparing.
3. **Error Handling**: Always handle potential errors when marking sessions as started.
4. **Logging**: The system automatically logs when sessions are marked as started.

## Example Flow

1. User launches desktop app → `created_at` set
2. Desktop app connects to session → `desktop_connected_at` set, status = 'active'  
3. First interview question displayed → `session_started_at` set via `mark_session_started()`
4. Interview completes → status = 'completed'

This provides granular tracking of the session lifecycle for better analytics and user experience.
