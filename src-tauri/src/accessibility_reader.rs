use anyhow::Result;
use log::{info, debug, warn};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time::interval;
use windows_sys::Win32::{
    Foundation::{HWND, BOOL, TRUE, FALSE},
    UI::{
        WindowsAndMessaging::{
            GetWindowTextW, EnumWindows, IsWindowVisible,
            SendMessageW, WM_GETTEXT, GetClassNameW,
            GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE
        },
    },
    System::{
        Com::{CoInitialize, CoUninitialize},
        ProcessStatus::K32GetModuleBaseNameW,
        Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};
use winapi::um::{
    winuser::{GetWindowThreadProcessId, GetForegroundWindow, SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE},
};

/// Configuration for accessibility text reading
#[derive(Debug, Clone)]
pub struct AccessibilityConfig {
    /// Target applications to monitor (e.g., "Teams", "Zoom", "Chrome")
    pub target_apps: Vec<String>,
    /// Whether to read from focused window only
    pub focused_only: bool,
    /// Minimum text length to consider as a question
    pub min_question_length: usize,
    /// Maximum text length to process
    pub max_text_length: usize,
    /// Monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Enable real-time monitoring
    pub enable_realtime_monitoring: bool,
    /// Enable OCR fallback
    pub enable_ocr_fallback: bool,
    /// Track previously focused window for background monitoring
    pub track_previous_focus: bool,
    /// Monitor all windows regardless of visibility
    pub monitor_hidden_windows: bool,
    /// Temporarily bring windows to front for text extraction
    pub allow_window_activation: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            target_apps: vec![
                "Microsoft Teams".to_string(),
                "Zoom".to_string(),
                "Google Chrome".to_string(),
                "Mozilla Firefox".to_string(),
                "Edge".to_string(),
                "Notepad".to_string(),
                "Visual Studio Code".to_string(),
                "Discord".to_string(),
                "Slack".to_string(),
                "WhatsApp".to_string(),
            ],
            focused_only: true,  // Focus on current window only
            min_question_length: 5,  // Lower threshold for better browser content capture
            max_text_length: 2000,
            monitoring_interval_ms: 1000,  // Check more frequently for active window
            enable_realtime_monitoring: true,
            enable_ocr_fallback: true,
            track_previous_focus: false,  // Not needed for current window focus
            monitor_hidden_windows: false,  // Not needed for current window focus
            allow_window_activation: false,
        }
    }
}

/// Result of text extraction from accessibility APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityTextResult {
    /// Extracted text content
    pub text: String,
    /// Source application name
    pub source_app: String,
    /// Window title
    pub window_title: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Whether this looks like a question
    pub is_potential_question: bool,
    /// Timestamp of extraction
    pub timestamp: u64,
    /// Extraction method used
    pub extraction_method: String,
    /// Window class name
    pub window_class: String,
    /// Process ID
    pub process_id: u32,
    /// Text length in characters
    pub text_length: usize,
}

/// Windows-specific accessibility text reader
pub struct WindowsAccessibilityReader {
    config: AccessibilityConfig,
    last_seen_text: Option<String>,
    question_patterns: Vec<regex::Regex>,
    /// Track the last focused window before MockMate took focus
    previous_focused_window: Option<HWND>,
    /// Track per-window text state for hidden windows
    window_text_cache: std::collections::HashMap<HWND, String>,
}

impl WindowsAccessibilityReader {
    pub fn new(config: AccessibilityConfig) -> Result<Self> {
        // Initialize COM for UI Automation
        unsafe {
            CoInitialize(std::ptr::null_mut());
        }

        // Compile question detection patterns
        let question_patterns = vec![
            regex::Regex::new(r".*\?$").unwrap(),                           // Ends with question mark
            regex::Regex::new(r"(?i)^(what|how|why|when|where|which|who|can you|could you|would you|do you|have you|are you|will you|did you)").unwrap(),
            regex::Regex::new(r"(?i)(explain|describe|tell me|walk me through|discuss)").unwrap(),
            regex::Regex::new(r"(?i)(experience with|familiar with|worked with|used)").unwrap(),
            regex::Regex::new(r"(?i)(interview question|technical question)").unwrap(),
            // Technical content patterns
            regex::Regex::new(r"(?i)(write|create|build|develop|implement|design)").unwrap(),
            regex::Regex::new(r"(?i)(jenkins|docker|kubernetes|python|javascript|react|node)").unwrap(),
            regex::Regex::new(r"(?i)(pipeline|script|function|class|method|api)").unwrap(),
        ];

        Ok(Self {
            config,
            last_seen_text: None,
            question_patterns,
            previous_focused_window: None,
            window_text_cache: std::collections::HashMap::new(),
        })
    }

    /// Read text from the currently focused/active window
    pub fn read_text_from_current_window(&mut self) -> Result<Option<AccessibilityTextResult>> {
        info!("🎯 Reading text from current active window...");

        let focused_hwnd = unsafe { GetForegroundWindow() };
        if focused_hwnd.is_null() {
            debug!("No window is currently focused");
            return Ok(None);
        }

        // Get window information first to check if it's a target app
        let window_title = self.get_window_title(focused_hwnd as isize).unwrap_or_else(|_| "Unknown".to_string());
        let app_name = self.get_application_name(focused_hwnd as isize).unwrap_or_else(|_| "Unknown".to_string());
        
        // Check if this window is from a target application
        if !self.is_target_application(&window_title, &app_name) {
            debug!("Current window '{}' from '{}' is not a target application", window_title, app_name);
            return Ok(None);
        }

        info!("📖 Extracting from current window: {} ({})", app_name, window_title);
        
        match self.extract_text_from_window(focused_hwnd as isize) {
            Ok(Some(result)) => {
                if self.is_new_content(&result.text) {
                    info!("📝 New content detected from current window {}: {}", 
                          result.source_app, 
                          result.text.chars().take(100).collect::<String>());
                    Ok(Some(result))
                } else {
                    debug!("Content hasn't changed in current window");
                    Ok(None)
                }
            }
            Ok(None) => {
                debug!("No meaningful text found in current window");
                Ok(None)
            }
            Err(e) => {
                debug!("Failed to extract text from current window: {}", e);
                Ok(None)
            }
        }
    }
    
    /// Check if the given window/app is in our target applications list
    fn is_target_application(&self, window_title: &str, app_name: &str) -> bool {
        let window_lower = window_title.to_lowercase();
        let app_lower = app_name.to_lowercase();
        
        for target_app in &self.config.target_apps {
            let target_lower = target_app.to_lowercase();
            if window_lower.contains(&target_lower) || app_lower.contains(&target_lower) {
                return true;
            }
        }
        
        // Also check for common interview/meeting keywords
        let interview_keywords = ["meeting", "interview", "call", "video", "conference"];
        for keyword in &interview_keywords {
            if window_lower.contains(keyword) {
                return true;
            }
        }
        
        false
    }

