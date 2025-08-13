# Q&A Storage Fix: Desktop App Database Update

## Issue Identified
The desktop app was trying to store Q&A data in non-existent database tables (`interview_questions` and `interview_answers`), while the web app was reading from the `interview_messages` table. This caused Q&A data to never appear on the web app.

## Changes Made

### 1. Updated `insert_interview_question` function
**File:** `src-tauri/src/database/postgres.rs`

**Before:** Stored questions in `interview_questions` table  
**After:** Stores questions in `interview_messages` table with `message_type = 'question'`

**Key Changes:**
- Uses `interview_messages` table instead of `interview_questions`
- Stores question details in `metadata` JSON field
- Uses `message_type = 'question'`
- Preserves question number, category, difficulty in metadata

### 2. Updated `insert_interview_answer` function
**File:** `src-tauri/src/database/postgres.rs`

**Before:** Stored answers in `interview_answers` table  
**After:** Stores answers in `interview_messages` table with `message_type = 'answer'`

**Key Changes:**
- Uses `interview_messages` table instead of `interview_answers`
- Stores answer details in `metadata` JSON field
- Uses `message_type = 'answer'`
- Links answers to questions using `parent_message_id`
- Preserves response time, AI feedback, AI score in metadata

### 3. Updated `get_session_questions` function
**File:** `src-tauri/src/database/postgres.rs`

**Before:** Read from `interview_questions` table  
**After:** Reads from `interview_messages` table where `message_type = 'question'`

**Key Changes:**
- Queries `interview_messages` table
- Extracts question details from `metadata` JSON
- Maintains compatibility with existing `InterviewQuestion` struct

### 4. Updated `get_session_answers` function
**File:** `src-tauri/src/database/postgres.rs`

**Before:** Read from `interview_answers` table  
**After:** Reads from `interview_messages` table where `message_type = 'answer'`

**Key Changes:**
- Queries `interview_messages` table
- Extracts answer details from `metadata` JSON
- Uses `parent_message_id` to link answers to questions
- Maintains compatibility with existing `InterviewAnswer` struct

## Database Schema Compatibility

### interview_messages Table Structure
```sql
- id: uuid (PRIMARY KEY)
- session_id: uuid (NOT NULL)
- message_type: varchar ('question' or 'answer')
- content: text (question text or answer text)
- metadata: jsonb (structured data)
- timestamp: timestamp
- parent_message_id: uuid (links answers to questions)
```

### Metadata Structure

**For Questions:**
```json
{
  "questionNumber": 1,
  "category": "technical",
  "difficulty": "medium",
  "expectedDuration": 30,
  "source": "desktop_app",
  "timestamp": "2025-08-13T06:58:55Z"
}
```

**For Answers:**
```json
{
  "questionId": "uuid-of-question",
  "responseTime": 45,
  "aiFeedback": "Good answer...",
  "aiScore": 85,
  "source": "desktop_app", 
  "timestamp": "2025-08-13T06:58:55Z"
}
```

## Benefits of This Approach

1. **Unified Data Storage:** Both desktop and web apps now use the same table
2. **Backward Compatibility:** Existing Tauri commands still work
3. **Rich Metadata:** JSON metadata allows flexible data storage
4. **Proper Linking:** Answers are linked to questions via `parent_message_id`
5. **Web App Compatibility:** Data is immediately visible on web app

## Testing Required

After rebuilding the desktop app with these changes:

1. **Start a new interview session**
2. **Ask questions and provide answers**  
3. **Check that data appears in `interview_messages` table**
4. **Verify Q&A shows up on web app history page**

## Build Instructions

To apply these changes:

1. **Rebuild the desktop app:**
   ```bash
   cd desktop-app
   npm run tauri build
   # or
   npm run tauri dev
   ```

2. **Test Q&A storage:**
   - Start a new interview session
   - Conduct interview with questions/answers
   - Check web app to verify Q&A appears

## Expected Result

After these changes, when users conduct interviews using the desktop app:
- ✅ Questions will be stored in `interview_messages` table  
- ✅ Answers will be stored in `interview_messages` table
- ✅ Q&A data will appear on the web app history page
- ✅ Existing desktop app functionality remains unchanged
- ✅ Data structure is preserved for reports and analytics

The fix ensures that Q&A data flows correctly from desktop app → database → web app display.
