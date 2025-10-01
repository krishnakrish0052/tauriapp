use anyhow::{Result, anyhow};
use log::{info, warn, error};
use std::process::Command;
use serde_json::json;

/// Windows Stereo Mix Manager for automatic enablement
pub struct StereoMixManager;

impl StereoMixManager {
    /// Check if Stereo Mix is currently enabled using Windows API
    pub fn is_stereo_mix_enabled() -> Result<bool> {
        info!("Checking if Stereo Mix is enabled using Windows API...");
        
        // Use PowerShell to enumerate audio devices
        let powershell_cmd = r#"
            $deviceEnumerator = New-Object -ComObject MMDeviceEnumerator
            $devices = $deviceEnumerator.EnumAudioEndpoints(1, 1) # eCapture, DEVICE_STATE_ACTIVE
            
            for ($i = 0; $i -lt $devices.GetCount(); $i++) {
                $device = $devices.Item($i)
                $properties = $device.OpenPropertyStore(0)
                $deviceName = $properties.GetValue([GUID]"{a45c254e-df1c-4efd-8020-67d146a850e0}", 2).GetValue()
                
                if ($deviceName -match "Stereo Mix|What U Hear|Wave Out Mix") {
                    Write-Output "FOUND:$deviceName"
                }
            }
        "#;
        
        match Command::new("powershell")
            .args(&["-Command", powershell_cmd])
            .output() {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("FOUND:") {
                    let device_name = output_str.lines()
                        .find(|line| line.starts_with("FOUND:"))
                        .map(|line| &line[6..]) // Remove "FOUND:" prefix
                        .unwrap_or("Unknown");
                    info!("Found active Stereo Mix device: {}", device_name);
                    Ok(true)
                } else {
                    warn!("No active Stereo Mix device found");
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Failed to check for Stereo Mix devices: {}", e);
                Ok(false)
            }
        }
    }