    /// Read text from the currently focused window
    pub fn read_text_from_focused_window(&mut self) -> Result<Option<AccessibilityTextResult>> {
        info!("🎯 Reading text from focused window...");

        let focused_hwnd = unsafe { GetForegroundWindow() };
        if focused_hwnd.is_null() {
            return Ok(None);
        }

        self.extract_text_from_window(focused_hwnd as isize)
    }

    /// Extract text from a specific window using UI Automation
    fn extract_text_from_window(&self, hwnd: HWND) -> Result<Option<AccessibilityTextResult>> {
        // Get window information
        let window_title = self.get_window_title(hwnd)?;
        let app_name = self.get_application_name(hwnd)?;

        debug!("📖 Extracting text from: {} ({})", app_name, window_title);

        // Try multiple text extraction methods
        let extracted_text = self.extract_text_multiple_methods(hwnd)?;

        if extracted_text.trim().is_empty() || extracted_text.len() < self.config.min_question_length {
            debug!("Text too short or empty: {} chars", extracted_text.len());
            return Ok(None);
        }

        if extracted_text.len() > self.config.max_text_length {
            warn!("Text too long, truncating: {} chars", extracted_text.len());
        }

        let truncated_text = extracted_text.chars().take(self.config.max_text_length).collect::<String>();
        
        let is_potential_question = self.detect_question_patterns(&truncated_text);
        
        // Get additional window information
        let window_class = self.get_window_class_name(hwnd).unwrap_or_else(|_| "Unknown".to_string());
        let process_id = unsafe {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd as *mut winapi::shared::windef::HWND__, &mut pid);
            pid
        };
        
        let result = AccessibilityTextResult {
            text: truncated_text.clone(),
            source_app: app_name,
            window_title,
            confidence: 0.8, // High confidence for accessibility API
            is_potential_question,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            extraction_method: "UI_Automation".to_string(),
            window_class,
            process_id,
            text_length: truncated_text.len(),
        };

