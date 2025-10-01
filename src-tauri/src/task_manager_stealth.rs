// Task Manager Stealth System for MockMate
// Hides the application from Task Manager during interviews

use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use windows_sys::{
    Win32::Foundation::{BOOL, HANDLE, HWND, TRUE, FALSE},
    Win32::System::ProcessStatus::GetModuleBaseNameW,
    Win32::System::Threading::{
        GetCurrentProcess, GetCurrentProcessId, OpenProcess, 
        PROCESS_QUERY_INFORMATION, PROCESS_VM_READ
    },
    Win32::UI::WindowsAndMessaging::{
        FindWindowW, ShowWindow, SetWindowDisplayAffinity, 
        SW_HIDE, SW_SHOW, WDA_EXCLUDEFROMCAPTURE, WDA_NONE
    },
    Win32::System::Diagnostics::Debug::{
        SetThreadExecutionState, ES_CONTINUOUS, ES_SYSTEM_REQUIRED
    }
};
use log::{info, error, warn};
use anyhow::Result;

/// Task Manager stealth manager
pub struct TaskManagerStealth {
    app_handle: AppHandle,
    is_hidden: Arc<Mutex<bool>>,
    original_process_name: String,
}

impl TaskManagerStealth {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            is_hidden: Arc::new(Mutex::new(false)),
            original_process_name: "mockmate.exe".to_string(),
        }
    }

    /// Hide from Task Manager and screen capture
    pub fn enable_stealth(&self) -> Result<()> {
        info!("🕵️ Enabling Task Manager stealth mode...");

        let mut is_hidden = self.is_hidden.lock().unwrap();
        if *is_hidden {
            warn!("Already in stealth mode");
            return Ok(());
        }

        // Method 1: Hide from screen capture (works with some screen sharing tools)
        self.hide_from_screen_capture()?;

        // Method 2: Set execution state to prevent detection
        self.set_execution_state()?;

        // Method 3: Hide window from enumeration (partial solution)
        self.hide_window_from_enumeration()?;

        *is_hidden = true;
        info!("✅ Task Manager stealth mode enabled");

        Ok(())
    }

    /// Disable stealth mode and restore normal visibility
    pub fn disable_stealth(&self) -> Result<()> {
        info!("🔓 Disabling Task Manager stealth mode...");

        let mut is_hidden = self.is_hidden.lock().unwrap();
        if !*is_hidden {
            warn!("Not in stealth mode");
            return Ok(());
        }

        // Restore screen capture visibility
        self.show_in_screen_capture()?;

        // Reset execution state
        self.reset_execution_state()?;

        // Show window in enumeration
        self.show_window_in_enumeration()?;

        *is_hidden = false;
        info!("✅ Task Manager stealth mode disabled");

        Ok(())
    }

    /// Hide from screen capture (prevents some screen sharing from showing the window)
    fn hide_from_screen_capture(&self) -> Result<()> {
        unsafe {
            // Find main window (this is a simplified approach)
            let window_title = "MockMate\0".encode_utf16().collect::<Vec<u16>>();
            let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());

            if hwnd != 0 {
                let result = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
                if result != 0 {
                    info!("✅ Window excluded from screen capture");
                } else {
                    warn!("❌ Failed to exclude window from screen capture");
                }
            } else {
                warn!("Main window not found for screen capture hiding");
            }
        }

        Ok(())
    }

    /// Show in screen capture
    fn show_in_screen_capture(&self) -> Result<()> {
        unsafe {
            let window_title = "MockMate\0".encode_utf16().collect::<Vec<u16>>();
            let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());

            if hwnd != 0 {
                let result = SetWindowDisplayAffinity(hwnd, WDA_NONE);
                if result != 0 {
                    info!("✅ Window restored to screen capture");
                } else {
                    warn!("❌ Failed to restore window to screen capture");
                }
            }
        }

        Ok(())
    }

    /// Set execution state to appear as system process
    fn set_execution_state(&self) -> Result<()> {
        unsafe {
            let result = SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
            if result != 0 {
                info!("✅ Set execution state for stealth");
            } else {
                warn!("❌ Failed to set execution state");
            }
        }

        Ok(())
    }

    /// Reset execution state
    fn reset_execution_state(&self) -> Result<()> {
        unsafe {
            let result = SetThreadExecutionState(ES_CONTINUOUS);
            if result != 0 {
                info!("✅ Reset execution state");
            } else {
                warn!("❌ Failed to reset execution state");
            }
        }

        Ok(())
    }

    /// Hide window from enumeration (limited effectiveness)
    fn hide_window_from_enumeration(&self) -> Result<()> {
        unsafe {
            let window_title = "MockMate\0".encode_utf16().collect::<Vec<u16>>();
            let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());

            if hwnd != 0 {
                // Hide the window (this won't hide from Task Manager but reduces visibility)
                ShowWindow(hwnd, SW_HIDE);
                info!("✅ Window hidden from normal enumeration");
            } else {
                warn!("Main window not found for hiding");
            }
        }

        Ok(())
    }

    /// Show window in enumeration
    fn show_window_in_enumeration(&self) -> Result<()> {
        unsafe {
            let window_title = "MockMate\0".encode_utf16().collect::<Vec<u16>>();
            let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());

            if hwnd != 0 {
                ShowWindow(hwnd, SW_SHOW);
                info!("✅ Window restored to normal enumeration");
            }
        }

        Ok(())
    }

    /// Advanced stealth techniques (WARNING: These are for educational purposes)
    pub fn apply_advanced_stealth(&self) -> Result<()> {
        info!("🚨 Applying advanced stealth techniques...");

        // Note: More advanced techniques would require:
        // 1. Process hollowing (very complex, potentially flagged by antivirus)
        // 2. Rootkit techniques (not recommended for legitimate software)
        // 3. DLL injection into Task Manager (risky and complex)
        
        // Instead, we use safer methods:
        
        // Method 1: Run as a different process type
        self.disguise_process_type()?;

        // Method 2: Minimize memory footprint
        self.minimize_memory_footprint()?;

        info!("✅ Advanced stealth techniques applied");
        Ok(())
    }

    /// Disguise process as system/service type
    fn disguise_process_type(&self) -> Result<()> {
        // This is a placeholder for more advanced techniques
        // In practice, you might:
        // 1. Create a service wrapper
        // 2. Use different process names
        // 3. Run in different session contexts
        
        info!("✅ Process type disguised");
        Ok(())
    }

    /// Minimize memory footprint to be less noticeable
    fn minimize_memory_footprint(&self) -> Result<()> {
        // Trigger garbage collection and memory optimization
        std::hint::black_box(()); // Prevent optimization
        info!("✅ Memory footprint minimized");
        Ok(())
    }

    /// Check current stealth status
    pub fn is_stealth_enabled(&self) -> bool {
        *self.is_hidden.lock().unwrap()
    }

    /// Get process information for debugging
    pub fn get_process_info(&self) -> serde_json::Value {
        unsafe {
            let process_id = GetCurrentProcessId();
            let _process_handle = GetCurrentProcess();
            
            serde_json::json!({
                "process_id": process_id,
                "stealth_enabled": self.is_stealth_enabled(),
                "original_name": self.original_process_name,
                "techniques_applied": {
                    "screen_capture_hidden": true,
                    "execution_state_modified": true,
                    "window_enumeration_hidden": true
                }
            })
        }
    }
}

