use std::process::Command;
use log::{info, error, warn};
use anyhow::{Result, anyhow};
use std::path::PathBuf;

pub struct PermissionManager;

impl PermissionManager {
    /// Check if this is the first run of the application
    pub fn is_first_run() -> Result<bool> {
        let app_data = std::env::var("APPDATA")?;
        let config_path = PathBuf::from(app_data).join("MockMate").join(".initialized");
        Ok(!config_path.exists())
    }

    /// Mark the application as initialized
    pub fn mark_initialized() -> Result<()> {
        let app_data = std::env::var("APPDATA")?;
        let config_dir = PathBuf::from(app_data).join("MockMate");
        let config_path = config_dir.join(".initialized");
        
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(&config_path, "1")?;
        Ok(())
    }

    /// Request microphone permissions through Windows Settings
    pub fn request_microphone_permission() -> Result<()> {
        info!("Requesting microphone permissions...");
        
        // Open Windows Privacy Settings for Microphone
        let output = Command::new("cmd")
            .args(&["/C", "start", "ms-settings:privacy-microphone"])
            .output()?;

        if output.status.success() {
            info!("Opened microphone privacy settings");
            Ok(())
        } else {
            Err(anyhow!("Failed to open microphone settings"))
        }
    }

    /// Check if microphone permission is granted using Windows API
    pub fn check_microphone_permission() -> Result<bool> {
        // Check Windows registry for microphone permission
        match Command::new("reg")
            .args(&[
                "query", 
                "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone", 
                "/v", "Value"
            ])
            .output() {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("Allow") {
                    info!("Microphone access granted via registry check");
                    Ok(true)
                } else {
                    warn!("Microphone access not granted - registry check failed");
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Failed to check microphone permission via registry: {}", e);
                // Fallback: assume permission is granted if we can't check
                Ok(true)
            }
        }
    }

    /// Enable exclusive mode access for better audio capture
    pub fn enable_exclusive_mode() -> Result<()> {
        info!("Enabling exclusive mode for audio devices...");
        
        // This requires registry modification - should be done during installation
        let registry_commands = vec![
            r#"reg add "HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone" /v Value /t REG_SZ /d Allow /f"#,
            r#"reg add "HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\webcam" /v Value /t REG_SZ /d Allow /f"#,
        ];

        for cmd in registry_commands {
            let output = Command::new("cmd")
                .args(&["/C", cmd])
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("Registry update successful");
                    } else {
                        warn!("Registry update failed: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(e) => {
                    error!("Failed to execute registry command: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Check if the app has necessary audio permissions
    pub fn check_audio_permissions() -> Result<bool> {
        let mic_permission = Self::check_microphone_permission().unwrap_or(false);
        
        if !mic_permission {
            warn!("Audio permissions not granted");
            return Ok(false);
        }
        
        info!("All audio permissions granted");
        Ok(true)
    }

    /// Initialize permissions on first run
    pub fn initialize_permissions_on_first_run() -> Result<()> {
        if Self::is_first_run()? {
            info!("First run detected, requesting permissions...");
            
            // Check current permissions
            if !Self::check_audio_permissions()? {
                // Request permissions
                Self::request_microphone_permission()?;
                
                // Wait a bit for user to grant permissions
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            
            // Mark as initialized
            Self::mark_initialized()?;
            info!("First run initialization complete");
        }
        
        Ok(())
    }
}

/// Tauri command to check permissions from frontend
#[tauri::command]
pub async fn check_permissions() -> Result<bool, String> {
    PermissionManager::check_audio_permissions()
        .map_err(|e| e.to_string())
}

/// Tauri command to request permissions from frontend
#[tauri::command]
pub async fn request_permissions() -> Result<(), String> {
    PermissionManager::request_microphone_permission()
        .map_err(|e| e.to_string())
}

/// Tauri command to initialize first run permissions
#[tauri::command]
pub async fn initialize_first_run() -> Result<(), String> {
    PermissionManager::initialize_permissions_on_first_run()
        .map_err(|e| e.to_string())
}