        Ok(Some(result))
    }

    /// Try multiple methods to extract text from a window
    fn extract_text_multiple_methods(&self, hwnd: HWND) -> Result<String> {
        // For web browsers, prioritize content area extraction
        let app_name = self.get_application_name(hwnd).unwrap_or_default();
        let window_title = self.get_window_title(hwnd).unwrap_or_default();
        
        let is_browser = app_name.to_lowercase().contains("chrome") ||
                        app_name.to_lowercase().contains("firefox") ||
                        app_name.to_lowercase().contains("edge") ||
                        window_title.to_lowercase().contains("chrome") ||
                        window_title.to_lowercase().contains("firefox") ||
                        window_title.to_lowercase().contains("edge");

        if is_browser {
            debug!("🌐 Detected web browser, using enhanced content extraction");
            // For browsers, try specialized content extraction first
            if let Ok(text) = self.extract_browser_content(hwnd) {
                if !text.trim().is_empty() && text.len() > 10 {
                    debug!("✅ Browser content extraction found {} chars", text.len());
                    return Ok(text);
                }
            }
        }

        // Method 1: UI Automation (most reliable)
        if let Ok(text) = self.extract_text_ui_automation(hwnd) {
            if !text.trim().is_empty() {
                debug!("✅ UI Automation extracted {} chars", text.len());
                return Ok(text);
            }
        }

        // Method 2: SendMessage with WM_GETTEXT (for simple controls)
        if let Ok(text) = self.extract_text_sendmessage(hwnd) {
            if !text.trim().is_empty() {
                debug!("✅ SendMessage extracted {} chars", text.len());
                return Ok(text);
            }
        }

        // Method 3: Clipboard monitoring (as fallback)
        if let Ok(text) = self.extract_text_clipboard() {
            if !text.trim().is_empty() {
                debug!("✅ Clipboard extracted {} chars", text.len());
                return Ok(text);
            }
        }

        debug!("⚠️ All text extraction methods failed");
        Ok(String::new())
    }

    /// Extract text using UI Automation API
    fn extract_text_ui_automation(&self, hwnd: HWND) -> Result<String> {
        debug!("🤖 UI Automation text extraction starting...");
        
        // Try different UI Automation approaches
        let mut extracted_texts = Vec::new();
        
        // Method 1: Extract from child windows (most common for chat applications)
        if let Ok(child_texts) = self.extract_from_child_windows(hwnd) {
            extracted_texts.extend(child_texts);
        }
        
        // Method 2: Extract from edit controls
        if let Ok(edit_texts) = self.extract_from_edit_controls(hwnd) {
            extracted_texts.extend(edit_texts);
        }
        
        // Method 3: Extract using accessibility patterns
        if let Ok(accessible_text) = self.extract_using_accessibility_patterns(hwnd) {
            if !accessible_text.trim().is_empty() {
                extracted_texts.push(accessible_text);
            }
        }
        
        // Method 4: Extract from rich text controls (Teams, Zoom often use these)
        if let Ok(rich_texts) = self.extract_from_rich_text_controls(hwnd) {
            extracted_texts.extend(rich_texts);
        }
        
        // Prioritize and filter extracted texts
        let prioritized_content = self.prioritize_extracted_content(&extracted_texts);
        
        if !prioritized_content.trim().is_empty() {
            debug!("✅ UI Automation extracted {} chars total", prioritized_content.len());
            Ok(prioritized_content)
        } else {
            debug!("⚠️ UI Automation found no meaningful text");
            Ok(String::new())
        }
    }
    
    /// Extract text from child windows
    fn extract_from_child_windows(&self, parent_hwnd: HWND) -> Result<Vec<String>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{EnumChildWindows, GetClassNameW};
        
        let mut texts = Vec::new();
        
        unsafe extern "system" fn enum_child_proc(hwnd: HWND, lparam: isize) -> BOOL {
            let texts_ptr = lparam as *mut Vec<String>;
            let texts = &mut *texts_ptr;
            
            // Get window class name
            let mut class_name: [u16; 256] = [0; 256];
            let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 256);
            
            if class_len > 0 {
                let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                
                // Check for common text-containing controls
                if class_str.to_lowercase().contains("edit") ||
                   class_str.to_lowercase().contains("static") ||
                   class_str.to_lowercase().contains("richedit") ||
                   class_str.to_lowercase().contains("text") {
                    
                    // Try to get text from this control
                    let mut buffer: [u16; 2048] = [0; 2048];
                    let length = SendMessageW(hwnd, WM_GETTEXT, 2048, buffer.as_mut_ptr() as isize);
                    
                    if length > 0 {
                        let text = String::from_utf16_lossy(&buffer[..length as usize]);
                        if !text.trim().is_empty() && text.len() >= 5 {
                            texts.push(text);
                        }
                    }
                }
            }
            
            TRUE
        }
        
        unsafe {
            EnumChildWindows(parent_hwnd, Some(enum_child_proc), &mut texts as *mut _ as isize);
        }
        
        debug!("Found {} text blocks from child windows", texts.len());
        Ok(texts)
    }
    
    /// Extract text from edit controls specifically
    fn extract_from_edit_controls(&self, hwnd: HWND) -> Result<Vec<String>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            WM_GETTEXTLENGTH, GetWindow, GW_CHILD, GW_HWNDNEXT
        };
        
        let mut texts = Vec::new();
        
        // Look for edit controls in the window hierarchy
        let _edit_class = "Edit\0".encode_utf16().collect::<Vec<u16>>();
        
        unsafe {
            let mut child_hwnd = GetWindow(hwnd, GW_CHILD);
            
            while child_hwnd != 0 {
                // Check if this is an edit control
                let mut class_name: [u16; 256] = [0; 256];
                let class_len = GetClassNameW(child_hwnd, class_name.as_mut_ptr(), 256);
                
                if class_len > 0 {
                    let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                    
                    if class_str.to_lowercase().contains("edit") {
                        // Get text length first
                        let text_length = SendMessageW(child_hwnd, WM_GETTEXTLENGTH, 0, 0);
                        
                        if text_length > 0 && text_length < 4096 {
                            let mut buffer = vec![0u16; (text_length + 1) as usize];
                            let actual_length = SendMessageW(
                                child_hwnd,
                                WM_GETTEXT,
                                buffer.len(),
                                buffer.as_mut_ptr() as isize,
                            );
                            
                            if actual_length > 0 {
                                let text = String::from_utf16_lossy(&buffer[..actual_length as usize]);
                                if !text.trim().is_empty() && text.len() >= 5 {
                                    texts.push(text);
                                }
                            }
                        }
                    }
                }
                
                child_hwnd = GetWindow(child_hwnd, GW_HWNDNEXT);
            }
        }
        
        debug!("Found {} text blocks from edit controls", texts.len());
        Ok(texts)
    }
    
    /// Extract using Windows accessibility patterns
    fn extract_using_accessibility_patterns(&self, hwnd: HWND) -> Result<String> {
        // This would use proper UI Automation COM interfaces
        // For now, we'll use a simpler approach that works with most applications
        
        // Try to get accessible text using legacy accessibility APIs
        let text = self.extract_using_legacy_accessibility(hwnd)?;
        
        debug!("Accessibility patterns extracted {} chars", text.len());
        Ok(text)
    }
    
    /// Extract using legacy accessibility APIs
    fn extract_using_legacy_accessibility(&self, hwnd: HWND) -> Result<String> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWL_STYLE, WS_VISIBLE};
        
        // Check if window is visible
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            if (style & WS_VISIBLE as isize) == 0 {
                return Ok(String::new());
            }
        }
        
        // Try multiple approaches to get accessible text
        let mut texts = Vec::new();
        
        // Approach 1: Direct window text
        if let Ok(window_text) = self.get_window_title(hwnd) {
            if !window_text.trim().is_empty() && window_text != "Unknown" {
                texts.push(window_text);
            }
        }
        
        // Approach 2: Try to get text from the window using different messages
        let text_messages = [
            WM_GETTEXT,
            0x000D, // WM_GETTEXT alternative
            0x0030, // WM_SETTEXT - sometimes reveals text
        ];
        
        for &msg in &text_messages {
            unsafe {
                let mut buffer: [u16; 1024] = [0; 1024];
                let length = SendMessageW(hwnd, msg, 1024, buffer.as_mut_ptr() as isize);
                
                if length > 0 {
                    let text = String::from_utf16_lossy(&buffer[..length as usize]);
                    if !text.trim().is_empty() && text.len() >= 3 {
                        texts.push(text);
                    }
                }
            }
        }
        
        Ok(texts.join(" "))
    }
    
    /// Extract from rich text controls (common in Teams, Zoom)
    fn extract_from_rich_text_controls(&self, hwnd: HWND) -> Result<Vec<String>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, GW_CHILD, GW_HWNDNEXT};
        
        let mut texts = Vec::new();
        
        unsafe {
            let mut child_hwnd = GetWindow(hwnd, GW_CHILD);
            
            while child_hwnd != 0 {
                // Get class name to identify rich text controls
                let mut class_name: [u16; 256] = [0; 256];
                let class_len = GetClassNameW(child_hwnd, class_name.as_mut_ptr(), 256);
                
                if class_len > 0 {
                    let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                    
                    // Check for rich text control classes
                    if class_str.to_lowercase().contains("richedit") ||
                       class_str.to_lowercase().contains("richedit20w") ||
                       class_str.to_lowercase().contains("richedit50w") ||
                       class_str.contains("RichEdit") {
                        
                        // Try to extract text from rich text control
                        let mut buffer: [u16; 4096] = [0; 4096];
                        let length = SendMessageW(child_hwnd, WM_GETTEXT, 4096, buffer.as_mut_ptr() as isize);
                        
                        if length > 0 {
                            let text = String::from_utf16_lossy(&buffer[..length as usize]);
                            if !text.trim().is_empty() && text.len() >= 5 {
                                texts.push(text);
                            }
                        }
                    }
                }
                
                child_hwnd = GetWindow(child_hwnd, GW_HWNDNEXT);
            }
        }
        
        debug!("Found {} text blocks from rich text controls", texts.len());
        Ok(texts)
    }

    /// Extract text using SendMessage WM_GETTEXT
    fn extract_text_sendmessage(&self, hwnd: HWND) -> Result<String> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_GETTEXT};

        const MAX_TEXT_LENGTH: usize = 4096;
        let mut buffer: Vec<u16> = vec![0; MAX_TEXT_LENGTH];

        let length = unsafe {
            SendMessageW(
                hwnd,
                WM_GETTEXT,
                MAX_TEXT_LENGTH,
                buffer.as_mut_ptr() as isize,
            )
        };

        if length > 0 {
            buffer.truncate(length as usize);
            let text = String::from_utf16_lossy(&buffer);
            debug!("📝 SendMessage extracted: {}", text.chars().take(50).collect::<String>());
            Ok(text)
        } else {
            Ok(String::new())
        }
    }

    /// Extract text from clipboard (fallback method)
    fn extract_text_clipboard(&self) -> Result<String> {
        // Clipboard access is complex and not always reliable.
        // For now, we'll return empty text as this is a fallback method
        // and the main UI Automation methods should be sufficient.
        debug!("📋 Clipboard extraction skipped (placeholder implementation)");
        Ok(String::new())
    }

    /// Extract content specifically from web browsers
    fn extract_browser_content(&self, hwnd: HWND) -> Result<String> {
        debug!("🌐 Starting browser-specific content extraction...");
        
        let mut content_texts = Vec::new();
        
        // For browsers, prioritize content areas over UI elements
        // Method 1: Look for text input areas and contenteditable elements
        if let Ok(input_texts) = self.extract_browser_input_content(hwnd) {
            content_texts.extend(input_texts);
        }
        
        // Method 2: Extract from document content areas
        if let Ok(document_texts) = self.extract_browser_document_content(hwnd) {
            content_texts.extend(document_texts);
        }
        
        // Method 3: Look for text areas and rich editors
        if let Ok(editor_texts) = self.extract_browser_editor_content(hwnd) {
            content_texts.extend(editor_texts);
        }
        
        // Filter out UI noise and prioritize meaningful content
        let filtered_content = self.filter_browser_content(&content_texts);
        
        debug!("🌐 Browser content extraction found {} relevant text blocks", filtered_content.len());
        
        Ok(filtered_content.join(" "))
    }
    
    /// Extract content from browser input fields
    fn extract_browser_input_content(&self, hwnd: HWND) -> Result<Vec<String>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, GW_CHILD, GW_HWNDNEXT};
        
        let mut texts = Vec::new();
        
        unsafe {
            let mut child_hwnd = GetWindow(hwnd, GW_CHILD);
            
            while child_hwnd != 0 {
                let mut class_name: [u16; 256] = [0; 256];
                let class_len = GetClassNameW(child_hwnd, class_name.as_mut_ptr(), 256);
                
                if class_len > 0 {
                    let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                    
                    // Look for browser input elements and content editable areas
                    if class_str.to_lowercase().contains("edit") ||
                       class_str.to_lowercase().contains("chrome_widgetwin") ||
                       class_str.to_lowercase().contains("internet explorer") ||
                       class_str.to_lowercase().contains("gecko") ||
                       class_str.to_lowercase().contains("webkit") {
                        
                        let mut buffer: [u16; 4096] = [0; 4096];
                        let length = SendMessageW(child_hwnd, WM_GETTEXT, 4096, buffer.as_mut_ptr() as isize);
                        
                        if length > 0 {
                            let text = String::from_utf16_lossy(&buffer[..length as usize]);
                            if self.is_meaningful_content(&text) {
                                texts.push(text);
                            }
                        }
                    }
                }
                
                // Recursively check child windows
                if let Ok(child_texts) = self.extract_browser_input_content(child_hwnd) {
                    texts.extend(child_texts);
                }
                
                child_hwnd = GetWindow(child_hwnd, GW_HWNDNEXT);
            }
        }
        
        debug!("Found {} input content blocks", texts.len());
        Ok(texts)
    }
    
    /// Extract content from browser document areas
    fn extract_browser_document_content(&self, hwnd: HWND) -> Result<Vec<String>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, GW_CHILD, GW_HWNDNEXT};
        
        let mut texts = Vec::new();
        
        // Look for browser rendering areas and document containers
        unsafe {
            let mut child_hwnd = GetWindow(hwnd, GW_CHILD);
            
            while child_hwnd != 0 {
                let mut class_name: [u16; 256] = [0; 256];
                let class_len = GetClassNameW(child_hwnd, class_name.as_mut_ptr(), 256);
                
                if class_len > 0 {
                    let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                    
                    // Browser-specific content containers
                    if class_str.contains("Chrome_RenderWidgetHostHWND") ||
                       class_str.contains("Mozilla") ||
                       class_str.contains("Edge") ||
                       class_str.contains("WebView") ||
                       class_str.to_lowercase().contains("document") {
                        
                        // Try to get text from this rendering area
                        if let Ok(child_texts) = self.extract_from_child_windows(child_hwnd) {
                            for text in child_texts {
                                if self.is_meaningful_content(&text) {
                                    texts.push(text);
                                }
                            }
                        }
                    }
                }
                
                child_hwnd = GetWindow(child_hwnd, GW_HWNDNEXT);
            }
        }
        
        debug!("Found {} document content blocks", texts.len());
        Ok(texts)
    }
    
    /// Extract content from text editors in browsers
    fn extract_browser_editor_content(&self, hwnd: HWND) -> Result<Vec<String>> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, GW_CHILD, GW_HWNDNEXT};
        
        let mut texts = Vec::new();
        
        unsafe {
            let mut child_hwnd = GetWindow(hwnd, GW_CHILD);
            
            while child_hwnd != 0 {
                let mut class_name: [u16; 256] = [0; 256];
                let class_len = GetClassNameW(child_hwnd, class_name.as_mut_ptr(), 256);
                
                if class_len > 0 {
                    let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                    
                    // Look for text editors, code editors, and content editable areas
                    if class_str.to_lowercase().contains("textarea") ||
                       class_str.to_lowercase().contains("editor") ||
                       class_str.to_lowercase().contains("codemirror") ||
                       class_str.to_lowercase().contains("ace_editor") ||
                       class_str.to_lowercase().contains("monaco") {
                        
                        let mut buffer: [u16; 8192] = [0; 8192];
                        let length = SendMessageW(child_hwnd, WM_GETTEXT, 8192, buffer.as_mut_ptr() as isize);
                        
                        if length > 0 {
                            let text = String::from_utf16_lossy(&buffer[..length as usize]);
                            if self.is_meaningful_content(&text) {
                                texts.push(text);
                            }
                        }
                    }
                }
                
                // Recursively check deeper
                if let Ok(child_texts) = self.extract_browser_editor_content(child_hwnd) {
                    texts.extend(child_texts);
                }
                
                child_hwnd = GetWindow(child_hwnd, GW_HWNDNEXT);
            }
        }
        
        debug!("Found {} editor content blocks", texts.len());
        Ok(texts)
    }
    
    /// Filter browser content to prioritize meaningful text over UI noise
    fn filter_browser_content(&self, content_texts: &[String]) -> Vec<String> {
        let mut filtered = Vec::new();
        
        for text in content_texts {
            let text = text.trim();
            
            // Skip if too short
            if text.len() < 3 {
                continue;
            }
            
            // Skip common UI noise
            let text_lower = text.to_lowercase();
            let ui_noise = [
                "online notepad", "file", "edit", "view", "help", "font family",
                "font sizes", "untitled document", "loading", "please wait",
                "click here", "menu", "toolbar", "status", "ready"
            ];
            
            // Skip if it's just UI noise
            if ui_noise.iter().any(|noise| text_lower == *noise) {
                continue;
            }
            
            // Prioritize longer, meaningful content
            if text.len() >= 5 {
                filtered.push(text.to_string());
            }
        }
        
        // Sort by length descending - longer content is more likely to be meaningful
        filtered.sort_by(|a, b| b.len().cmp(&a.len()));
        
        debug!("Filtered {} content blocks to {} meaningful texts", content_texts.len(), filtered.len());
        filtered
    }
    
    /// Check if content is meaningful (not just UI elements)
    fn is_meaningful_content(&self, text: &str) -> bool {
        let text = text.trim();
        
        // Must be at least 3 characters
        if text.len() < 3 {
            return false;
        }
        
        // Skip single words that are likely UI elements
        if !text.contains(' ') && text.len() < 10 {
            return false;
        }
        
        // Skip pure UI text
        let ui_elements = [
            "font", "size", "edit", "view", "file", "help", "menu", "toolbar",
            "ready", "loading", "please wait", "click", "button", "link"
        ];
        
        let text_lower = text.to_lowercase();
        if ui_elements.iter().any(|element| text_lower == *element) {
            return false;
        }
        
        // Prefer content with some complexity
        text.len() >= 10 || text.contains(' ') || self.detect_question_patterns(text)
    }
    
    /// Prioritize extracted content to favor user-typed text over UI noise
    fn prioritize_extracted_content(&self, texts: &[String]) -> String {
        if texts.is_empty() {
            return String::new();
        }
        
        // Separate texts into different priority levels
        let mut high_priority = Vec::new(); // Likely user content
        let mut medium_priority = Vec::new(); // Potentially user content
        let mut low_priority = Vec::new(); // UI elements
        
        for text in texts {
            let text_lower = text.trim().to_lowercase();
            let score = self.calculate_content_priority_score(&text_lower, text.len());
            
            if score >= 80 {
                high_priority.push(text.clone());
            } else if score >= 40 {
                medium_priority.push(text.clone());
            } else {
                low_priority.push(text.clone());
            }
        }
        
        debug!("Content prioritization: {} high, {} medium, {} low priority texts", 
               high_priority.len(), medium_priority.len(), low_priority.len());
        
        // Return in priority order, preferring high priority content
        if !high_priority.is_empty() {
            debug!("✅ Using high priority content");
            high_priority.sort_by(|a, b| b.len().cmp(&a.len())); // Longest first
            high_priority.join(" ")
        } else if !medium_priority.is_empty() {
            debug!("⚠️ Using medium priority content");
            medium_priority.sort_by(|a, b| b.len().cmp(&a.len())); // Longest first
            medium_priority.join(" ")
        } else {
            debug!("❌ Only low priority content available");
            low_priority.sort_by(|a, b| b.len().cmp(&a.len())); // Longest first
            low_priority.join(" ")
        }
    }
    
    /// Calculate priority score for content (0-100, higher = more likely to be user content)
    fn calculate_content_priority_score(&self, text_lower: &str, original_length: usize) -> u32 {
        let mut score = 0u32;
        
        // Base score based on length - longer text is more likely to be meaningful
        if original_length >= 50 {
            score += 30;
        } else if original_length >= 20 {
            score += 20;
        } else if original_length >= 10 {
            score += 10;
        }
        
        // High priority indicators (likely user content)
        let user_content_indicators = [
            "write", "create", "build", "develop", "implement", "jenkins", "pipeline",
            "script", "function", "class", "method", "api", "docker", "kubernetes",
            "python", "javascript", "react", "node", "sql", "database", "server",
            "algorithm", "data structure", "framework", "library", "microservice"
        ];
        
        for indicator in &user_content_indicators {
            if text_lower.contains(indicator) {
                score += 25;
                break; // Don't double-count
            }
        }
        
        // Technical action words get high priority
        if text_lower.contains("write a") || text_lower.contains("create a") || 
           text_lower.contains("build a") || text_lower.contains("implement a") {
            score += 35;
        }
        
        // Question patterns get medium-high priority
        if self.detect_question_patterns(text_lower) {
            score += 20;
        }
        
        // Medium priority indicators
        if text_lower.contains(" ") && original_length >= 15 {
            score += 15; // Multi-word phrases of decent length
        }
        
        // Low priority penalties (UI noise)
        let ui_noise_exact = [
            "online notepad", "file", "edit", "view", "help", "font family", "font sizes",
            "untitled document", "loading", "please wait", "ready", "menu", "toolbar",
            "status", "home", "search", "settings", "options", "preferences", "about"
        ];
        
        for noise in &ui_noise_exact {
            if text_lower == *noise {
                score = score.saturating_sub(50); // Heavy penalty for exact UI matches
                break;
            }
        }
        
        // Partial UI noise penalties
        let ui_noise_partial = ["click", "button", "link", "tab", "window", "dialog"];
        for noise in &ui_noise_partial {
            if text_lower.contains(noise) && original_length < 20 {
                score = score.saturating_sub(20);
            }
        }
        
        // Single character or very short text penalty
        if original_length <= 3 {
            score = score.saturating_sub(30);
        }
        
        // Cap score at 100
        std::cmp::min(score, 100)
    }

    /// Enumerate all windows (including hidden ones if configured)
    fn enumerate_windows(&self) -> Result<Vec<WindowInfo>> {
        let mut windows: Vec<WindowInfo> = Vec::new();
        let monitor_hidden = self.config.monitor_hidden_windows;
        
        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> BOOL {
            let (windows_ptr, monitor_hidden) = {
                let lparam_tuple = lparam as *mut (Vec<WindowInfo>, bool);
                let tuple_ref = &mut *lparam_tuple;
                (&mut tuple_ref.0, tuple_ref.1)
            };

            // Check visibility based on configuration
            let should_include = if monitor_hidden {
                // Include all windows, even hidden ones
                true
            } else {
                IsWindowVisible(hwnd) == TRUE
            };

            if should_include {
                let mut title_buffer: [u16; 512] = [0; 512];
                let title_len = GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), 512);
                
                if title_len > 0 {
                    let title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);
                    if !title.trim().is_empty() {
                        windows_ptr.push(WindowInfo {
                            hwnd,
                            title,
                        });
                    }
                } else if monitor_hidden {
                    // For hidden windows, also check if they might be interesting
                    // even without a title
                    let mut class_name: [u16; 256] = [0; 256];
                    let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 256);
                    
                    if class_len > 0 {
                        let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                        // Include windows with interesting class names
                        if class_str.to_lowercase().contains("chrome") ||
                           class_str.to_lowercase().contains("teams") ||
                           class_str.to_lowercase().contains("zoom") ||
                           class_str.to_lowercase().contains("firefox") ||
                           class_str.to_lowercase().contains("edge") {
                            windows_ptr.push(WindowInfo {
                                hwnd,
                                title: format!("[Hidden] {}", class_str),
                            });
                        }
                    }
                }
            }

            TRUE
        }

        let mut enum_data = (windows, monitor_hidden);
        unsafe {
            EnumWindows(Some(enum_proc), &mut enum_data as *mut _ as isize);
        }

        windows = enum_data.0;
        let hidden_count = windows.iter().filter(|w| w.title.starts_with("[Hidden]")).count();
        debug!("Found {} windows (including {} hidden)", windows.len(), hidden_count);
        Ok(windows)
    }

    /// Get window title
    fn get_window_title(&self, hwnd: HWND) -> Result<String> {
        let mut title_buffer: [u16; 512] = [0; 512];
        let length = unsafe { GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), 512) };
        
        if length > 0 {
            Ok(String::from_utf16_lossy(&title_buffer[..length as usize]))
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Get application name from window
    fn get_application_name(&self, hwnd: HWND) -> Result<String> {
        use windows_sys::Win32::Foundation::CloseHandle;

        unsafe {
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd as *mut winapi::shared::windef::HWND__, &mut process_id);

            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                FALSE,
                process_id,
            );

            if process_handle == 0 {
                return Ok("Unknown".to_string());
            }

            let mut module_name: [u16; 512] = [0; 512];
            let length = K32GetModuleBaseNameW(
                process_handle,
                0,
                module_name.as_mut_ptr(),
                512,
            );

            CloseHandle(process_handle);

            if length > 0 {
                Ok(String::from_utf16_lossy(&module_name[..length as usize]))
            } else {
                Ok("Unknown".to_string())
            }
        }
    }
    
    /// Get window class name
    fn get_window_class_name(&self, hwnd: HWND) -> Result<String> {
        let mut class_name: [u16; 256] = [0; 256];
        let length = unsafe { GetClassNameW(hwnd, class_name.as_mut_ptr(), 256) };
        
        if length > 0 {
            Ok(String::from_utf16_lossy(&class_name[..length as usize]))
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Check if window should be processed based on configuration
    fn should_process_window(&self, window_info: &WindowInfo) -> bool {
        // Always check previous focused window if tracking is enabled
        if self.config.track_previous_focus {
            if let Some(prev_hwnd) = self.previous_focused_window {
                if window_info.hwnd == prev_hwnd {
                    debug!("🎯 Processing previous focused window: {}", window_info.title);
                    return true;
                }
            }
        }

        if self.config.focused_only {
            let focused_hwnd = unsafe { GetForegroundWindow() };
            return window_info.hwnd == (focused_hwnd as isize);
        }

        // Check if window title or app name matches target applications
        for target_app in &self.config.target_apps {
            if window_info.title.to_lowercase().contains(&target_app.to_lowercase()) {
                return true;
            }
        }

        // Additional check for background windows with interesting class names
        if self.config.monitor_hidden_windows && window_info.title.starts_with("[Hidden]") {
            return true;
        }

        false
    }

    /// Check if content is new compared to last seen
    fn is_new_content(&mut self, text: &str) -> bool {
        let text_trimmed = text.trim();
        
        match &self.last_seen_text {
            Some(last) if last == text_trimmed => false,
            _ => {
                self.last_seen_text = Some(text_trimmed.to_string());
                true
            }
        }
    }

    /// Detect if text contains question patterns
    fn detect_question_patterns(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        
        for pattern in &self.question_patterns {
            if pattern.is_match(&text_lower) {
                debug!("✅ Question pattern detected: {}", pattern.as_str());
                return true;
            }
        }
        
        false
    }
}

impl Drop for WindowsAccessibilityReader {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

/// Window information structure
#[derive(Debug, Clone)]
struct WindowInfo {
    hwnd: HWND,
    title: String,
}

/// Initialize and create accessibility reader
pub fn create_accessibility_reader() -> Result<WindowsAccessibilityReader> {
    let config = AccessibilityConfig::default();
    WindowsAccessibilityReader::new(config)
}

/// Tauri command to read text from the current active window
#[tauri::command]
pub async fn read_text_from_current_window() -> Result<Option<AccessibilityTextResult>, String> {
    info!("🚀 Reading text from current active window...");
    
    let mut reader = create_accessibility_reader()
        .map_err(|e| format!("Failed to create accessibility reader: {}", e))?;
    
    let result = reader.read_text_from_current_window()
        .map_err(|e| format!("Failed to read current window: {}", e))?;
    
    match &result {
        Some(text_result) => {
            info!("✅ Current window text: {} chars from {}", 
                  text_result.text.len(), text_result.source_app);
        }
        None => {
            info!("ℹ️ No relevant text found in current window");
        }
    }
    
    Ok(result)
}

/// Tauri command to read text from applications (legacy - now uses current window)
#[tauri::command]
pub async fn read_text_from_applications() -> Result<Vec<AccessibilityTextResult>, String> {
    info!("🚀 Starting accessibility text reading from current window...");
    
    let mut reader = create_accessibility_reader()
        .map_err(|e| format!("Failed to create accessibility reader: {}", e))?;
    
    match reader.read_text_from_current_window()
        .map_err(|e| format!("Failed to read text: {}", e))? {
        Some(result) => {
            info!("✅ Text reading completed: 1 result from current window");
            Ok(vec![result])
        }
        None => {
            info!("ℹ️ No text found in current window");
            Ok(vec![])
        }
    }
}

/// Tauri command to read text from focused window
#[tauri::command]
pub async fn read_text_from_focused_window() -> Result<Option<AccessibilityTextResult>, String> {
    info!("🎯 Reading text from focused window...");
    
    let mut reader = create_accessibility_reader()
        .map_err(|e| format!("Failed to create accessibility reader: {}", e))?;
    
    let result = reader.read_text_from_focused_window()
        .map_err(|e| format!("Failed to read focused window: {}", e))?;
    
    match &result {
        Some(text_result) => {
            info!("✅ Focused window text: {} chars from {}", 
                  text_result.text.len(), text_result.source_app);
        }
        None => {
            info!("ℹ️ No text found in focused window");
        }
    }
    
    Ok(result)
}

/// Real-time text monitoring service
#[derive(Debug, Clone)]
pub struct RealtimeTextMonitor {
    app_handle: AppHandle,
    is_monitoring: Arc<Mutex<bool>>,
    config: AccessibilityConfig,
}

impl RealtimeTextMonitor {
    pub fn new(app_handle: AppHandle, config: AccessibilityConfig) -> Self {
        Self {
            app_handle,
            is_monitoring: Arc::new(Mutex::new(false)),
            config,
        }
    }
    
    /// Start real-time monitoring
    pub async fn start_monitoring(&self) -> Result<(), String> {
        info!("🚀 Starting real-time text monitoring...");
        
        // Set monitoring flag
        {
            let mut monitoring = self.is_monitoring.lock().map_err(|e| e.to_string())?;
            if *monitoring {
                return Err("Monitoring is already active".to_string());
            }
            *monitoring = true;
        }
        
        // Clone necessary data for the monitoring task
        let app_handle = self.app_handle.clone();
        let is_monitoring = self.is_monitoring.clone();
        let config = self.config.clone();
        
        // Start monitoring task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.monitoring_interval_ms));
            let mut last_seen_texts = std::collections::HashMap::new();
            
            loop {
                // Check if monitoring should continue
                {
                    let monitoring = match is_monitoring.lock() {
                        Ok(guard) => *guard,
                        Err(_) => break,
                    };
                    if !monitoring {
                        break;
                    }
                }
                
                // Wait for next interval
                interval.tick().await;
                
                // Read text from current window
                if let Ok(mut reader) = create_accessibility_reader() {
                    match reader.read_text_from_current_window() {
                        Ok(Some(result)) => {
                            // Check if this is new text
                            let key = format!("{}-{}", result.source_app, result.window_title);
                            let is_new = match last_seen_texts.get(&key) {
                                Some(last_text) => last_text != &result.text,
                                None => true,
                            };
                            
                            if is_new && result.is_potential_question {
                                info!("📝 Real-time question detected from current window {}: {}", 
                                      result.source_app, 
                                      result.text.chars().take(100).collect::<String>());
                                
                                // Emit event to frontend
                                if let Err(e) = app_handle.emit("accessibility-question-detected", &result) {
                                    warn!("Failed to emit question detection event: {}", e);
                                }
                                
                                // Update last seen text
                                last_seen_texts.insert(key, result.text);
                            }
                        }
                        Ok(None) => {
                            // No text found in current window - this is normal
                        }
                        Err(e) => {
                            debug!("Current window monitoring failed: {}", e);
                        }
                    }
                }
            }
            
            info!("🛑 Real-time text monitoring stopped");
        });
        
        Ok(())
    }
    
    /// Stop real-time monitoring
    pub fn stop_monitoring(&self) -> Result<(), String> {
        info!("🛑 Stopping real-time text monitoring...");
        
        let mut monitoring = self.is_monitoring.lock().map_err(|e| e.to_string())?;
        *monitoring = false;
        
        Ok(())
    }
    
    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.is_monitoring.lock().map(|guard| *guard).unwrap_or(false)
    }
}

