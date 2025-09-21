# MockMate Desktop App - macOS Build Requirements Report

## Executive Summary

This report analyzes the current Windows-based MockMate Tauri desktop application and identifies the requirements, challenges, and action items needed to successfully build and run the application on macOS with full feature parity.

## Current Application Overview

MockMate is a sophisticated AI-powered interview assistant desktop application built with:
- **Backend**: Rust (Tauri framework v2)
- **Frontend**: HTML/CSS/JavaScript 
- **Database**: PostgreSQL (shared cloud database)
- **Key Features**:
  - Real-time audio transcription (Deepgram API)
  - AI response generation (OpenAI, Pollinations)
  - Windows accessibility API integration for text extraction
  - System audio and microphone capture
  - Screen content analysis
  - Session management with cloud database

## Platform-Specific Analysis

### ✅ Cross-Platform Compatible Components

**1. Tauri Configuration**
- `tauri.conf.json` is platform-agnostic
- Bundle targets include macOS formats (`.icns` icon present)
- Deep link support should work on macOS

**2. Database Integration**
- PostgreSQL client (`tokio-postgres`, `deadpool-postgres`) is cross-platform
- All database operations are platform-independent
- Environment variable configuration works across platforms

**3. Frontend Components**
- HTML/CSS/JavaScript code is platform-agnostic
- No browser-specific APIs detected
- Tauri API usage is cross-platform

**4. Core Business Logic**
- Session management
- AI integration (OpenAI, Pollinations APIs)
- WebSocket communication
- JSON serialization/deserialization

### ❌ Windows-Specific Components Requiring macOS Equivalents

**1. Windows Accessibility APIs (CRITICAL ISSUE)**
- **Files**: `src/accessibility_reader.rs` (2,500+ lines)
- **Dependencies**: `windows-sys`, `winapi`
- **Functionality**: Extracts text from other applications (Teams, Zoom, browsers)
- **Impact**: Core feature for reading interview questions from other apps

**2. Audio System Integration**
- **File**: `src/wasapi_loopback.rs` 
- **Windows API**: Uses WASAPI for system audio loopback
- **Cross-platform**: Uses `cpal` crate (should work on macOS)
- **Issue**: WASAPI-specific optimizations may not apply to macOS

**3. Shell Commands**
- **Configuration**: `tauri.conf.json` line 54: `"cmd": "/C", "start"`
- **Impact**: Windows Command Prompt specific
- **Usage**: Opening browser URLs

## Required Changes for macOS

### 1. CRITICAL: Replace Windows Accessibility APIs

**Challenge**: The core functionality relies heavily on Windows-specific accessibility APIs for reading text from other applications.

**Solutions**:

**Option A: macOS Accessibility APIs (Recommended)**
```rust
// New file: src/macos_accessibility.rs
use accessibility_sys::{AXUIElement, AXValue};
use core_graphics::window::CGWindowList;
```

**Option B: Cross-Platform Alternative**
- Use screenshot + OCR approach
- Implement universal screen text extraction
- Performance trade-offs but platform-independent

**Action Items**:
1. Create `src/macos_accessibility.rs` module
2. Implement macOS accessibility API wrappers
3. Add conditional compilation flags
4. Update `Cargo.toml` with macOS-specific dependencies

### 2. Update Cargo.toml Dependencies

**Current Windows-specific dependencies to modify**:
```toml
# Current Windows-only
windows-sys = { version = "0.52.0", features = [...] }
winapi = { version = "0.3", features = [...] }

# Add macOS-specific dependencies
[target.'cfg(target_os = "macos")'.dependencies]
accessibility-sys = "0.1"
core-graphics = "0.23"
core-foundation = "0.9"
objc = "0.2"
cocoa = "0.24"
```

### 3. Fix Shell Command Configuration

**Current** (Windows):
```json
{
  "name": "open-browser",
  "cmd": "cmd",
  "args": ["/C", "start", { "validator": "https?://.*" }]
}
```

**Required** (macOS):
```json
{
  "name": "open-browser",
  "cmd": "open",
  "args": [{ "validator": "https?://.*" }]
}
```

**Solution**: Platform-specific configurations
```json
{
  "shell": {
    "scope": [
      {
        "name": "open-browser-windows", 
        "cmd": "cmd",
        "args": ["/C", "start", { "validator": "https?://.*" }]
      },
      {
        "name": "open-browser-macos",
        "cmd": "open", 
        "args": [{ "validator": "https?://.*" }]
      }
    ]
  }
}
```

