// Stealth Mode Hotkey System for MockMate
// Implements global hotkeys to avoid mouse cursor visibility during interviews

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use windows_sys::{
    Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Win32::UI::WindowsAndMessaging::{
        RegisterHotKey, UnregisterHotKey, MSG, GetMessageW, TranslateMessage, DispatchMessageW,
        DefWindowProcW, MOD_SHIFT, WM_HOTKEY
    },
    Win32::UI::Input::KeyboardAndMouse::{
        VK_S, VK_Z, VK_X, VK_M, VK_A, VK_I, VK_C, VK_RETURN
    }
};
use log::{info, error, warn};
use anyhow::Result;

/// Stealth hotkey identifiers
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum StealthHotkey {
    SystemSound = 1,    // Shift+S
    AIAnswer = 2,       // Shift+Z  
    WindowToggle = 3,   // Shift+X
    MicToggle = 4,      // Shift+M
    AnalyzeScreen = 5,  // Shift+A
    ManualInput = 6,    // Shift+I
    SubmitQuestion = 7, // Enter
    ClearArea = 8,      // Shift+C
}

impl StealthHotkey {
    pub fn to_string(&self) -> &'static str {
        match self {
            StealthHotkey::SystemSound => "system_sound_toggle",
            StealthHotkey::AIAnswer => "ai_answer_trigger",
            StealthHotkey::WindowToggle => "window_toggle",
            StealthHotkey::MicToggle => "mic_toggle",
            StealthHotkey::AnalyzeScreen => "analyze_screen",
            StealthHotkey::ManualInput => "manual_input",
            StealthHotkey::SubmitQuestion => "submit_question",
            StealthHotkey::ClearArea => "clear_area",
        }
    }
}

/// Global hotkey manager for stealth mode
pub struct StealthHotkeyManager {
    app_handle: AppHandle,
    registered_hotkeys: Arc<Mutex<HashMap<u32, StealthHotkey>>>,
    is_active: Arc<Mutex<bool>>,
}

