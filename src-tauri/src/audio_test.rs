use cpal::traits::{DeviceTrait, HostTrait};
use log::info;
use anyhow::Result;

/// Audio testing utility for debugging audio device issues
pub struct AudioTest;

impl AudioTest {
    /// Test and list all available audio devices
    pub fn list_all_devices() -> Result<()> {
        info!("🔍 Testing audio devices...");
        
        let host = cpal::default_host();
        info!("Using audio host: {:?}", host.id());

        // List input devices
        info!("\n📥 INPUT DEVICES:");
        match host.input_devices() {
            Ok(devices) => {
                for (i, device) in devices.enumerate() {
                    match device.name() {
                        Ok(name) => {
                            info!("  {}. {}", i + 1, name);
                            
                            // Test if device supports input
                            match device.default_input_config() {
                                Ok(config) => {
                                    info!("     ✅ Input: {} Hz, {} channels, {:?}", 
                                          config.sample_rate().0, 
                                          config.channels(), 
                                          config.sample_format());
                                }
                                Err(e) => {
                                    info!("     ❌ Input not supported: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            info!("  {}. [Error getting name: {}]", i + 1, e);
                        }
                    }
                }
            }
            Err(e) => {
                info!("❌ Failed to get input devices: {}", e);
            }
        }

        // List output devices
        info!("\n📤 OUTPUT DEVICES:");
        match host.output_devices() {
            Ok(devices) => {
                for (i, device) in devices.enumerate() {
                    match device.name() {
                        Ok(name) => {
                            info!("  {}. {}", i + 1, name);
                            
                            // Test if device supports output
                            match device.default_output_config() {
                                Ok(config) => {
                                    info!("     ✅ Output: {} Hz, {} channels, {:?}", 
                                          config.sample_rate().0, 
                                          config.channels(), 
                                          config.sample_format());
                                }
                                Err(e) => {
                                    info!("     ❌ Output not supported: {}", e);
                                }
                            }
                            
                            // Check if output device supports input (loopback)
                            match device.default_input_config() {
                                Ok(config) => {
                                    info!("     🔄 Loopback: {} Hz, {} channels, {:?}", 
                                          config.sample_rate().0, 
                                          config.channels(), 
                                          config.sample_format());
                                }
                                Err(_) => {
                                    info!("     ❌ No loopback support");
                                }
                            }
                        }
                        Err(e) => {
                            info!("  {}. [Error getting name: {}]", i + 1, e);
                        }
                    }
                }
            }
            Err(e) => {
                info!("❌ Failed to get output devices: {}", e);
            }
        }

        // Show default devices
        info!("\n🎯 DEFAULT DEVICES:");
        match host.default_input_device() {
            Some(device) => {
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("  Default Input: {}", name);
            }
            None => {
                info!("  ❌ No default input device");
            }
        }

        match host.default_output_device() {
            Some(device) => {
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("  Default Output: {}", name);
            }
            None => {
                info!("  ❌ No default output device");
            }
        }

        info!("\n✅ Audio device test completed!");
        Ok(())
    }

    /// Test specific device configurations
    pub fn test_device_configs(device_name: &str) -> Result<()> {
        info!("🔧 Testing configurations for device: {}", device_name);
        
        let host = cpal::default_host();
        
        // Find device by name
        let mut target_device = None;
        
        // Check input devices
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name.to_lowercase().contains(&device_name.to_lowercase()) {
                        target_device = Some(device);
                        break;
                    }
                }
            }
        }
        
        // Check output devices if not found in input
        if target_device.is_none() {
            if let Ok(devices) = host.output_devices() {
                for device in devices {
                    if let Ok(name) = device.name() {
                        if name.to_lowercase().contains(&device_name.to_lowercase()) {
                            target_device = Some(device);
                            break;
                        }
                    }
                }
            }
        }
        
        match target_device {
            Some(device) => {
                let full_name = device.name().unwrap_or("Unknown".to_string());
                info!("Found device: {}", full_name);
                
                // Test input configurations
                info!("\n📥 INPUT CONFIGURATIONS:");
                match device.supported_input_configs() {
                    Ok(configs) => {
                        for (i, config) in configs.enumerate() {
                            info!("  Config {}: {} Hz (min: {}, max: {}), {} channels max, {:?}", 
                                  i + 1,
                                  config.max_sample_rate().0,
                                  config.min_sample_rate().0,
                                  config.max_sample_rate().0,
                                  config.channels(),
                                  config.sample_format());
                            
                            // Test specific rates we care about
                            for &rate in &[16000, 44100, 48000] {
                                let sample_rate = cpal::SampleRate(rate);
                                if config.min_sample_rate() <= sample_rate && sample_rate <= config.max_sample_rate() {
                                    info!("    ✅ Supports {} Hz", rate);
                                } else {
                                    info!("    ❌ Does not support {} Hz", rate);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        info!("  ❌ Failed to get input configs: {}", e);
                    }
                }
                
                // Test output configurations
                info!("\n📤 OUTPUT CONFIGURATIONS:");
                match device.supported_output_configs() {
                    Ok(configs) => {
                        for (i, config) in configs.enumerate() {
                            info!("  Config {}: {} Hz (min: {}, max: {}), {} channels max, {:?}", 
                                  i + 1,
                                  config.max_sample_rate().0,
                                  config.min_sample_rate().0,
                                  config.max_sample_rate().0,
                                  config.channels(),
                                  config.sample_format());
                        }
                    }
                    Err(e) => {
                        info!("  ❌ Failed to get output configs: {}", e);
                    }
                }
            }
            None => {
                info!("❌ Device '{}' not found", device_name);
                return Err(anyhow::anyhow!("Device not found"));
            }
        }
        
        Ok(())
    }
}

/// Tauri command to test audio devices
#[tauri::command]
pub async fn test_audio_devices() -> Result<String, String> {
    match AudioTest::list_all_devices() {
        Ok(_) => Ok("Audio device test completed successfully. Check logs for details.".to_string()),
        Err(e) => Err(format!("Audio test failed: {}", e))
    }
}

/// Tauri command to test specific device
#[tauri::command]
pub async fn test_specific_device(device_name: String) -> Result<String, String> {
    match AudioTest::test_device_configs(&device_name) {
        Ok(_) => Ok(format!("Device '{}' test completed. Check logs for details.", device_name)),
        Err(e) => Err(format!("Device test failed: {}", e))
    }
}