    /// Attempt to enable Stereo Mix using PowerShell commands
    /// This requires elevated privileges
    pub fn enable_stereo_mix_powershell() -> Result<()> {
        info!("Attempting to enable Stereo Mix using PowerShell...");
        
        // PowerShell script to enable recording devices
        let powershell_script = r#"
            Add-Type -TypeDefinition @"
                using System;
                using System.Runtime.InteropServices;
                public class AudioDevices {
                    [DllImport("ole32.dll")]
                    public static extern int CoInitialize(IntPtr pvReserved);
                    
                    [DllImport("ole32.dll")]
                    public static extern void CoUninitialize();
                }
"@

            # Initialize COM
            [AudioDevices]::CoInitialize([IntPtr]::Zero)

            try {
                # Get audio devices
                $deviceEnumerator = New-Object -ComObject MMDeviceEnumerator
                $devices = $deviceEnumerator.EnumAudioEndpoints(1, 10) # eCapture, DEVICE_STATEMASK_ALL
                
                for ($i = 0; $i -lt $devices.GetCount(); $i++) {
                    $device = $devices.Item($i)
                    $properties = $device.OpenPropertyStore(0)
                    $deviceName = $properties.GetValue([GUID]"{a45c254e-df1c-4efd-8020-67d146a850e0}", 2).GetValue()
                    
                    # Check if this is Stereo Mix
                    if ($deviceName -match "Stereo Mix|What U Hear|Wave Out Mix") {
                        Write-Host "Found disabled recording device: $deviceName"
                        
                        # Enable the device (this part requires admin privileges)
                        $device.Activate([GUID]"{2EEF81BE-33FA-4800-9670-1CD474972C3F}", 23, [IntPtr]::Zero, [ref]$null)
                        Write-Host "Enabled: $deviceName"
                    }
                    
                    [System.Runtime.InteropServices.Marshal]::ReleaseComObject($properties) | Out-Null
                    [System.Runtime.InteropServices.Marshal]::ReleaseComObject($device) | Out-Null
                }
                
                [System.Runtime.InteropServices.Marshal]::ReleaseComObject($devices) | Out-Null
                [System.Runtime.InteropServices.Marshal]::ReleaseComObject($deviceEnumerator) | Out-Null
                
                Write-Host "Stereo Mix enablement completed"
            }
            catch {
                Write-Error "Failed to enable Stereo Mix: $_"
                exit 1
            }
            finally {
                # Cleanup COM
                [AudioDevices]::CoUninitialize()
            }
        "#;
        
        // Execute PowerShell script as administrator
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                "Start-Process",
                "powershell",
                "-ArgumentList",
                &format!("'{}'", powershell_script.replace('\n', "; ")),
                "-Verb",
                "RunAs",
                "-WindowStyle",
                "Hidden"
            ])
            .output()?;
            
        if output.status.success() {
            info!("PowerShell Stereo Mix enablement completed");
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("PowerShell command failed: {}", error_msg);
            Err(anyhow!("Failed to enable Stereo Mix via PowerShell: {}", error_msg))
        }
    }

    /// Enable Stereo Mix using Windows Registry modifications
    /// This is more reliable but requires elevated privileges
    pub fn enable_stereo_mix_registry() -> Result<()> {
        info!("Attempting to enable Stereo Mix using Registry...");
        
        // First, we need to identify the Stereo Mix device in the registry
        // This is complex because device IDs vary by system
        let registry_commands = vec![
            // Enable disabled audio devices globally
            r#"reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture" /f"#,
            
            // Common registry paths for Stereo Mix (these may vary)
            r#"reg add "HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e96c-e325-11ce-bfc1-08002be10318}" /v "EnableDefaultDevices" /t REG_DWORD /d 1 /f"#,
        ];
        
        for cmd in registry_commands {
            let output = Command::new("cmd")
                .args(&["/C", cmd])
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("Registry command successful: {}", cmd);
                    } else {
                        warn!("Registry command failed: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(e) => {
                    error!("Failed to execute registry command: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Use Device Manager command-line tools to enable Stereo Mix
    pub fn enable_stereo_mix_devcon() -> Result<()> {
        info!("Attempting to enable Stereo Mix using devcon...");
        
        // This requires devcon.exe to be available
        // devcon is part of Windows Driver Kit
        let commands = vec![
            "devcon enable *STEREOMIX*",
            "devcon enable *WAVEMIX*", 
            "devcon enable *WHATHEAR*",
        ];
        
        for cmd in commands {
            let output = Command::new("cmd")
                .args(&["/C", cmd])
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("devcon command successful: {}", cmd);
                        return Ok(());
                    } else {
                        warn!("devcon command failed: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(e) => {
                    warn!("devcon not available or failed: {}", e);
                }
            }
        }
        
        Err(anyhow!("All devcon commands failed"))
    }

    /// Open Windows Sound Control Panel for user to manually enable
    pub fn open_sound_control_panel() -> Result<()> {
        info!("Opening Windows Sound Control Panel...");
        
        let output = Command::new("cmd")
            .args(&["/C", "start", "mmsys.cpl"])
            .output()?;
            
        if output.status.success() {
            info!("Sound Control Panel opened successfully");
            Ok(())
        } else {
            Err(anyhow!("Failed to open Sound Control Panel"))
        }
    }

    /// Open specific sound recording tab in Control Panel
    pub fn open_recording_devices() -> Result<()> {
        info!("Opening Recording Devices...");
        
        // This opens directly to the Recording tab
        let output = Command::new("rundll32.exe")
            .args(&["shell32.dll,Control_RunDLL", "mmsys.cpl,,1"])
            .output()?;
            
        if output.status.success() {
            info!("Recording devices opened successfully");
            Ok(())
        } else {
            Err(anyhow!("Failed to open Recording devices"))
        }
    }

    /// Show user instructions for manually enabling Stereo Mix
    pub fn get_manual_enable_instructions() -> Vec<String> {
        vec![
            "Right-click on the Sound icon in the system tray".to_string(),
            "Select 'Open Sound settings'".to_string(),
            "Click 'Sound Control Panel' on the right side".to_string(),
            "Go to the 'Recording' tab".to_string(),
            "Right-click in empty space and select 'Show Disabled Devices'".to_string(),
            "Find 'Stereo Mix' and right-click on it".to_string(),
            "Select 'Enable' from the context menu".to_string(),
            "Click 'OK' to apply changes".to_string(),
        ]
    }

    /// Comprehensive method to attempt enabling Stereo Mix
    /// Tries multiple approaches in order of reliability
    pub fn auto_enable_stereo_mix() -> Result<String> {
        info!("Starting comprehensive Stereo Mix enablement...");
        
        // First check if it's already enabled
        if Self::is_stereo_mix_enabled()? {
            return Ok("Stereo Mix is already enabled".to_string());
        }
        
        // Method 1: Try devcon (most reliable if available)
        info!("Attempt 1: Using devcon...");
        if let Ok(_) = Self::enable_stereo_mix_devcon() {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if Self::is_stereo_mix_enabled()? {
                return Ok("Stereo Mix enabled successfully using devcon".to_string());
            }
        }
        
        // Method 2: Try PowerShell approach
        info!("Attempt 2: Using PowerShell...");
        if let Ok(_) = Self::enable_stereo_mix_powershell() {
            std::thread::sleep(std::time::Duration::from_secs(3));
            if Self::is_stereo_mix_enabled()? {
                return Ok("Stereo Mix enabled successfully using PowerShell".to_string());
            }
        }
        
        // Method 3: Try registry approach
        info!("Attempt 3: Using Registry...");
        if let Ok(_) = Self::enable_stereo_mix_registry() {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if Self::is_stereo_mix_enabled()? {
                return Ok("Stereo Mix enabled successfully using Registry".to_string());
            }
        }
        
        // If all automated methods fail, guide user to manual enablement
        warn!("All automatic methods failed, opening manual controls...");
        Self::open_recording_devices()?;
        
        Ok("Could not automatically enable Stereo Mix. Please enable it manually in the Recording devices window that just opened.".to_string())
    }

    /// Check system capabilities for Stereo Mix using Windows API
    pub fn check_stereo_mix_capability() -> Result<serde_json::Value> {
        info!("Checking system Stereo Mix capabilities using Windows API...");
        
        let mut capabilities = json!({
            "stereo_mix_available": false,
            "alternative_devices": [],
            "requires_manual_enable": false,
            "system_info": {}
        });
        
        // PowerShell script to enumerate all audio devices
        let powershell_cmd = r#"
            $deviceEnumerator = New-Object -ComObject MMDeviceEnumerator
            
            # Check active capture devices
            $devices = $deviceEnumerator.EnumAudioEndpoints(1, 1) # eCapture, DEVICE_STATE_ACTIVE
            for ($i = 0; $i -lt $devices.GetCount(); $i++) {
                $device = $devices.Item($i)
                $properties = $device.OpenPropertyStore(0)
                $deviceName = $properties.GetValue([GUID]"{a45c254e-df1c-4efd-8020-67d146a850e0}", 2).GetValue()
                
                if ($deviceName -match "Stereo Mix|What U Hear|Wave Out Mix") {
                    Write-Output "STEREO_MIX_ACTIVE:$deviceName"
                } elseif ($deviceName -match "loopback|mix") {
                    Write-Output "ALTERNATIVE_ACTIVE:$deviceName"
                }
            }
            
            # Check disabled capture devices
            $devices = $deviceEnumerator.EnumAudioEndpoints(1, 4) # eCapture, DEVICE_STATE_DISABLED
            for ($i = 0; $i -lt $devices.GetCount(); $i++) {
                $device = $devices.Item($i)
                $properties = $device.OpenPropertyStore(0)
                $deviceName = $properties.GetValue([GUID]"{a45c254e-df1c-4efd-8020-67d146a850e0}", 2).GetValue()
                
                if ($deviceName -match "Stereo Mix|What U Hear|Wave Out Mix") {
                    Write-Output "STEREO_MIX_DISABLED:$deviceName"
                } elseif ($deviceName -match "loopback|mix") {
                    Write-Output "ALTERNATIVE_DISABLED:$deviceName"
                }
            }
        "#;
        
        match Command::new("powershell")
            .args(&["-Command", powershell_cmd])
            .output() {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut alternative_devices = Vec::new();
                
                for line in output_str.lines() {
                    if line.starts_with("STEREO_MIX_ACTIVE:") {
                        capabilities["stereo_mix_available"] = json!(true);
                    } else if line.starts_with("STEREO_MIX_DISABLED:") {
                        capabilities["requires_manual_enable"] = json!(true);
                    } else if line.starts_with("ALTERNATIVE_ACTIVE:") || line.starts_with("ALTERNATIVE_DISABLED:") {
                        let device_name = line.split(':').nth(1).unwrap_or("Unknown");
                        alternative_devices.push(device_name.to_string());
                    }
                }
                
                capabilities["alternative_devices"] = json!(alternative_devices);
            }
            Err(e) => {
                error!("Failed to check system capabilities: {}", e);
                capabilities["requires_manual_enable"] = json!(true);
            }
        }
        
        // System info
        capabilities["system_info"] = json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        });
        
        Ok(capabilities)
    }
}

/// Tauri command to check if Stereo Mix is enabled
#[tauri::command]
pub async fn check_stereo_mix_enabled() -> Result<bool, String> {
    StereoMixManager::is_stereo_mix_enabled()
        .map_err(|e| e.to_string())
}

/// Tauri command to automatically enable Stereo Mix
#[tauri::command]
pub async fn enable_stereo_mix() -> Result<String, String> {
    StereoMixManager::auto_enable_stereo_mix()
        .map_err(|e| e.to_string())
}

/// Tauri command to open recording devices manually
#[tauri::command]
pub async fn open_recording_devices() -> Result<(), String> {
    StereoMixManager::open_recording_devices()
        .map_err(|e| e.to_string())
}

/// Tauri command to get system Stereo Mix capabilities
#[tauri::command]
pub async fn get_stereo_mix_capabilities() -> Result<serde_json::Value, String> {
    StereoMixManager::check_stereo_mix_capability()
        .map_err(|e| e.to_string())
}

/// Tauri command to get manual enablement instructions
#[tauri::command]
pub async fn get_stereo_mix_instructions() -> Result<Vec<String>, String> {
    Ok(StereoMixManager::get_manual_enable_instructions())
}