impl StealthHotkeyManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            registered_hotkeys: Arc::new(Mutex::new(HashMap::new())),
            is_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Activate stealth mode with global hotkeys
    pub fn activate_stealth_mode(&self) -> Result<()> {
        info!("🕵️ Activating stealth mode hotkeys...");
        
        let mut is_active = self.is_active.lock().unwrap();
        if *is_active {
            warn!("Stealth mode already active");
            return Ok(());
        }

        let hwnd = self.get_window_handle()?;
        let mut registered = self.registered_hotkeys.lock().unwrap();

        // Register all stealth hotkeys
        let hotkeys = vec![
            (StealthHotkey::SystemSound, MOD_SHIFT, VK_S),      // Shift+S
            (StealthHotkey::AIAnswer, MOD_SHIFT, VK_Z),         // Shift+Z
            (StealthHotkey::WindowToggle, MOD_SHIFT, VK_X),     // Shift+X
            (StealthHotkey::MicToggle, MOD_SHIFT, VK_M),        // Shift+M
            (StealthHotkey::AnalyzeScreen, MOD_SHIFT, VK_A),    // Shift+A
            (StealthHotkey::ManualInput, MOD_SHIFT, VK_I),      // Shift+I
            (StealthHotkey::SubmitQuestion, 0, VK_RETURN),      // Enter
            (StealthHotkey::ClearArea, MOD_SHIFT, VK_C),        // Shift+C
        ];

        for (hotkey, modifiers, key) in hotkeys {
            unsafe {
                let result = RegisterHotKey(
                    hwnd,
                    hotkey as i32,
                    modifiers,
                    key as u32
                );
                
                if result != 0 {
                    registered.insert(hotkey as u32, hotkey);
                    info!("✅ Registered hotkey: {} ({})", hotkey.to_string(), hotkey as u32);
                } else {
                    error!("❌ Failed to register hotkey: {}", hotkey.to_string());
                }
            }
        }

        *is_active = true;
        info!("✅ Stealth mode activated with {} hotkeys", registered.len());

        // Start message loop in background
        self.start_message_loop()?;

        Ok(())
    }

    /// Deactivate stealth mode and unregister hotkeys
    pub fn deactivate_stealth_mode(&self) -> Result<()> {
        info!("🔓 Deactivating stealth mode...");
        
        let mut is_active = self.is_active.lock().unwrap();
        if !*is_active {
            warn!("Stealth mode not active");
            return Ok(());
        }

        let hwnd = self.get_window_handle()?;
        let mut registered = self.registered_hotkeys.lock().unwrap();

        // Unregister all hotkeys
        for &hotkey_id in registered.keys() {
            unsafe {
                let result = UnregisterHotKey(hwnd, hotkey_id as i32);
                if result != 0 {
                    info!("✅ Unregistered hotkey: {}", hotkey_id);
                } else {
                    error!("❌ Failed to unregister hotkey: {}", hotkey_id);
                }
            }
        }

        registered.clear();
        *is_active = false;
        info!("✅ Stealth mode deactivated");

        Ok(())
    }

    /// Get window handle for hotkey registration
    fn get_window_handle(&self) -> Result<HWND> {
        // For now, use null HWND for global hotkeys
        // In production, you might want to get the actual window handle
        Ok(0)
    }

    /// Start Windows message loop for hotkey handling
    fn start_message_loop(&self) -> Result<()> {
        let app_handle = self.app_handle.clone();
        let registered_hotkeys = self.registered_hotkeys.clone();
        let is_active = self.is_active.clone();

        // Start message loop in separate thread
        std::thread::spawn(move || {
            info!("🔄 Starting stealth hotkey message loop...");
            
            unsafe {
                let mut msg: MSG = std::mem::zeroed();
                
                while *is_active.lock().unwrap() {
                    let result = GetMessageW(&mut msg, 0, 0, 0);
                    
                    if result > 0 {
                        if msg.message == WM_HOTKEY {
                            Self::handle_hotkey_message(&app_handle, &registered_hotkeys, &msg);
                        }
                        
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    } else if result == 0 {
                        // WM_QUIT received
                        break;
                    } else {
                        // Error occurred
                        error!("Error in message loop: {}", result);
                        break;
                    }
                }
            }
            
            info!("✅ Stealth hotkey message loop ended");
        });

        Ok(())
    }

    /// Handle hotkey message and emit to frontend
    fn handle_hotkey_message(
        app_handle: &AppHandle,
        registered_hotkeys: &Arc<Mutex<HashMap<u32, StealthHotkey>>>,
        msg: &MSG
    ) {
        let hotkey_id = msg.wParam as u32;
        let registered = registered_hotkeys.lock().unwrap();
        
        if let Some(&hotkey) = registered.get(&hotkey_id) {
            info!("🎯 Stealth hotkey triggered: {} ({})", hotkey.to_string(), hotkey_id);
            
            // Emit to frontend without any visual indication
            let _ = app_handle.emit("stealth-hotkey", serde_json::json!({
                "action": hotkey.to_string(),
                "hotkey_id": hotkey_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
            
            // Log for debugging (remove in production)
            info!("📡 Emitted stealth hotkey event: {}", hotkey.to_string());
        } else {
            warn!("Unknown hotkey ID received: {}", hotkey_id);
        }
    }

    /// Check if stealth mode is active
    pub fn is_stealth_active(&self) -> bool {
        *self.is_active.lock().unwrap()
    }

    /// Get list of registered hotkeys for debugging
    pub fn get_registered_hotkeys(&self) -> Vec<String> {
        let registered = self.registered_hotkeys.lock().unwrap();
        registered.values()
            .map(|hotkey| hotkey.to_string().to_string())
            .collect()
    }
}

/// Global instance of stealth hotkey manager
static STEALTH_MANAGER: std::sync::OnceLock<Mutex<Option<StealthHotkeyManager>>> = std::sync::OnceLock::new();

fn get_stealth_manager() -> &'static Mutex<Option<StealthHotkeyManager>> {
    STEALTH_MANAGER.get_or_init(|| Mutex::new(None))
}

/// Initialize stealth hotkey system
pub fn init_stealth_hotkeys(app_handle: AppHandle) {
    let manager = get_stealth_manager();
    let mut guard = manager.lock().unwrap();
    *guard = Some(StealthHotkeyManager::new(app_handle));
    info!("✅ Stealth hotkey system initialized");
}

/// Tauri command to activate stealth mode
#[tauri::command]
pub async fn activate_stealth_mode() -> Result<String, String> {
    let manager = get_stealth_manager();
    let guard = manager.lock().unwrap();
    
    if let Some(ref stealth_manager) = *guard {
        stealth_manager.activate_stealth_mode()
            .map_err(|e| e.to_string())?;
        Ok("Stealth mode activated - all hotkeys registered".to_string())
    } else {
        Err("Stealth manager not initialized".to_string())
    }
}

/// Tauri command to deactivate stealth mode
#[tauri::command]
pub async fn deactivate_stealth_mode() -> Result<String, String> {
    let manager = get_stealth_manager();
    let guard = manager.lock().unwrap();
    
    if let Some(ref stealth_manager) = *guard {
        stealth_manager.deactivate_stealth_mode()
            .map_err(|e| e.to_string())?;
        Ok("Stealth mode deactivated - all hotkeys unregistered".to_string())
    } else {
        Err("Stealth manager not initialized".to_string())
    }
}

/// Tauri command to check stealth status
#[tauri::command]
pub async fn get_stealth_status() -> Result<serde_json::Value, String> {
    let manager = get_stealth_manager();
    let guard = manager.lock().unwrap();
    
    if let Some(ref stealth_manager) = *guard {
        let status = serde_json::json!({
            "active": stealth_manager.is_stealth_active(),
            "registered_hotkeys": stealth_manager.get_registered_hotkeys(),
            "hotkey_mappings": {
                "Shift+Ctrl+S": "System Sound Toggle",
                "Shift+Ctrl+Z": "AI Answer Trigger", 
                "Shift+Ctrl+X": "Window Hide/Show",
                "Shift+Ctrl+M": "Microphone Toggle",
                "Shift+Ctrl+A": "Analyze Screen",
                "Shift+Ctrl+I": "Manual Question Entry",
                "Shift+Ctrl+Enter": "Submit Question",
                "Shift+Ctrl+C": "Clear Listening Area"
            }
        });
        Ok(status)
    } else {
        Err("Stealth manager not initialized".to_string())
    }
}

/// Tauri command to test hotkey functionality
#[tauri::command]
pub async fn test_stealth_hotkey(hotkey_name: String) -> Result<String, String> {
    info!("🧪 Testing stealth hotkey: {}", hotkey_name);
    
    // This would normally be triggered by actual hotkey press
    // For testing, we simulate the event
    let test_result = match hotkey_name.as_str() {
        "system_sound_toggle" => "System sound toggled (test)",
        "ai_answer_trigger" => "AI answer triggered (test)",
        "window_toggle" => "Window visibility toggled (test)",
        "mic_toggle" => "Microphone toggled (test)",
        "analyze_screen" => "Screen analysis triggered (test)",
        "manual_input" => "Manual input mode activated (test)",
        "submit_question" => "Question submitted (test)",
        "clear_area" => "Listening area cleared (test)",
        _ => "Unknown hotkey (test)"
    };
    
    Ok(test_result.to_string())
}
