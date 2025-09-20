# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

MockMate is an AI-powered interview assistant desktop application built with Tauri (Rust backend + React frontend). The app provides real-time transcription, AI answer generation, screen analysis, and interview session management for job interview preparation.

## Development Commands

### Essential Commands

```bash
# Install dependencies
npm install

# Development mode (starts both frontend and Tauri)
npm run tauri:dev

# Build for production
npm run tauri:build

# Frontend only (for React development)
npm run dev

# Build frontend only
npm run build

# Preview built frontend
npm run preview

# TypeScript compilation
npx tsc

# Run Tauri CLI directly
npx tauri dev
npx tauri build
```

### Rust Backend Commands

```bash
# Build Rust backend only
cd src-tauri && cargo build

# Release build
cd src-tauri && cargo build --release

# Run tests
cd src-tauri && cargo test

# Check for compilation errors
cd src-tauri && cargo check

# Format code
cd src-tauri && cargo fmt

# Run clippy (linter)
cd src-tauri && cargo clippy

# Generate documentation
cd src-tauri && cargo doc --open

# Run specific binary utilities
cd src-tauri && cargo run --bin test_db_connection
cd src-tauri && cargo run --bin test_realtime_transcription
```

### Database Operations

```bash
# Test database connection
cd src-tauri && cargo run --bin test_db_connection

# Test production database
cd src-tauri && cargo run --bin test_prod_db

# List audio sessions
cd src-tauri && cargo run --bin list_sessions
```

## Architecture Overview

### High-Level Structure

MockMate follows a **hybrid desktop architecture** combining:

- **Frontend**: React 19 + TypeScript + Tauri WebView
- **Backend**: Rust with Tauri framework providing native OS integration
- **Database**: PostgreSQL for session/interview data persistence
- **Real-time Services**: WebSocket connections + audio processing + AI streaming

### Key Architectural Components

#### 1. **Tauri Bridge Layer** (`src-tauri/src/lib.rs`)
- **Central Command Hub**: ~150+ Tauri commands bridging React ↔ Rust
- **State Management**: Global app state with AI clients and interview context
- **Window Management**: Multi-window system (main app + AI response window)
- **Event System**: Real-time events for transcription, AI streaming, and UI updates

#### 2. **React Frontend** (`src/App.tsx`)
- **State-Driven UI**: Single comprehensive state object managing all app screens
- **Three Main Screens**: Session connection → Confirmation → Main interview interface
- **Real-time Integration**: Event listeners for transcription and AI streaming updates
- **Responsive Window Management**: Auto-resizing based on content and screen state

#### 3. **Audio Processing Pipeline** (`src-tauri/src/`)
- **Dual Audio Capture**: Microphone + system audio (WASAPI loopback on Windows)
- **Real-time Transcription**: Deepgram WebSocket integration with streaming results
- **Audio Management**: `audio.rs`, `wasapi_loopback.rs`, `realtime_transcription.rs`
- **Permission Handling**: Windows-specific audio permission management

#### 4. **AI Integration Layer** (`src-tauri/src/`)
- **Multi-Provider Support**: OpenAI + Pollinations.ai with fallback mechanisms
- **Streaming Responses**: Real-time AI answer generation with token-by-token display
- **Context-Aware**: Interview context (company, position, job description) integration
- **Model Selection**: Dynamic model switching with provider-specific optimizations

#### 5. **Database & Session Management** (`src-tauri/src/database/`)
- **PostgreSQL Integration**: Session tracking, questions/answers storage, user management
- **Session Lifecycle**: Connect → Activate → Track timing → Store Q&A → Finalize
- **Data Models**: Users, sessions, interview questions, answers with metadata
- **Sync Operations**: Real-time synchronization between desktop app and web platform

#### 6. **Windows-Specific Integration** (`src-tauri/src/`)
- **Accessibility API**: Screen text extraction via Windows Accessibility API
- **Screen Capture**: Screenshot analysis for interview question detection
- **Audio System Management**: Stereo Mix auto-enablement and device management
- **Permission Management**: Audio/screen permissions with user guidance

### Critical Data Flow

```
User Input → React State → Tauri Command → Rust Handler → External Service
    ↓                                                            ↓
Real-time Updates ← Tauri Events ← Background Processing ← API Response
```

## Key Development Patterns

### 1. **Tauri Command Pattern**
All frontend-backend communication uses `#[tauri::command]` functions:
```rust
#[tauri::command]
async fn some_operation(payload: SomePayload) -> Result<String, String> {
    // Implementation with proper error handling
}
```