// Global monitoring instance
static GLOBAL_MONITOR: std::sync::OnceLock<Arc<Mutex<Option<RealtimeTextMonitor>>>> = std::sync::OnceLock::new();

/// Initialize real-time monitoring
pub fn init_realtime_monitoring(app_handle: AppHandle) {
    let config = AccessibilityConfig::default();
    let monitor = RealtimeTextMonitor::new(app_handle, config);
    
    let _global_monitor = GLOBAL_MONITOR.get_or_init(|| {
        Arc::new(Mutex::new(Some(monitor)))
    });
    
    info!("✅ Real-time accessibility monitoring initialized");
}

/// Tauri command to start real-time monitoring
#[tauri::command]
pub async fn start_realtime_monitoring() -> Result<String, String> {
    info!("🚀 Starting real-time monitoring via command...");
    
    let global_monitor = GLOBAL_MONITOR.get()
        .ok_or("Monitoring not initialized")?;
    
    // Clone the monitor to avoid holding the lock across await
    let monitor = {
        let monitor_guard = global_monitor.lock().map_err(|e| e.to_string())?;
        monitor_guard.as_ref()
            .ok_or("Monitor not available")?
            .clone()
    };
    
    monitor.start_monitoring().await?;
    
    Ok("Real-time monitoring started".to_string())
}

