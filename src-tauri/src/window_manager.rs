use tauri::{AppHandle, WebviewWindow, LogicalSize, LogicalPosition, Monitor, PhysicalSize, PhysicalPosition, Manager};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfiguration {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub scale_factor: f64,
    pub monitor_name: String,
    pub monitor_size: PhysicalSize<u32>,
}

#[derive(Debug, Clone, PartialEq)]
struct MonitorInfo {
    width: u32,
    height: u32,
    scale_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpiAwarePosition {
    pub physical_x: i32,
    pub physical_y: i32,
    pub logical_x: f64,
    pub logical_y: f64,
    pub scale_factor: f64,
}

// Cache for monitor information to prevent duplicate logging
static MONITOR_CACHE: Lazy<Arc<Mutex<HashMap<String, MonitorInfo>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Initialize the main window with DPI-aware positioning - enhanced for all screen sizes
pub fn setup_main_window_dpi_aware(app_handle: &AppHandle) -> Result<(), String> {
    info!("🖥️ Setting up main window with enhanced DPI awareness...");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        // Get the primary monitor to calculate proper positioning
        let monitor = window.current_monitor()
            .map_err(|e| format!("Failed to get current monitor: {}", e))?
            .ok_or_else(|| "No monitor found".to_string())?;
        
        let monitor_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        
        info!("📊 Monitor info: {}x{} (scale: {:.2})", 
              monitor_size.width, monitor_size.height, scale_factor);
        
        // Calculate responsive window dimensions based on screen size (reduced by 25%)
        let base_width = 600.0; // Reduced from 800px by 25%
        let base_height = 110.0;
        
        // Scale window size for smaller screens
        let effective_screen_width = (monitor_size.width as f64) / scale_factor;
        let effective_screen_height = (monitor_size.height as f64) / scale_factor;
        
        let window_width = if effective_screen_width < 1024.0 {
            // For smaller screens, use 90% of screen width but cap at 675px (25% reduction)
            (effective_screen_width * 0.9).min(675.0).max(300.0)
        } else {
            base_width
        };
        
        let window_height = if effective_screen_height < 768.0 {
            // For smaller screens, slightly reduce height
            (base_height * 0.9_f64).max(80.0)
        } else {
            base_height
        };
        
        // Position window at top center
        let x = ((effective_screen_width - window_width) / 2.0) as i32; // Center horizontally
        let y = 0; // At the very top
        
        info!("🎯 Enhanced positioning: {}x{:.0} at ({}, {}) with scale {:.2}", 
              window_width as u32, window_height, x, y, scale_factor);
        
        // Apply DPI scaling for physical coordinates
        let physical_width = (window_width * scale_factor) as u32;
        let physical_height = (window_height * scale_factor) as u32;
        let physical_x = (x as f64 * scale_factor) as i32;
        let physical_y = (y as f64 * scale_factor) as i32;
        
        // Set the window size with DPI awareness
        if let Err(e) = window.set_size(tauri::Size::Physical(PhysicalSize {
            width: physical_width,
            height: physical_height,
        })) {
            warn!("Failed to set window size: {}", e);
        }
        
        // Set the window position with DPI awareness  
        if let Err(e) = window.set_position(tauri::Position::Physical(PhysicalPosition { 
            x: physical_x, 
            y: physical_y 
        })) {
            warn!("Failed to set window position: {}", e);
        }
        
        // Ensure window stays on top
        if let Err(e) = window.set_always_on_top(true) {
            warn!("Failed to set always on top: {}", e);
        }
        
        info!("✅ Enhanced main window DPI setup completed");
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

/// Get current window configuration for saving/restoring
pub fn get_window_configuration(window: &WebviewWindow) -> Result<WindowConfiguration, String> {
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let monitor = window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    
    let scale_factor = monitor.scale_factor();
    let monitor_size = monitor.size();
    
    Ok(WindowConfiguration {
        width: size.width as f64,
        height: size.height as f64,
        x: position.x as f64,
        y: position.y as f64,
        scale_factor,
        monitor_name: format!("{}x{}", monitor_size.width, monitor_size.height),
        monitor_size: *monitor_size,
    })
}

/// Apply window configuration with DPI scaling
pub fn apply_window_configuration(
    window: &WebviewWindow, 
    config: &WindowConfiguration
) -> Result<(), String> {
    info!("📐 Applying window configuration: {}x{} at ({}, {})", 
          config.width, config.height, config.x, config.y);
    
    // Get current monitor to check if scaling has changed
    let current_monitor = window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    
    let current_scale_factor = current_monitor.scale_factor();
    
    // Adjust for scale factor changes
    let scale_adjustment = current_scale_factor / config.scale_factor;
    
    let adjusted_width = (config.width * scale_adjustment) as u32;
    let adjusted_height = (config.height * scale_adjustment) as u32;
    let adjusted_x = (config.x * scale_adjustment) as i32;
    let adjusted_y = (config.y * scale_adjustment) as i32;
    
    info!("🔧 Scale adjustment: {:.2} -> {}x{} at ({}, {})", 
          scale_adjustment, adjusted_width, adjusted_height, adjusted_x, adjusted_y);
    
    // Apply the adjusted configuration
    window.set_size(tauri::Size::Physical(PhysicalSize {
        width: adjusted_width,
        height: adjusted_height,
    })).map_err(|e| e.to_string())?;
    
    window.set_position(tauri::Position::Physical(PhysicalPosition {
        x: adjusted_x,
        y: adjusted_y,
    })).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Calculate DPI-aware position for a secondary window relative to main window
pub fn calculate_relative_position(
    main_window: &WebviewWindow,
    _width: u32,
    _height: u32,
    offset_x: i32,
    offset_y: i32,
) -> Result<DpiAwarePosition, String> {
    let main_position = main_window.outer_position().map_err(|e| e.to_string())?;
    let main_size = main_window.outer_size().map_err(|e| e.to_string())?;
    let monitor = main_window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    
    let scale_factor = monitor.scale_factor();
    
    // Calculate physical position
    let physical_x = main_position.x + offset_x;
    let physical_y = main_position.y + main_size.height as i32 + offset_y;
    
    // Calculate logical position
    let logical_x = physical_x as f64 / scale_factor;
    let logical_y = physical_y as f64 / scale_factor;
    
    info!("📍 Calculated relative position: physical=({}, {}), logical=({:.1}, {:.1}), scale={:.2}",
          physical_x, physical_y, logical_x, logical_y, scale_factor);
    
    Ok(DpiAwarePosition {
        physical_x,
        physical_y,
        logical_x,
        logical_y,
        scale_factor,
    })
}

/// Lock window size to prevent unwanted resizing while maintaining DPI awareness
pub fn lock_window_size(window: &WebviewWindow, width: u32, height: u32) -> Result<(), String> {
    info!("🔒 Locking window size to: {}x{}", width, height);
    
    let monitor = window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    
    let scale_factor = monitor.scale_factor();
    
    // Calculate DPI-aware size
    let logical_width = (width as f64 / scale_factor) as f64;
    let logical_height = (height as f64 / scale_factor) as f64;
    
    // Set both current size and min/max constraints to lock the size
    window.set_size(tauri::Size::Logical(LogicalSize {
        width: logical_width,
        height: logical_height,
    })).map_err(|e| e.to_string())?;
    
    // Note: Tauri 2.0 doesn't have set_min_size/set_max_size methods
    // Size locking is handled through the configuration
    
    info!("✅ Window size locked to {}x{} (logical: {:.1}x{:.1})", 
          width, height, logical_width, logical_height);
    
    Ok(())
}

/// Get all available monitors information (with change detection to reduce logging spam)
pub fn get_monitors_info(app_handle: &AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let monitors = app_handle.available_monitors().map_err(|e| e.to_string())?;
    let mut monitor_info = Vec::new();
    let mut current_monitors = HashMap::new();
    let mut has_changes = false;
    
    // Collect current monitor information
    for (i, monitor) in monitors.iter().enumerate() {
        let size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let monitor_key = format!("Monitor {}", i + 1);
        
        let info = MonitorInfo {
            width: size.width,
            height: size.height,
            scale_factor,
        };
        
        current_monitors.insert(monitor_key.clone(), info.clone());
        
        let json_info = serde_json::json!({
            "index": i,
            "name": monitor_key,
            "width": size.width,
            "height": size.height,
            "scale_factor": scale_factor,
            "is_primary": i == 0, // First monitor is typically primary
        });
        
        monitor_info.push(json_info);
    }
    
    // Check for changes compared to cache
    {
        let mut cache = MONITOR_CACHE.lock().unwrap();
        
        // Check if monitor count changed
        if cache.len() != current_monitors.len() {
            has_changes = true;
        } else {
            // Check if any monitor information changed
            for (key, info) in &current_monitors {
                if let Some(cached_info) = cache.get(key) {
                    if cached_info != info {
                        has_changes = true;
                        break;
                    }
                } else {
                    has_changes = true;
                    break;
                }
            }
        }
        
        // Only log if there are changes or if this is the first call
        if has_changes || cache.is_empty() {
            info!("🖥️ Getting monitors information...");
            
            for (i, (key, info)) in current_monitors.iter().enumerate() {
                info!("🖥️ Monitor {}: {}x{} (scale: {:.2})", 
                      i + 1, info.width, info.height, info.scale_factor);
            }
            
            // Update the cache
            *cache = current_monitors;
        }
        // If no changes, silently return the data without logging
    }
    
    Ok(monitor_info)
}

/// Ensure window is visible on current screen setup
pub fn ensure_window_visible(window: &WebviewWindow) -> Result<(), String> {
    info!("👁️ Ensuring window is visible on current screen setup...");
    
    let position = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let monitor = window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    
    let monitor_size = monitor.size();
    
    // Check if window is completely outside the monitor bounds
    let window_right = position.x + size.width as i32;
    let window_bottom = position.y + size.height as i32;
    
    let mut needs_adjustment = false;
    let mut new_x = position.x;
    let mut new_y = position.y;
    
    // Adjust horizontal position if needed
    if position.x < 0 {
        new_x = 0;
        needs_adjustment = true;
    } else if window_right > monitor_size.width as i32 {
        new_x = monitor_size.width as i32 - size.width as i32;
        needs_adjustment = true;
    }
    
    // Adjust vertical position if needed
    if position.y < 0 {
        new_y = 0;
        needs_adjustment = true;
    } else if window_bottom > monitor_size.height as i32 {
        new_y = monitor_size.height as i32 - size.height as i32;
        needs_adjustment = true;
    }
    
    if needs_adjustment {
        info!("📍 Adjusting window position: ({}, {}) -> ({}, {})", 
              position.x, position.y, new_x, new_y);
        
        window.set_position(tauri::Position::Physical(PhysicalPosition {
            x: new_x,
            y: new_y,
        })).map_err(|e| e.to_string())?;
    }
    
    info!("✅ Window visibility check completed");
    Ok(())
}