### 2. **React State Management**
Centralized state object with TypeScript interfaces:
```typescript
interface AppState {
    currentScreen: AppScreen;
    session: SessionState;
    // ... all app state in one place
}
```

### 3. **Real-time Event Streaming**
Rust backend emits events to React frontend:
```rust
app_handle.emit("transcription-result", payload)?;
app_handle.emit("ai-stream-token", token_data)?;
```

### 4. **Error Handling Strategy**
- Rust: `Result<T, E>` with `anyhow` for complex errors
- React: Try-catch with user-friendly error messages
- Graceful degradation for external service failures

### 5. **Multi-Window Coordination**
- Main window: Primary UI and controls
- AI response window: Dedicated streaming AI responses with auto-positioning

## Environment Configuration

### Required Environment Variables

```bash
# Core AI Services
DEEPGRAM_API_KEY=your_deepgram_key
OPENAI_API_KEY=your_openai_key
POLLINATIONS_API_KEY=your_pollinations_key

# Database (PostgreSQL)
DATABASE_URL=postgresql://user:pass@host:port/dbname

# Optional AI Configuration
DEEPGRAM_MODEL=nova-3
DEEPGRAM_LANGUAGE=en-US
POLLINATIONS_REFERER=mockmate
```

### Build-Time Environment Embedding
The app uses `build.rs` to embed environment variables at compile time for secure deployment.

## Development Workflow

### 1. **Feature Development**
- Start with React UI changes in `src/App.tsx`
- Add corresponding Tauri commands in `src-tauri/src/lib.rs`
- Implement business logic in appropriate Rust modules
- Test integration between frontend and backend

### 2. **Database Changes**
- Update models in `src-tauri/src/database/models.rs`
- Modify queries in `src-tauri/src/database/postgres.rs`
- Test with database utility binaries

### 3. **AI Integration Updates**
- Modify clients in `src-tauri/src/openai.rs` or `src-tauri/src/pollinations.rs`
- Update streaming logic for real-time responses
- Test fallback mechanisms

### 4. **Audio/Transcription Changes**
- Update audio capture in `src-tauri/src/audio.rs`
- Modify transcription handling in `src-tauri/src/realtime_transcription.rs`
- Test on Windows for WASAPI compatibility

## Testing & Debugging

### Testing Commands
```bash
# Test specific components
cd src-tauri && cargo test audio
cd src-tauri && cargo test database
cd src-tauri && cargo test transcription

# Integration tests
npm run tauri:dev  # Manual testing in dev mode
```

### Debug Utilities
```bash
# Debug database connectivity
cd src-tauri && cargo run --bin test_db_connection

# Test transcription services
cd src-tauri && cargo run --bin test_realtime_transcription

# Environment variable debugging
cd src-tauri && cargo run --bin test_env_access
```

### Logging & Monitoring
- Rust: `log` crate with `env_logger` (console output in dev)
- React: Browser console for frontend debugging
- Tauri: DevTools available in development mode

## Platform-Specific Notes

### Windows Development
- **Audio Permissions**: App handles Windows audio permission requests automatically
- **WASAPI Integration**: System audio capture requires specific Windows APIs
- **DPI Awareness**: Window positioning accounts for high-DPI displays
- **Accessibility API**: Screen text reading via Windows Accessibility framework

### Build Requirements
- **Node.js** 18+ for frontend build
- **Rust** 1.70+ for Tauri backend
- **Windows SDK** for Windows-specific audio/accessibility features
- **PostgreSQL** client libraries for database connectivity

## Security & Performance Considerations

### Security
- Environment variables embedded at build time (not runtime)
- Window capture protection enabled
- API keys never exposed to frontend
- Database connections use connection pooling

### Performance
- **Streaming Optimizations**: Real-time AI response streaming with minimal latency
- **Audio Processing**: Efficient WASAPI loopback with minimal CPU usage
- **Memory Management**: Proper cleanup of audio streams and database connections
- **Window Management**: DPI-aware positioning for responsive UI

## Troubleshooting Common Issues

### Build Issues
- Ensure Rust toolchain is up-to-date: `rustup update`
- Clear build cache: `cd src-tauri && cargo clean`
- Reinstall dependencies: `rm -rf node_modules && npm install`

### Runtime Issues
- Check environment variables are properly set
- Verify database connectivity with test utilities
- Confirm audio device permissions on Windows
- Check Tauri logs in console for backend errors

### AI Service Issues
- Test individual AI providers with their respective test utilities
- Verify API keys and rate limits
- Check network connectivity and service status
- Use fallback mechanisms when primary services fail

This architecture supports a sophisticated real-time interview assistant with robust error handling, multi-provider AI integration, and platform-specific optimizations for Windows desktop deployment.