/// Tauri command to stop real-time monitoring
#[tauri::command]
pub async fn stop_realtime_monitoring() -> Result<String, String> {
    info!("🛑 Stopping real-time monitoring via command...");
    
    let global_monitor = GLOBAL_MONITOR.get()
        .ok_or("Monitoring not initialized")?;
    
    // Clone the monitor to avoid holding the lock
    let monitor = {
        let monitor_guard = global_monitor.lock().map_err(|e| e.to_string())?;
        monitor_guard.as_ref()
            .ok_or("Monitor not available")?
            .clone()
    };
    
    monitor.stop_monitoring()?;
    
    Ok("Real-time monitoring stopped".to_string())
}

/// Tauri command to get monitoring status
#[tauri::command]
pub async fn get_monitoring_status() -> Result<serde_json::Value, String> {
    let global_monitor = GLOBAL_MONITOR.get()
        .ok_or("Monitoring not initialized")?;
    
    let is_active = {
        let monitor_guard = global_monitor.lock().map_err(|e| e.to_string())?;
        match monitor_guard.as_ref() {
            Some(monitor) => monitor.is_monitoring(),
            None => false,
        }
    };
    
    Ok(serde_json::json!({
        "is_monitoring": is_active,
        "interval_ms": AccessibilityConfig::default().monitoring_interval_ms,
        "target_apps": AccessibilityConfig::default().target_apps
    }))
}

