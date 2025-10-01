// Taskbar Manager for MockMate
// Provides functionality to hide/show the application in the Windows taskbar

use anyhow::Result;
use log::{info, warn, error};
use tauri::{AppHandle, Manager};

#[cfg(windows)]
use winapi::um::winuser::{
    GetWindowLongPtrW, SetWindowLongPtrW, ShowWindow,
    GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, SW_HIDE, SW_SHOW,
    SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW
};
#[cfg(windows)]
use winapi::shared::windef::HWND;

/// Taskbar visibility manager
pub struct TaskbarManager {
    app_handle: AppHandle,
    is_hidden_from_taskbar: bool,
}

impl TaskbarManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            is_hidden_from_taskbar: false,
        }
    }

    /// Hide the application from the Windows taskbar
    #[cfg(windows)]
    pub fn hide_from_taskbar(&mut self) -> Result<()> {
        info!("🔒 Hiding application from Windows taskbar...");
        
        if let Some(main_window) = self.app_handle.get_webview_window("main") {
            unsafe {
                // Get the window handle
                let hwnd = main_window.hwnd()?.0 as HWND;
                
                // Get current extended window styles
                let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                info!("📊 Current extended window style: 0x{:X}", ex_style);
                
                // Remove WS_EX_APPWINDOW and add WS_EX_TOOLWINDOW
                let new_ex_style = (ex_style as u32 & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW;
                
                info!("📊 New extended window style: 0x{:X}", new_ex_style);
                
                // Apply the new extended style
                let result = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style as isize);
                
                if result == 0 {
                    let error_code = winapi::um::errhandlingapi::GetLastError();
                    warn!("⚠️ SetWindowLongPtrW returned 0, error code: {}", error_code);
                } else {
                    info!("✅ Window extended style updated successfully");
                }
                
                // Force window refresh by hiding and showing
                ShowWindow(hwnd, SW_HIDE);
                ShowWindow(hwnd, SW_SHOW);
                
                // Ensure window stays on top if needed
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
                );
                
                self.is_hidden_from_taskbar = true;
                info!("✅ Application successfully hidden from taskbar");
            }
        } else {
            error!("❌ Main window not found");
            return Err(anyhow::anyhow!("Main window not found"));
        }
        
        Ok(())
    }

    /// Show the application in the Windows taskbar
    #[cfg(windows)]
    pub fn show_in_taskbar(&mut self) -> Result<()> {
        info!("🔓 Showing application in Windows taskbar...");
        
        if let Some(main_window) = self.app_handle.get_webview_window("main") {
            unsafe {
                // Get the window handle
                let hwnd = main_window.hwnd()?.0 as HWND;
                
                // Get current extended window styles
                let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                
                // Add WS_EX_APPWINDOW and remove WS_EX_TOOLWINDOW
                let new_ex_style = (ex_style as u32 | WS_EX_APPWINDOW) & !WS_EX_TOOLWINDOW;
                
                // Apply the new extended style
                let result = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style as isize);
                
                if result == 0 {
                    let error_code = winapi::um::errhandlingapi::GetLastError();
                    warn!("⚠️ SetWindowLongPtrW returned 0, error code: {}", error_code);
                } else {
                    info!("✅ Window extended style restored successfully");
                }
                
                // Force window refresh by hiding and showing
                ShowWindow(hwnd, SW_HIDE);
                ShowWindow(hwnd, SW_SHOW);
                
                // Remove topmost if it was set
                SetWindowPos(
                    hwnd,
                    HWND_NOTOPMOST,
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW
                );
                
                self.is_hidden_from_taskbar = false;
                info!("✅ Application restored to taskbar");
            }
        } else {
            error!("❌ Main window not found");
            return Err(anyhow::anyhow!("Main window not found"));
        }
        
        Ok(())
    }

    /// Toggle taskbar visibility
    #[cfg(windows)]
    pub fn toggle_taskbar_visibility(&mut self) -> Result<String> {
        if self.is_hidden_from_taskbar {
            self.show_in_taskbar()?;
            Ok("Application shown in taskbar".to_string())
        } else {
            self.hide_from_taskbar()?;
            Ok("Application hidden from taskbar".to_string())
        }
    }

    /// Check if currently hidden from taskbar
    pub fn is_hidden_from_taskbar(&self) -> bool {
        self.is_hidden_from_taskbar
    }

    /// Get taskbar status information
    pub fn get_taskbar_status(&self) -> serde_json::Value {
        serde_json::json!({
            "hidden_from_taskbar": self.is_hidden_from_taskbar,
            "platform": if cfg!(windows) { "windows" } else { "other" },
            "supported": cfg!(windows)
        })
    }

    // Fallback implementations for non-Windows platforms
    #[cfg(not(windows))]
    pub fn hide_from_taskbar(&mut self) -> Result<()> {
        warn!("Taskbar hiding not supported on this platform");
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn show_in_taskbar(&mut self) -> Result<()> {
        warn!("Taskbar showing not supported on this platform");
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn toggle_taskbar_visibility(&mut self) -> Result<String> {
        Ok("Taskbar visibility not supported on this platform".to_string())
    }
}

// Global instance management
static mut TASKBAR_MANAGER: Option<TaskbarManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global taskbar manager
pub fn initialize_taskbar_manager(app_handle: AppHandle) {
    INIT.call_once(|| {
        unsafe {
            TASKBAR_MANAGER = Some(TaskbarManager::new(app_handle));
        }
        info!("🔧 Taskbar manager initialized");
    });
}

/// Get reference to the global taskbar manager
pub fn get_taskbar_manager() -> Option<&'static mut TaskbarManager> {
    unsafe { TASKBAR_MANAGER.as_mut() }
}

// Tauri commands for taskbar management

#[tauri::command]
pub fn hide_from_taskbar() -> Result<String, String> {
    match get_taskbar_manager() {
        Some(manager) => manager.hide_from_taskbar().map(|_| "Application hidden from taskbar".to_string()).map_err(|e| e.to_string()),
        None => Err("Taskbar manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn show_in_taskbar() -> Result<String, String> {
    match get_taskbar_manager() {
        Some(manager) => manager.show_in_taskbar().map(|_| "Application shown in taskbar".to_string()).map_err(|e| e.to_string()),
        None => Err("Taskbar manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn toggle_taskbar_visibility() -> Result<String, String> {
    match get_taskbar_manager() {
        Some(manager) => manager.toggle_taskbar_visibility().map_err(|e| e.to_string()),
        None => Err("Taskbar manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn get_taskbar_status() -> Result<serde_json::Value, String> {
    match get_taskbar_manager() {
        Some(manager) => Ok(manager.get_taskbar_status()),
        None => Err("Taskbar manager not initialized".to_string()),
    }
}

#[tauri::command]
pub fn is_hidden_from_taskbar() -> Result<bool, String> {
    match get_taskbar_manager() {
        Some(manager) => Ok(manager.is_hidden_from_taskbar()),
        None => Err("Taskbar manager not initialized".to_string()),
    }
}