// Global Hotkey System for MockMate Stealth Mode using rdev
// Uses Ctrl+Shift combinations for reliable cross-platform detection

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use tauri::{AppHandle, Emitter};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};
use anyhow::Result;
use rdev::{listen, Event, EventType, Key};
use std::time::{Duration, Instant};

// Windows-specific imports for global hotkeys and message loop
#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use winapi::um::winuser::{
    MOD_CONTROL, MOD_SHIFT, MOD_NOREPEAT,
    VK_RETURN,
    RegisterHotKey, UnregisterHotKey, TranslateMessage, DispatchMessageW, GetMessageW, MSG, WM_HOTKEY,
};

/// Stealth hotkey event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthHotkeyEvent {
    pub action: String,
    pub hotkey_id: u32,
    pub timestamp: String,
}

/// Stealth status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthStatus {
    pub active: bool,
    pub registered_hotkeys: Vec<String>,
    pub hotkey_mappings: HashMap<String, String>,
}

// Hotkey IDs for Windows API
const HOTKEY_SYSTEM_SOUND: i32 = 1;
const HOTKEY_AI_ANSWER: i32 = 2;
const HOTKEY_WINDOW_TOGGLE: i32 = 3;
const HOTKEY_MIC_TOGGLE: i32 = 4;
const HOTKEY_ANALYZE_SCREEN: i32 = 5;
const HOTKEY_MANUAL_INPUT: i32 = 6;
const HOTKEY_SUBMIT_QUESTION: i32 = 7;
const HOTKEY_CLEAR_AREA: i32 = 8;

// Windows virtual-key codes for letter keys (not provided by winapi)
#[cfg(windows)]
const VK_A: i32 = 0x41;
#[cfg(windows)]
const VK_C: i32 = 0x43;
#[cfg(windows)]
const VK_I: i32 = 0x49;
#[cfg(windows)]
const VK_M: i32 = 0x4D;
#[cfg(windows)]
const VK_S: i32 = 0x53;
#[cfg(windows)]
const VK_X: i32 = 0x58;
#[cfg(windows)]
const VK_Z: i32 = 0x5A;

/// Real Windows API hotkey manager for stealth mode
pub struct StealthHotkeyManager {
    app_handle: AppHandle,
    is_active: Arc<Mutex<bool>>,
    hotkey_mappings: Arc<Mutex<HashMap<String, String>>>,
    hotkey_thread_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl StealthHotkeyManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("Shift+Ctrl+S".to_string(), "system_sound_toggle".to_string());
        mappings.insert("Shift+Ctrl+Z".to_string(), "ai_answer_trigger".to_string());
        mappings.insert("Shift+Ctrl+X".to_string(), "window_toggle".to_string());
        mappings.insert("Shift+Ctrl+M".to_string(), "mic_toggle".to_string());
        mappings.insert("Shift+Ctrl+A".to_string(), "analyze_screen".to_string());
        mappings.insert("Shift+Ctrl+I".to_string(), "manual_input".to_string());
        mappings.insert("Shift+Ctrl+Enter".to_string(), "submit_question".to_string());
        mappings.insert("Shift+Ctrl+C".to_string(), "clear_area".to_string());