/// Tauri command for hybrid text extraction (Accessibility + OCR fallback)
#[tauri::command]
pub async fn extract_text_hybrid_approach() -> Result<Vec<AccessibilityTextResult>, String> {
    info!("🔄 Starting hybrid text extraction (Accessibility + OCR fallback)...");
    
    // First try accessibility API
    let accessibility_results = match read_text_from_applications().await {
        Ok(results) if !results.is_empty() => {
            info!("✅ Accessibility API found {} results", results.len());
            results
        }
        _ => {
            info!("⚠️ Accessibility API found no results, trying OCR fallback...");
            Vec::new()
        }
    };
    
    // If no results from accessibility, try OCR as fallback
    if accessibility_results.is_empty() {
        // This would integrate with your existing screenshot + OCR functionality
        // For now, we'll return a placeholder that indicates OCR should be used
        info!("📸 Would trigger OCR screenshot analysis here...");
        
        // You can integrate this with your existing screenshot analysis
        // by calling the screenshot module and OCR processing
    }
    
    Ok(accessibility_results)
}

impl WindowsAccessibilityReader {
    /// Update the previously focused window
    pub fn update_previous_focused_window(&mut self) {
        let current_focused = unsafe { GetForegroundWindow() };
        if !current_focused.is_null() {
            self.previous_focused_window = Some(current_focused as isize);
            debug!("📝 Updated previous focused window: {}", current_focused as isize);
        }
    }

