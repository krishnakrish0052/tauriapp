// WASAPI Loopback stub for compatibility
// The actual WASAPI functionality is now in pluely_audio.rs

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub device_type: String,
    pub supports_loopback: bool,
}

pub struct WasapiLoopback;

impl WasapiLoopback {
    pub fn list_all_devices() -> Result<Vec<AudioDevice>> {
        Ok(vec![
            AudioDevice {
                name: "System Audio (WASAPI Loopback)".to_string(),
                is_default: true,
                device_type: "system_audio".to_string(),
                supports_loopback: true,
            },
        ])
    }
    
    pub fn get_device_supported_configs(_device_name: &str) -> Result<Vec<String>> {
        Ok(vec![
            "44100 Hz, 1 channel, f32 (Pluely WASAPI)".to_string(),
        ])
    }
}