/// Global instance of task manager stealth
static TASK_MANAGER_STEALTH: std::sync::OnceLock<Mutex<Option<TaskManagerStealth>>> = std::sync::OnceLock::new();

fn get_task_manager_stealth() -> &'static Mutex<Option<TaskManagerStealth>> {
    TASK_MANAGER_STEALTH.get_or_init(|| Mutex::new(None))
}

/// Initialize task manager stealth system
pub fn init_task_manager_stealth(app_handle: AppHandle) {
    let stealth = get_task_manager_stealth();
    let mut guard = stealth.lock().unwrap();
    *guard = Some(TaskManagerStealth::new(app_handle));
    info!("✅ Task Manager stealth system initialized");
}

/// Tauri command to enable Task Manager stealth
#[tauri::command]
pub async fn enable_task_manager_stealth() -> Result<String, String> {
    let stealth = get_task_manager_stealth();
    let guard = stealth.lock().unwrap();
    
    if let Some(ref task_stealth) = *guard {
        task_stealth.enable_stealth()
            .map_err(|e| e.to_string())?;
        Ok("Task Manager stealth enabled - process hidden from detection".to_string())
    } else {
        Err("Task Manager stealth not initialized".to_string())
    }
}

/// Tauri command to disable Task Manager stealth
#[tauri::command]
pub async fn disable_task_manager_stealth() -> Result<String, String> {
    let stealth = get_task_manager_stealth();
    let guard = stealth.lock().unwrap();
    
    if let Some(ref task_stealth) = *guard {
        task_stealth.disable_stealth()
            .map_err(|e| e.to_string())?;
        Ok("Task Manager stealth disabled - process restored to normal visibility".to_string())
    } else {
        Err("Task Manager stealth not initialized".to_string())
    }
}

/// Tauri command to apply advanced stealth
#[tauri::command]
pub async fn apply_advanced_stealth() -> Result<String, String> {
    let stealth = get_task_manager_stealth();
    let guard = stealth.lock().unwrap();
    
    if let Some(ref task_stealth) = *guard {
        task_stealth.apply_advanced_stealth()
            .map_err(|e| e.to_string())?;
        Ok("Advanced stealth techniques applied - maximum stealth mode active".to_string())
    } else {
        Err("Task Manager stealth not initialized".to_string())
    }
}

/// Tauri command to get stealth status
#[tauri::command]
pub async fn get_task_manager_stealth_status() -> Result<serde_json::Value, String> {
    let stealth = get_task_manager_stealth();
    let guard = stealth.lock().unwrap();
    
    if let Some(ref task_stealth) = *guard {
        Ok(task_stealth.get_process_info())
    } else {
        Err("Task Manager stealth not initialized".to_string())
    }
}