    /// Get text from background windows without affecting focus
    pub fn read_background_windows(&mut self) -> Result<Vec<AccessibilityTextResult>> {
        info!("🔍 Reading text from background windows...");
        
        // Store current focused window first
        self.update_previous_focused_window();
        
        let mut results = Vec::new();
        let windows = self.enumerate_windows()?;
        
        for window_info in windows {
            if self.should_process_background_window(&window_info) {
                // Try to extract text without changing window focus
                match self.extract_text_from_background_window(window_info.hwnd) {
                    Ok(Some(result)) => {
                        // Check against per-window cache to detect new content
                        let is_new = self.is_window_content_new(window_info.hwnd, &result.text);
                        if is_new {
                            info!("📝 New background content from {}: {}",
                                  result.source_app,
                                  result.text.chars().take(50).collect::<String>());
                            results.push(result);
                        }
                    }
                    Ok(None) => {
                        debug!("No text found in background window: {}", window_info.title);
                    }
                    Err(e) => {
                        debug!("Failed to read background window {}: {}", window_info.title, e);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Check if a background window should be processed
    fn should_process_background_window(&self, window_info: &WindowInfo) -> bool {
        // Don't process the currently focused window (MockMate)
        let current_focused = unsafe { GetForegroundWindow() };
        if window_info.hwnd == (current_focused as isize) {
            return false;
        }
        
        // Process previously focused window
        if self.config.track_previous_focus {
            if let Some(prev_hwnd) = self.previous_focused_window {
                if window_info.hwnd == prev_hwnd {
                    return true;
                }
            }
        }
        
        // Check target applications
        for target_app in &self.config.target_apps {
            if window_info.title.to_lowercase().contains(&target_app.to_lowercase()) {
                return true;
            }
        }
        
        // Check hidden windows with interesting classes
        if self.config.monitor_hidden_windows && window_info.title.starts_with("[Hidden]") {
            return true;
        }
        
        false
    }
    
    /// Extract text from background window using non-intrusive methods
    fn extract_text_from_background_window(&self, hwnd: HWND) -> Result<Option<AccessibilityTextResult>> {
        debug!("🔍 Background extraction from window: {}", hwnd);
        
        // Use only non-intrusive text extraction methods
        let mut extracted_texts = Vec::new();
        
        // Method 1: Child window enumeration (doesn't affect focus)
        if let Ok(child_texts) = self.extract_from_child_windows(hwnd) {
            extracted_texts.extend(child_texts);
        }
        
        // Method 2: SendMessage (non-intrusive)
        if let Ok(text) = self.extract_text_sendmessage(hwnd) {
            if !text.trim().is_empty() {
                extracted_texts.push(text);
            }
        }
        
        // Method 3: Windows accessibility without activation
        if let Ok(text) = self.extract_using_legacy_accessibility(hwnd) {
            if !text.trim().is_empty() {
                extracted_texts.push(text);
            }
        }
        
        let combined_text = extracted_texts.join(" ").trim().to_string();
        
        if combined_text.len() < self.config.min_question_length {
            return Ok(None);
        }
        
        let truncated_text = combined_text.chars().take(self.config.max_text_length).collect::<String>();
        let is_potential_question = self.detect_question_patterns(&truncated_text);
        
        if !is_potential_question {
            return Ok(None); // Only return questions for background monitoring
        }
        
        // Get window information
        let window_title = self.get_window_title(hwnd)?;
        let app_name = self.get_application_name(hwnd)?;
        let window_class = self.get_window_class_name(hwnd).unwrap_or_else(|_| "Unknown".to_string());
        
        let process_id = unsafe {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd as *mut winapi::shared::windef::HWND__, &mut pid);
            pid
        };
        
        let result = AccessibilityTextResult {
            text: truncated_text.clone(),
            source_app: app_name,
            window_title,
            confidence: 0.7, // Slightly lower confidence for background extraction
            is_potential_question: true,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            extraction_method: "Background_Monitoring".to_string(),
            window_class,
            process_id,
            text_length: truncated_text.len(),
        };
        
        Ok(Some(result))
    }
    
    /// Check if window content is new (per-window tracking)
    fn is_window_content_new(&mut self, hwnd: HWND, text: &str) -> bool {
        let text_trimmed = text.trim();
        
        match self.window_text_cache.get(&hwnd) {
            Some(cached_text) if cached_text == text_trimmed => false,
            _ => {
                self.window_text_cache.insert(hwnd, text_trimmed.to_string());
                true
            }
        }
    }
}

/// Tauri command to read from background windows
#[tauri::command]
pub async fn read_text_from_background_windows() -> Result<Vec<AccessibilityTextResult>, String> {
    info!("🔍 Reading text from background windows...");
    
    let mut reader = create_accessibility_reader()
        .map_err(|e| format!("Failed to create accessibility reader: {}", e))?;
    
    let results = reader.read_background_windows()
        .map_err(|e| format!("Failed to read background windows: {}", e))?;
    
    info!("✅ Background window reading completed: {} results", results.len());
    Ok(results)
}

/// Configuration update command
#[tauri::command]
pub async fn update_accessibility_config(
    target_apps: Option<Vec<String>>,
    focused_only: Option<bool>,
    min_question_length: Option<usize>,
    monitoring_interval_ms: Option<u64>,
    track_previous_focus: Option<bool>,
    monitor_hidden_windows: Option<bool>,
    allow_window_activation: Option<bool>
) -> Result<String, String> {
    info!("⚙️ Updating accessibility configuration...");
    
    // Note: In a real implementation, you'd want to store this config
    // and apply it to the global monitor. For now, we'll just log the changes.
    
    if let Some(apps) = target_apps {
        info!("📱 Updated target apps: {:?}", apps);
    }
    if let Some(focused) = focused_only {
        info!("🎯 Updated focused_only: {}", focused);
    }
    if let Some(min_len) = min_question_length {
        info!("📏 Updated min_question_length: {}", min_len);
    }
    if let Some(interval) = monitoring_interval_ms {
        info!("⏱️ Updated monitoring_interval_ms: {}", interval);
    }
    if let Some(track_focus) = track_previous_focus {
        info!("👁️ Updated track_previous_focus: {}", track_focus);
    }
    if let Some(monitor_hidden) = monitor_hidden_windows {
        info!("👻 Updated monitor_hidden_windows: {}", monitor_hidden);
    }
    if let Some(allow_activation) = allow_window_activation {
        info!("🎯 Updated allow_window_activation: {}", allow_activation);
    }
    
    Ok("Configuration updated successfully".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_detection() {
        let config = AccessibilityConfig::default();
        let reader = WindowsAccessibilityReader::new(config).unwrap();
        
        assert!(reader.detect_question_patterns("What is your experience with React?"));
        assert!(reader.detect_question_patterns("Can you explain how REST APIs work?"));
        assert!(reader.detect_question_patterns("Tell me about your background"));
        assert!(!reader.detect_question_patterns("This is just a statement."));
    }
}
