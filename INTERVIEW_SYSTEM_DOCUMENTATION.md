# MockMate Interview System Documentation

## Timer System Overview

The MockMate desktop application includes a comprehensive timer system that tracks interview sessions and manages credits. Here's how it works:

### Core Timer Functionality

#### 1. Timer States
```rust
pub struct TimerState {
    pub session_id: String,
    pub elapsed_seconds: u64,
    pub elapsed_minutes: u64,
    pub credits_used: u32,
    pub is_running: bool,
    pub started_at: u64,
    pub paused_duration: u64,
}
```

#### 2. Credit System
- **Initial Credit**: 1 credit is charged when a session starts
- **Additional Credits**: 1 credit per 60 minutes of interview time
- **Credit Calculation**: `1 + (elapsed_minutes / 60)`

#### 3. Timer Operations
- **Start**: Begins tracking time for a session
- **Pause**: Temporarily stops time tracking (paused time is excluded from billing)
- **Resume**: Continues time tracking after a pause
- **Stop**: Finalizes the session and saves to database

### Database Storage Structure

The system stores comprehensive interview data in PostgreSQL with the following tables:

#### Sessions Table
```sql
- id (UUID) - Primary session identifier
- user_id (UUID) - Reference to user
- session_name (VARCHAR) - Human-readable session name
- job_title (VARCHAR) - Position being interviewed for
- job_description (TEXT) - Detailed job description
- status (VARCHAR) - Session status (created, active, completed)
- interview_duration (INTEGER) - Total duration in minutes
- credits_used (INTEGER) - Total credits consumed
- created_at (TIMESTAMP) - Session creation time
- started_at (TIMESTAMP) - When interview actually started
- ended_at (TIMESTAMP) - When interview finished
```

#### Interview Questions Table
```sql
- id (UUID) - Unique question identifier
- session_id (UUID) - Reference to session
- question_number (INTEGER) - Order in interview
- question_text (TEXT) - The actual question asked
- category (VARCHAR) - Type: technical, behavioral, situational, introduction
- difficulty_level (VARCHAR) - Question difficulty
- expected_duration (INTEGER) - Expected answer time in minutes
- asked_at (TIMESTAMP) - When question was presented
- created_at (TIMESTAMP) - Record creation time
```

#### Interview Answers Table
```sql
- id (UUID) - Unique answer identifier
- question_id (UUID) - Reference to question
- session_id (UUID) - Reference to session
- answer_text (TEXT) - The candidate's response
- response_time (INTEGER) - Time taken to answer in seconds
- ai_feedback (TEXT) - AI-generated feedback on the answer
- ai_score (INTEGER) - Numerical score (1-10)
- answered_at (TIMESTAMP) - When answer was submitted
- created_at (TIMESTAMP) - Record creation time
```

## Tauri Commands for Frontend Integration

### Timer Management Commands
```rust
// Start the interview timer
start_interview_timer(session_id: String) -> TimerState

// Pause the timer
pause_interview_timer(session_id: String) -> TimerState

// Stop timer and finalize session
stop_interview_timer(session_id: String) -> TimerState

// Get current timer state
get_timer_state(session_id: String) -> TimerState

// Get credit usage breakdown
get_credit_usage(session_id: String) -> CreditUsage
```

### Database Operations Commands
```rust
// Save interview questions
save_interview_question(
    session_id: String,
    question_number: i32,
    question_text: String,
    category: String,
    difficulty_level: String,
    expected_duration: i32
) -> String (question_id)

// Save interview answers
save_interview_answer(
    session_id: String,
    question_id: String,
    answer_text: String,
    response_time: i32,
    ai_feedback: Option<String>,
    ai_score: Option<i32>
) -> String (answer_id)

// Retrieve session questions
get_session_questions(session_id: String) -> Vec<InterviewQuestion>

// Retrieve session answers
get_session_answers(session_id: String) -> Vec<InterviewAnswer>

// Generate comprehensive interview report
get_interview_report(session_id: String) -> SessionReport

// Finalize session duration
finalize_session_duration(session_id: String, total_minutes: i32) -> String
```

## Interview Report Generation

### Report Structure
The system generates comprehensive reports containing:

```rust
pub struct SessionReport {
    pub session: Session,                    // Session metadata
    pub user: User,                         // User information
    pub questions: Vec<InterviewQuestion>,  // All questions asked
    pub answers: Vec<InterviewAnswer>,      // All answers provided
    pub total_questions: i32,               // Question count
    pub total_answers: i32,                 // Answer count
    pub average_response_time: f64,         // Average answer time
    pub average_score: f64,                 // Average AI score
}
```

### PDF Report Data
For PDF generation on the web app, the report includes:
- **Session Details**: Duration, date, job title, user info
- **Question-Answer Pairs**: Complete conversation log
- **Performance Metrics**: Response times, AI scores, feedback
- **Statistical Summary**: Averages, totals, performance trends

## Frontend Integration Example

### JavaScript Usage
```javascript
// Start interview timer
const timerState = await invoke('start_interview_timer', { 
    sessionId: 'uuid-here' 
});

// Save a question during interview
const questionId = await invoke('save_interview_question', {
    sessionId: 'uuid-here',
    questionNumber: 1,
    questionText: 'Tell me about yourself',
    category: 'introduction',
    difficultyLevel: 'easy',
    expectedDuration: 3
});

// Save candidate's answer
const answerId = await invoke('save_interview_answer', {
    sessionId: 'uuid-here',
    questionId: questionId,
    answerText: 'I am a software developer...',
    responseTime: 120, // seconds
    aiFeedback: 'Good introduction, clear communication',
    aiScore: 8
});

// Stop timer and finalize session
const finalState = await invoke('stop_interview_timer', { 
    sessionId: 'uuid-here' 
});

// Generate report for PDF download
const report = await invoke('get_interview_report', { 
    sessionId: 'uuid-here' 
});
```

## Data Flow

### During Interview
1. **Session Start**: Timer starts, initial credit charged
2. **Questions Asked**: Each question saved with metadata
3. **Answers Recorded**: Responses saved with timing and AI feedback
4. **Credit Tracking**: Periodic sync with database every 5 minutes
5. **Session End**: Timer stopped, final duration saved

### For PDF Generation
1. **Data Retrieval**: Web app calls `get_interview_report`
2. **Report Assembly**: All session data compiled into structured format
3. **PDF Creation**: Web app generates PDF with complete interview data
4. **Download**: User receives comprehensive interview report

## Key Features

### Automatic Data Persistence
- Questions and answers automatically saved to database
- Session duration tracked precisely
- Credit usage calculated in real-time
- All data persisted for later retrieval

### Comprehensive Reporting
- Complete interview transcript
- Performance analytics
- Time tracking with pause support
- AI feedback integration

### Credit Management
- Fair usage billing (1 credit minimum + per hour)
- Pause time excluded from billing
- Automatic sync with web database
- Real-time credit tracking

This system ensures that all interview data is properly stored and available for generating detailed PDF reports through the web application interface.
