use anyhow::Result;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use log::{info, warn, error};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, SampleFormat, Sample};

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub device_type: String, // "input" or "output"
    pub supports_loopback: bool,
}

pub struct WasapiLoopback {
    is_recording: Arc<AtomicBool>,
    sample_rate: u32,
    channels: u16,
    captured_samples: Arc<Mutex<VecDeque<f32>>>,
    selected_device_name: Option<String>,
}

impl WasapiLoopback {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            sample_rate: 44100,
            channels: 2,
            captured_samples: Arc::new(Mutex::new(VecDeque::new())),
            selected_device_name: None,
        }
    }
    
    pub fn new_with_device(device_name: String) -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            sample_rate: 44100,
            channels: 2,
            captured_samples: Arc::new(Mutex::new(VecDeque::new())),
            selected_device_name: Some(device_name),
        }
    }

    pub fn start_capture(&mut self) -> Result<()> {
        if self.is_recording.load(Ordering::Relaxed) {
            info!("WASAPI loopback capture is already running");
            return Ok(());
        }

        info!("Starting WASAPI loopback capture...");
        
        let host = cpal::default_host();
        
        // For Windows loopback, we need to find a device that actually supports loopback
        let device = self.get_loopback_device(&host)
            .ok_or_else(|| anyhow::anyhow!("No loopback-capable device found"))?;
        
        let device_name = device.name().unwrap_or("Unknown".to_string());
        info!("Using loopback device: {}", device_name);
        
        // Try to get a working input configuration
        let config = self.get_working_input_config(&device)
            .ok_or_else(|| anyhow::anyhow!("No working input configuration found for device"))?;
            
        info!("Audio config: {} Hz, {} channels, {:?}", 
              config.sample_rate().0, config.channels(), config.sample_format());
        
        self.sample_rate = config.sample_rate().0;
        self.channels = config.channels();
        
        let samples_arc = self.captured_samples.clone();
        let is_recording = self.is_recording.clone();
        
        // Build the input stream
        let stream = match config.sample_format() {
            SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), samples_arc, is_recording)?,
            SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), samples_arc, is_recording)?,
            SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), samples_arc, is_recording)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", config.sample_format())),
        };
        
        // Start the stream
        stream.play().map_err(|e| anyhow::anyhow!("Failed to start audio stream: {}", e))?;
        
        self.is_recording.store(true, Ordering::Relaxed);
        
        info!("Audio capture started successfully");
        
        // Keep the stream alive by forgetting it (required for capture to continue)
        std::mem::forget(stream);
        
        Ok(())
    }

    pub fn stop_capture(&mut self) -> Result<()> {
        if !self.is_recording.load(Ordering::Relaxed) {
            warn!("WASAPI loopback capture is not running");
            return Ok(());
        }

        info!("Stopping WASAPI loopback capture...");
        self.is_recording.store(false, Ordering::Relaxed);

        // Give the thread a moment to stop
        thread::sleep(Duration::from_millis(100));

        Ok(())
    }

    pub fn get_captured_samples(&self) -> Vec<f32> {
        let samples = self.captured_samples.lock().unwrap();
        samples.iter().cloned().collect()
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn get_channels(&self) -> u16 {
        self.channels
    }
    
    /// Get detailed supported configurations for a device
    pub fn get_device_supported_configs(device_name: &str) -> Result<Vec<String>> {
        let host = cpal::default_host();
        let mut config_info = Vec::new();
        
        // Check input devices
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    if name == device_name {
                        info!("Found input device: {}", name);
                        
                        // Get supported input configurations
                        match device.supported_input_configs() {
                            Ok(configs) => {
                                info!("Supported input configurations for '{}':", name);
                                for (i, config) in configs.enumerate() {
                                    let config_str = format!(
                                        "  Config {}: {}Hz-{}Hz, {} channels, {:?}",
                                        i + 1,
                                        config.min_sample_rate().0,
                                        config.max_sample_rate().0,
                                        config.channels(),
                                        config.sample_format()
                                    );
                                    info!("{}", config_str);
                                    config_info.push(config_str);
                                }
                                
                                // Try to get default config
                                match device.default_input_config() {
                                    Ok(default_config) => {
                                        let default_str = format!(
                                            "  Default: {}Hz, {} channels, {:?}",
                                            default_config.sample_rate().0,
                                            default_config.channels(),
                                            default_config.sample_format()
                                        );
                                        info!("{}", default_str);
                                        config_info.push(default_str);
                                    }
                                    Err(e) => {
                                        let error_str = format!("  Default config error: {}", e);
                                        warn!("{}", error_str);
                                        config_info.push(error_str);
                                    }
                                }
                            }
                            Err(e) => {
                                let error_str = format!("  Failed to get input configs: {}", e);
                                warn!("{}", error_str);
                                config_info.push(error_str);
                            }
                        }
                        return Ok(config_info);
                    }
                }
            }
        }
        
        // Check output devices
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    if name == device_name {
                        info!("Found output device: {}", name);
                        
                        // Get supported input configurations (for loopback)
                        match device.supported_input_configs() {
                            Ok(configs) => {
                                info!("Supported input (loopback) configurations for '{}':", name);
                                for (i, config) in configs.enumerate() {
                                    let config_str = format!(
                                        "  Loopback Config {}: {}Hz-{}Hz, {} channels, {:?}",
                                        i + 1,
                                        config.min_sample_rate().0,
                                        config.max_sample_rate().0,
                                        config.channels(),
                                        config.sample_format()
                                    );
                                    info!("{}", config_str);
                                    config_info.push(config_str);
                                }
                                
                                // Try to get default input config from output device
                                match device.default_input_config() {
                                    Ok(default_config) => {
                                        let default_str = format!(
                                            "  Default Loopback: {}Hz, {} channels, {:?}",
                                            default_config.sample_rate().0,
                                            default_config.channels(),
                                            default_config.sample_format()
                                        );
                                        info!("{}", default_str);
                                        config_info.push(default_str);
                                    }
                                    Err(e) => {
                                        let error_str = format!("  Default loopback config error: {}", e);
                                        warn!("{}", error_str);
                                        config_info.push(error_str);
                                    }
                                }
                            }
                            Err(e) => {
                                let error_str = format!("  No loopback support: {}", e);
                                info!("{}", error_str);
                                config_info.push(error_str);
                            }
                        }
                        
                        // Also check regular output configs
                        match device.supported_output_configs() {
                            Ok(configs) => {
                                info!("Supported output configurations for '{}':", name);
                                for (i, config) in configs.enumerate() {
                                    let config_str = format!(
                                        "  Output Config {}: {}Hz-{}Hz, {} channels, {:?}",
                                        i + 1,
                                        config.min_sample_rate().0,
                                        config.max_sample_rate().0,
                                        config.channels(),
                                        config.sample_format()
                                    );
                                    info!("{}", config_str);
                                    config_info.push(config_str);
                                }
                            }
                            Err(e) => {
                                let error_str = format!("  Failed to get output configs: {}", e);
                                warn!("{}", error_str);
                                config_info.push(error_str);
                            }
                        }
                        
                        return Ok(config_info);
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Device '{}' not found", device_name))
    }
    
    /// List all available audio devices
    pub fn list_all_devices() -> Result<Vec<AudioDevice>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        info!("=== Listing All Audio Devices ===");
        
        // Get default devices for comparison
        let default_input = host.default_input_device();
        let default_output = host.default_output_device();
        
        // List input devices
        info!("Input Devices:");
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                let name = device.name().unwrap_or("Unknown Input Device".to_string());
                let is_default = default_input.as_ref().map_or(false, |d| {
                    d.name().unwrap_or_default() == name
                });
                
                info!("  - {} {}", name, if is_default { "(Default)" } else { "" });
                
                // Check supported configurations
                if let Ok(configs) = device.supported_input_configs() {
                    let config_count = configs.count();
                    info!("    Supported configurations: {}", config_count);
                }
                
                devices.push(AudioDevice {
                    name: name.clone(),
                    is_default,
                    device_type: "input".to_string(),
                    supports_loopback: false, // Most input devices don't support loopback
                });
            }
        }
        
        // List output devices
        info!("Output Devices:");
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                let name = device.name().unwrap_or("Unknown Output Device".to_string());
                let is_default = default_output.as_ref().map_or(false, |d| {
                    d.name().unwrap_or_default() == name
                });
                
                info!("  - {} {}", name, if is_default { "(Default)" } else { "" });
                
                // Check if this output device might support input (loopback)
                let supports_loopback = device.supported_input_configs().is_ok();
                if supports_loopback {
                    info!("    ✓ Supports loopback capture");
                }
                
                devices.push(AudioDevice {
                    name: name.clone(),
                    is_default,
                    device_type: "output".to_string(),
                    supports_loopback,
                });
            }
        }
        
        info!("=== Found {} total devices ===", devices.len());
        Ok(devices)
    }
    
    /// Find a device that supports loopback capture
    fn get_loopback_device(&self, host: &Host) -> Option<Device> {
        // If a specific device was selected, try to find it first
        if let Some(ref selected_name) = self.selected_device_name {
            info!("Looking for selected device: {}", selected_name);
            
            // Check input devices first
            if let Ok(input_devices) = host.input_devices() {
                for device in input_devices {
                    if let Ok(name) = device.name() {
                        if name == *selected_name && self.get_working_input_config(&device).is_some() {
                            info!("Found selected input device: {}", name);
                            return Some(device);
                        }
                    }
                }
            }
            
            // Then check output devices for loopback capability
            if let Ok(output_devices) = host.output_devices() {
                for device in output_devices {
                    if let Ok(name) = device.name() {
                        if name == *selected_name && self.get_working_input_config(&device).is_some() {
                            info!("Found selected output device with loopback: {}", name);
                            return Some(device);
                        }
                    }
                }
            }
        }
        
        // Fallback: find any device that has working input configuration
        
        // Try default input device first (most likely to work)
        if let Some(device) = host.default_input_device() {
            if self.get_working_input_config(&device).is_some() {
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("Using default input device: {}", name);
                return Some(device);
            }
        }
        
        // Try other input devices (microphones)
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if self.get_working_input_config(&device).is_some() {
                    let name = device.name().unwrap_or("Unknown".to_string());
                    info!("Using input device: {}", name);
                    return Some(device);
                }
            }
        }
        
        // Last resort: try output devices for loopback (often doesn't work on Windows)
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if self.get_working_input_config(&device).is_some() {
                    let name = device.name().unwrap_or("Unknown".to_string());
                    warn!("Using output device for loopback (may not capture system audio): {}", name);
                    return Some(device);
                }
            }
        }
        
        warn!("No devices with working input configurations found");
        None
    }
    
    /// Try to get a working input configuration from the device
    fn get_working_input_config(&self, device: &Device) -> Option<cpal::SupportedStreamConfig> {
        // First try the default input config
        if let Ok(config) = device.default_input_config() {
            info!("Using default input config: {} Hz, {} channels, {:?}", 
                  config.sample_rate().0, config.channels(), config.sample_format());
            return Some(config);
        }
        
        // If default doesn't work, try to find any supported configuration
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                // Try to use a reasonable sample rate and channel configuration
                let sample_rate = if config.min_sample_rate().0 <= 44100 && config.max_sample_rate().0 >= 44100 {
                    cpal::SampleRate(44100)
                } else {
                    config.min_sample_rate()
                };
                
                let stream_config = config.with_sample_rate(sample_rate);
                info!("Using supported input config: {} Hz, {} channels, {:?}", 
                      stream_config.sample_rate().0, stream_config.channels(), stream_config.sample_format());
                return Some(stream_config);
            }
        }
        
        None
    }
    
    fn get_system_audio_device(&self, host: &Host) -> Option<Device> {
        // If a specific device was selected, try to find it
        if let Some(ref selected_name) = self.selected_device_name {
            info!("Looking for selected device: {}", selected_name);
            
            // First check input devices
            if let Ok(input_devices) = host.input_devices() {
                for device in input_devices {
                    if let Ok(name) = device.name() {
                        if name == *selected_name {
                            info!("Found selected input device: {}", name);
                            return Some(device);
                        }
                    }
                }
            }
            
            // Then check output devices (for loopback)
            if let Ok(output_devices) = host.output_devices() {
                for device in output_devices {
                    if let Ok(name) = device.name() {
                        if name == *selected_name {
                            // Check if this output device supports input (loopback)
                            if device.supported_input_configs().is_ok() {
                                info!("Found selected output device with loopback support: {}", name);
                                return Some(device);
                            } else {
                                warn!("Selected output device '{}' doesn't support loopback", name);
                            }
                        }
                    }
                }
            }
            
            warn!("Selected device '{}' not found, falling back to default", selected_name);
        }
        
        // Fallback logic: prefer devices that support loopback, then default input
        
        // First, try to find output devices that support loopback (for system audio)
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if device.supported_input_configs().is_ok() {
                    let name = device.name().unwrap_or("Unknown".to_string());
                    info!("Using output device with loopback support: {}", name);
                    return Some(device);
                }
            }
        }
        
        // Then try default input device (microphone)
        if let Some(device) = host.default_input_device() {
            let name = device.name().unwrap_or("Unknown".to_string());
            info!("Using default input device: {}", name);
            return Some(device);
        }
        
        // Finally, try any available input device
        if let Ok(mut devices) = host.input_devices() {
            if let Some(device) = devices.next() {
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("Using first available input device: {}", name);
                return Some(device);
            }
        }
        
        warn!("No suitable audio devices found");
        None
    }
    
    fn build_stream<T>(
        device: &Device,
        config: &cpal::StreamConfig,
        samples_arc: Arc<Mutex<VecDeque<f32>>>,
        is_recording: Arc<AtomicBool>,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                // Always log the first few callbacks to verify it's working
                static mut CALLBACK_COUNT: u32 = 0;
                unsafe {
                    CALLBACK_COUNT += 1;
                    if CALLBACK_COUNT <= 5 {
                        info!("Audio callback #{}: received {} samples", CALLBACK_COUNT, data.len());
                    }
                }
                
                if !is_recording.load(Ordering::Relaxed) {
                    return;
                }
                
                if data.is_empty() {
                    warn!("Audio callback received empty data");
                    return;
                }
                
                if let Ok(mut buffer) = samples_arc.lock() {
                    let initial_len = buffer.len();
                    
                    for &sample in data {
                        let f32_sample: f32 = f32::from_sample(sample);
                        buffer.push_back(f32_sample);
                        
                        // Keep buffer size reasonable (about 30 seconds of audio)
                        if buffer.len() > 44100 * 2 * 30 {
                            buffer.pop_front();
                        }
                    }
                    
                    // Log more frequently to show capture is working
                    if buffer.len() > 0 && (buffer.len() - initial_len) > 0 {
                        unsafe {
                            if CALLBACK_COUNT % 100 == 0 {
                                info!("Audio capture active: {} total samples captured", buffer.len());
                            }
                        }
                    }
                } else {
                    warn!("Failed to acquire audio buffer lock");
                }
            },
            |err| error!("Audio stream error: {}", err),
            None,
        ).map_err(|e| anyhow::anyhow!("Failed to build input stream: {}", e))?;
        
        Ok(stream)
    }

}