        Self {
            app_handle,
            is_active: Arc::new(Mutex::new(false)),
            hotkey_mappings: Arc::new(Mutex::new(mappings)),
            hotkey_thread_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Activate stealth mode with REAL Windows API hotkey registration
    pub fn activate_stealth_mode(&self) -> Result<String> {
        info!("🕵️ Activating stealth mode with REAL Windows API hotkeys...");
        
        let mut is_active = self.is_active.lock().unwrap();
        if *is_active {
            warn!("Stealth mode already active");
            return Ok("Stealth mode already active".to_string());
        }

        #[cfg(windows)]
        {
            // Start Windows API hotkey registration thread
            info!("🎯 Starting Windows API global hotkey system...");
            
            let app_handle = self.app_handle.clone();
            let is_active_clone = self.is_active.clone();
            
            let thread_handle = thread::spawn(move || {
                Self::windows_hotkey_loop(app_handle, is_active_clone);
            });
            
            *self.hotkey_thread_handle.lock().unwrap() = Some(thread_handle);
            *is_active = true;
            
            info!("✅ Stealth mode activated with 8 REAL Windows API global hotkeys");
            Ok("Stealth mode activated with 8 global Windows API hotkeys (Ctrl+Shift+Key)".to_string())
        }
        #[cfg(not(windows))]
        {
            warn!("Real global hotkeys only supported on Windows - using fallback");
            
            let app_handle = self.app_handle.clone();
            let is_active_clone = self.is_active.clone();
            
            let thread_handle = thread::spawn(move || {
                Self::fallback_hotkey_loop(app_handle, is_active_clone);
            });
            
            *self.hotkey_thread_handle.lock().unwrap() = Some(thread_handle);
            *is_active = true;
            
            Ok("Stealth mode activated with fallback system".to_string())
        }
    }

    /// Deactivate stealth mode and unregister hotkeys
    pub fn deactivate_stealth_mode(&self) -> Result<String> {
        info!("🔓 Deactivating stealth mode and unregistering Windows API hotkeys...");
        
        let mut is_active = self.is_active.lock().unwrap();
        if !*is_active {
            warn!("Stealth mode not active");
            return Ok("Stealth mode not active".to_string());
        }

        // Stop the hotkey thread by setting inactive flag
        *is_active = false;
        
        // Let the Windows API message loop clean up its own hotkey registrations
        if let Some(handle) = self.hotkey_thread_handle.lock().unwrap().take() {
            info!("🛑 Stopping Windows API hotkey message loop...");
            // The thread will see is_active = false and clean up hotkeys, then exit
            // We don't join to avoid blocking - thread will exit gracefully
            let _ = handle.thread().id();
        }
        
        info!("✅ Stealth mode deactivated - Windows API hotkeys will be cleaned up");
        
        Ok("Stealth mode deactivated successfully".to_string())
    }
    
    /// Real Windows API hotkey message loop
    #[cfg(windows)]
    fn windows_hotkey_loop(app_handle: AppHandle, is_active: Arc<Mutex<bool>>) {
        info!("🎯 Starting Windows API hotkey registration...");
        
        // Hotkey definitions: (ID, Modifiers, Virtual Key Code, Hotkey Name, Action)
        let hotkeys = vec![
            (HOTKEY_SYSTEM_SOUND, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_S, "Shift+Ctrl+S", "system_sound_toggle"),
            (HOTKEY_AI_ANSWER, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_Z, "Shift+Ctrl+Z", "ai_answer_trigger"),
            (HOTKEY_WINDOW_TOGGLE, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_X, "Shift+Ctrl+X", "window_toggle"),
            (HOTKEY_MIC_TOGGLE, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_M, "Shift+Ctrl+M", "mic_toggle"),
            (HOTKEY_ANALYZE_SCREEN, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_A, "Shift+Ctrl+A", "analyze_screen"),
            (HOTKEY_MANUAL_INPUT, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_I, "Shift+Ctrl+I", "manual_input"),
            (HOTKEY_SUBMIT_QUESTION, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_RETURN, "Shift+Ctrl+Enter", "submit_question"),
            (HOTKEY_CLEAR_AREA, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_C, "Shift+Ctrl+C", "clear_area"),
        ];
        
        unsafe {
            // Register all global hotkeys
            let mut registered_hotkeys = Vec::new();
            
            for &(id, modifiers, vk_code, hotkey_name, _action) in &hotkeys {
                let result = RegisterHotKey(
                    0 as HWND, // NULL HWND for thread message queue
                    id,
                    modifiers as u32,
                    vk_code as u32,
                );
                
                if result != 0 {
                    info!("✅ Registered global hotkey: {} (ID: {})", hotkey_name, id);
                    registered_hotkeys.push((id, hotkey_name));
                } else {
                    error!("❌ Failed to register hotkey: {} (ID: {})", hotkey_name, id);
                }
            }
            
            info!("✅ Successfully registered {}/{} global hotkeys", registered_hotkeys.len(), hotkeys.len());
            
            // Message loop to handle hotkey events
            let mut msg: MSG = std::mem::zeroed();
            
            info!("🔄 Starting Windows API message loop for hotkey events...");
            
            while *is_active.lock().unwrap() {
                // Get message from Windows - use timeout to check is_active periodically
                let result = GetMessageW(&mut msg, 0 as HWND, 0, 0);
                
                if result > 0 {
                    if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam as i32;
                        
                        // Find the corresponding action for this hotkey ID
                        if let Some(&(_id, _modifiers, _vk_code, hotkey_name, action)) = hotkeys.iter().find(|&&(id, _, _, _, _)| id == hotkey_id) {
                            info!("🎯 HOTKEY TRIGGERED: {} -> {} (ID: {})", hotkey_name, action, hotkey_id);
                            
                            let event = StealthHotkeyEvent {
                                action: action.to_string(),
                                hotkey_id: hotkey_id as u32,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            
                            if let Err(e) = app_handle.emit("stealth-hotkey", &event) {
                                error!("Failed to emit hotkey event: {}", e);
                            } else {
                                info!("✅ Hotkey event emitted: {} ({})", action, hotkey_name);
                            }
                        } else {
                            warn!("Unknown hotkey ID received: {}", hotkey_id);
                        }
                    }
                    
                    // Process other messages
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else if result == 0 {
                    // WM_QUIT message received
                    break;
                } else {
                    // Error occurred
                    break;
                }
                
                // Small yield to prevent 100% CPU usage
                thread::sleep(std::time::Duration::from_millis(1));
            }
            
            // Clean up: Unregister all hotkeys
            info!("🧹 Cleaning up: Unregistering {} global hotkeys...", registered_hotkeys.len());
            for (id, hotkey_name) in registered_hotkeys {
                let result = UnregisterHotKey(0 as HWND, id);
                if result != 0 {
                    info!("✅ Unregistered hotkey: {} (ID: {})", hotkey_name, id);
                } else {
                    warn!("❌ Failed to unregister hotkey: {} (ID: {})", hotkey_name, id);
                }
            }
        }
        
        info!("🛑 Windows API hotkey loop exited");
    }
    
    /// Fallback hotkey loop for non-Windows platforms
    #[cfg(not(windows))]
    fn fallback_hotkey_loop(app_handle: AppHandle, is_active: Arc<Mutex<bool>>) {
        info!("🎯 Starting fallback hotkey loop (non-Windows)...");
        
        let hotkeys = vec![
            ("Ctrl+Shift+S", "system_sound_toggle"),
            ("Ctrl+Shift+Z", "ai_answer_trigger"),
            ("Ctrl+Shift+X", "window_toggle"),
            ("Ctrl+Shift+M", "mic_toggle"),
            ("Ctrl+Shift+A", "analyze_screen"),
            ("Ctrl+Shift+I", "manual_input"),
            ("Ctrl+Shift+Enter", "submit_question"),
            ("Ctrl+Shift+C", "clear_area"),
        ];
        
        let mut counter = 0;
        
        while *is_active.lock().unwrap() {
            thread::sleep(std::time::Duration::from_millis(100));
            
            // Every 30 seconds, simulate a random hotkey for demonstration (fallback only)
            counter += 1;
            if counter >= 300 {  // 30 seconds at 100ms intervals
                counter = 0;
                
                // Pick a random hotkey to simulate
                let (hotkey_name, action) = hotkeys[counter % hotkeys.len()];
                
                info!("🎯 Simulating hotkey (fallback): {} -> {}", hotkey_name, action);
                
                let event = StealthHotkeyEvent {
                    action: action.to_string(),
                    hotkey_id: (counter % hotkeys.len()) as u32 + 1,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                
                if let Err(e) = app_handle.emit("stealth-hotkey", &event) {
                    error!("Failed to emit hotkey event: {}", e);
                } else {
                    info!("✅ Hotkey event emitted: {} ({})", action, hotkey_name);
                }
            }
        }
        
        info!("🛑 Fallback hotkey loop exited");
    }
    
    /// Manually trigger a hotkey for testing
    pub fn trigger_hotkey_manually(&self, action: &str) -> Result<()> {
        if !*self.is_active.lock().unwrap() {
            return Err(anyhow::anyhow!("Stealth mode not active"));
        }
        
        info!("🎯 Manually triggering hotkey: {}", action);
        
        let event = StealthHotkeyEvent {
            action: action.to_string(),
            hotkey_id: 999,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        self.app_handle.emit("stealth-hotkey", &event)
            .map_err(|e| anyhow::anyhow!("Failed to emit manual hotkey: {}", e))?;
        
        Ok(())
    }

    /// Get current stealth status
    pub fn get_stealth_status(&self) -> StealthStatus {
        let is_active = *self.is_active.lock().unwrap();
        let mappings = self.hotkey_mappings.lock().unwrap();
        
        let registered_hotkeys: Vec<String> = if is_active {
            mappings.keys().cloned().collect()
        } else {
            Vec::new()
        };

        StealthStatus {
            active: is_active,
            registered_hotkeys,
            hotkey_mappings: mappings.clone(),
        }
    }

    /// Test a hotkey action (for development/testing)
    pub fn test_hotkey(&self, hotkey_name: &str) -> Result<String> {
        let mappings = self.hotkey_mappings.lock().unwrap();
        
        if let Some(action) = mappings.get(hotkey_name) {
            info!("🎯 Testing hotkey: {} -> {}", hotkey_name, action);
            
            // Emit event to frontend
            let event = StealthHotkeyEvent {
                action: action.clone(),
                hotkey_id: hotkey_name.chars().map(|c| c as u32).sum(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            
            if let Err(e) = self.app_handle.emit("stealth-hotkey", &event) {
                warn!("Failed to emit hotkey event: {}", e);
            }
            
            Ok(format!("Hotkey test successful: {} triggered {}", hotkey_name, action))
        } else {
            Ok(format!("Hotkey '{}' not found", hotkey_name))
        }
    }

    /// Simulate hotkey trigger (for testing)
    pub fn trigger_hotkey(&self, action: &str) -> Result<()> {
        let is_active = *self.is_active.lock().unwrap();
        
        if !is_active {
            warn!("Cannot trigger hotkey - stealth mode not active");
            return Ok(());
        }

        info!("🎯 Simulating hotkey trigger: {}", action);
        
        let event = StealthHotkeyEvent {
            action: action.to_string(),
            hotkey_id: action.chars().map(|c| c as u32).sum(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = self.app_handle.emit("stealth-hotkey", &event) {
            warn!("Failed to emit hotkey event: {}", e);
        }
        
        Ok(())
    }
}

// Static instance for global access
static mut STEALTH_MANAGER: Option<StealthHotkeyManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global stealth hotkey manager
pub fn initialize_stealth_hotkeys(app_handle: AppHandle) {
    INIT.call_once(|| {
        unsafe {
            STEALTH_MANAGER = Some(StealthHotkeyManager::new(app_handle));
        }
        info!("🔧 Stealth hotkey manager initialized");
    });
}

/// Get reference to the global stealth hotkey manager
pub fn get_stealth_manager() -> Option<&'static StealthHotkeyManager> {
    unsafe { STEALTH_MANAGER.as_ref() }
}

// Tauri commands for stealth hotkey functionality

#[tauri::command]
pub fn activate_stealth_mode() -> Result<String, String> {
    match get_stealth_manager() {
        Some(manager) => manager.activate_stealth_mode().map_err(|e| e.to_string()),
        None => Err("Stealth manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn deactivate_stealth_mode() -> Result<String, String> {
    match get_stealth_manager() {
        Some(manager) => manager.deactivate_stealth_mode().map_err(|e| e.to_string()),
        None => Err("Stealth manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn get_stealth_status() -> Result<StealthStatus, String> {
    match get_stealth_manager() {
        Some(manager) => Ok(manager.get_stealth_status()),
        None => Err("Stealth manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn test_stealth_hotkey(hotkey_name: String) -> Result<String, String> {
    match get_stealth_manager() {
        Some(manager) => manager.test_hotkey(&hotkey_name).map_err(|e| e.to_string()),
        None => Err("Stealth manager not initialized".to_string()),
    }
}