### 4. Audio System Verification

**Current Implementation**:
- Uses `cpal` crate (cross-platform)
- WASAPI optimizations for Windows

**macOS Considerations**:
- Core Audio framework integration
- Audio permissions (microphone access)
- System audio capture capabilities

**Action Items**:
1. Test audio capture on macOS
2. Verify microphone permissions handling
3. Check system audio loopback support

### 5. File System and Path Handling

**Current**:
- Uses environment variables for file paths
- `$APPDATA`, `$TEMP` in filesystem scope

**macOS Equivalent**:
```json
{
  "fs": {
    "scope": [
      "$APPDATA/*",        // Windows
      "$TEMP/*",           // Windows  
      "$HOME/Library/Application Support/MockMate/*", // macOS
      "$TMPDIR/*"          // macOS
    ]
  }
}
```

## Development Setup Requirements

### 1. macOS Development Environment

**Required**:
- macOS 10.15+ (Catalina or newer)
- Xcode Command Line Tools
- Rust toolchain with macOS targets
- Node.js/npm for frontend

**Setup Commands**:
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust with macOS targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Install Tauri CLI
cargo install tauri-cli --version "^2.0"
npm install -g @tauri-apps/cli@next
```

### 2. Code Signing and Distribution

**Requirements**:
- Apple Developer Account ($99/year)
- Code signing certificates
- Notarization for macOS 10.15+

**Configuration**:
```json
// tauri.conf.json additions
{
  "bundle": {
    "macOS": {
      "license": "LICENSE.txt",
      "minimumSystemVersion": "10.15",
      "signing": {
        "identity": "Developer ID Application: Your Name (TEAM_ID)"
      }
    }
  }
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1-2)
1. **Setup macOS development environment**
2. **Update Cargo.toml with conditional dependencies**
3. **Fix shell command configurations**
4. **Test basic Tauri app functionality**

### Phase 2: Audio System (Week 3)
1. **Test existing audio capture on macOS**
2. **Implement macOS-specific audio optimizations**
3. **Handle audio permissions**
4. **Verify microphone and system audio capture**

### Phase 3: Accessibility Implementation (Week 4-6) 
1. **Research macOS Accessibility APIs**
2. **Create macOS accessibility module**
3. **Implement text extraction for target apps**
4. **Add conditional compilation**
5. **Test with Teams, Zoom, browsers on macOS**

### Phase 4: Integration & Testing (Week 7-8)
1. **End-to-end testing**
2. **Performance optimization**
3. **UI/UX adjustments for macOS**
4. **Bug fixes and refinements**

### Phase 5: Distribution (Week 9)
1. **Code signing setup**
2. **Notarization process**
3. **App Store preparation (optional)**
4. **Distribution testing**

## Estimated Effort

- **Development Time**: 8-9 weeks
- **Complexity**: High (due to accessibility API replacement)
- **Risk Level**: Medium-High
- **Success Probability**: 85% (with proper macOS accessibility implementation)

## Critical Dependencies

1. **macOS Accessibility API access** - Requires user permission
2. **System audio capture** - May have limitations compared to Windows
3. **Application text extraction** - Core feature requiring complete rewrite
4. **Code signing certificate** - For distribution

## Recommended Next Steps

### Immediate Actions (Week 1)
1. **Set up macOS development environment**
2. **Create feature branch for macOS support**
3. **Update basic configurations (shell commands, file paths)**
4. **Test minimal Tauri app build on macOS**

### Priority Implementation
1. **Focus on accessibility API replacement first** (highest risk)
2. **Implement incremental testing approach**
3. **Maintain Windows compatibility throughout**
4. **Document platform differences**

### Success Criteria
- ✅ Application launches and runs on macOS
- ✅ Audio capture works (microphone + system audio)
- ✅ Text extraction from target applications works
- ✅ AI integration functions properly
- ✅ Database connectivity maintained
- ✅ All existing Windows features work on macOS

## Conclusion

Building MockMate for macOS is **technically feasible** but requires significant effort, primarily due to the Windows-specific accessibility API implementation. The core architecture is sound and most components are already cross-platform compatible.

**Key Success Factors**:
1. Successful implementation of macOS accessibility APIs
2. Proper handling of macOS security/permission requirements
3. Thorough testing across different macOS versions
4. Maintaining feature parity with Windows version

**Estimated Total Cost**: 8-9 weeks of development effort + Apple Developer Program costs ($99/year).

The project has a **high probability of success** with proper planning and execution of the accessibility API replacement.
