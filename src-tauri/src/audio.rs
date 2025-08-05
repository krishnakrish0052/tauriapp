use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;
use log::{info, error, warn, debug};
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use std::time::Duration;
use std::thread;
use base64::prelude::*;
use std::collections::VecDeque;
use crate::wasapi_loopback::{WasapiLoopback, AudioDevice};

// Global state for audio capture - now using WASAPI
static AUDIO_STATE: std::sync::OnceLock<Arc<Mutex<AudioCaptureState>>> = std::sync::OnceLock::new();

struct AudioCaptureState {
    is_recording: bool,
    config: AudioConfig,
    captured_samples: VecDeque<f32>,
    wasapi_loopback: Option<WasapiLoopback>,
}

fn get_audio_state() -> Arc<Mutex<AudioCaptureState>> {
    AUDIO_STATE.get_or_init(|| {
        Arc::new(Mutex::new(AudioCaptureState {
            is_recording: false,
            config: AudioConfig::default(),
            captured_samples: VecDeque::new(),
            wasapi_loopback: None,
        }))
    }).clone()
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            buffer_size: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub timestamp: std::time::SystemTime,
}

// Simplified audio functions to avoid complex state management

/// List all available audio devices (both input and output)
pub fn list_all_audio_devices() -> Result<Vec<AudioDevice>> {
    WasapiLoopback::list_all_devices()
}

/// List available audio input devices (legacy function for compatibility)
pub fn list_input_devices() -> Result<Vec<String>> {
    let devices = list_all_audio_devices()?;
    Ok(devices.into_iter()
        .filter(|d| d.device_type == "input")
        .map(|d| d.name)
        .collect())
}

/// List devices that support loopback capture (for system audio)
pub fn list_loopback_devices() -> Result<Vec<AudioDevice>> {
    let devices = list_all_audio_devices()?;
    Ok(devices.into_iter()
        .filter(|d| d.supports_loopback)
        .collect())
}

/// Get detailed supported configurations for a specific device
pub fn get_device_supported_configs(device_name: &str) -> Result<Vec<String>> {
    WasapiLoopback::get_device_supported_configs(device_name)
}


/// Get basic device information
pub fn get_default_device_info() -> Result<String> {
    let host = cpal::default_host();
    
    if let Some(device) = host.default_input_device() {
        let name = device.name().unwrap_or("Unknown".to_string());
        Ok(format!("Default input device: {}", name))
    } else {
        Err(anyhow!("No default input device available"))
    }
}

/// List all available audio devices (input and output) on startup
pub fn list_all_devices() {
    let host = cpal::default_host();
    
    info!("=== Audio Device Information ===");
    
    // List input devices
    info!("Input Devices:");
    match host.input_devices() {
        Ok(devices) => {
            let mut count = 0;
            for device in devices {
                count += 1;
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("  [{}] Input: {}", count, name);
                
                // Try to get supported configurations
                match device.supported_input_configs() {
                    Ok(configs) => {
                        let mut config_count = 0;
                        for config in configs {
                            config_count += 1;
                            info!("    - Sample rate: {}-{} Hz", config.min_sample_rate().0, config.max_sample_rate().0);
                            info!("    - Channels: {}", config.channels());
                            info!("    - Sample format: {:?}", config.sample_format());
                        }
                        if config_count == 0 {
                            warn!("    - No supported input configurations found");
                        }
                    }
                    Err(e) => {
                        warn!("    - Failed to get input configs: {}", e);
                    }
                }
            }
            if count == 0 {
                warn!("  No input devices found");
            }
        }
        Err(e) => {
            error!("Failed to enumerate input devices: {}", e);
        }
    }
    
    // List output devices
    info!("Output Devices:");
    match host.output_devices() {
        Ok(devices) => {
            let mut count = 0;
            for device in devices {
                count += 1;
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("  [{}] Output: {}", count, name);
                match device.supported_output_configs() {
                    Ok(configs) => {
                        for config in configs {
                            info!("    - Supported sample rate: {}-{} Hz", config.min_sample_rate().0, config.max_sample_rate().0);
                            info!("    - Channels: {}", config.channels());
                            info!("    - Sample format: {:?}", config.sample_format());
                        }
                    }
                    Err(e) => {
                        warn!("    - Failed to get output configs: {}", e);
                    }
                }
                
                // Also check if this output device supports input (for loopback)
                match device.supported_input_configs() {
                    Ok(mut configs) => {
                        if let Some(config) = configs.next() {
                            info!("    - LOOPBACK CAPABLE: Sample rate: {}-{} Hz", config.min_sample_rate().0, config.max_sample_rate().0);
                        }
                    }
                    Err(_) => {
                        info!("    - No loopback capability");
                    }
                }
            }
            if count == 0 {
                warn!("  No output devices found");
            }
        }
        Err(e) => {
            error!("Failed to enumerate output devices: {}", e);
        }
    }
    
    // Check default devices
    if let Some(device) = host.default_input_device() {
        info!("Default input device: {}", device.name().unwrap_or("Unknown".to_string()));
    } else {
        warn!("No default input device");
    }
    
    if let Some(device) = host.default_output_device() {
        info!("Default output device: {}", device.name().unwrap_or("Unknown".to_string()));
    } else {
        warn!("No default output device");
    }
    
    info!("=== End Audio Device Information ===");
}

/// Try to get system audio loopback device (Windows WASAPI)
fn get_system_audio_device() -> Result<Device> {
    let host = cpal::default_host();
    
    // First try to get the default output device for loopback capture
    if let Some(device) = host.default_output_device() {
        info!("Found default output device: {}", device.name().unwrap_or("Unknown".to_string()));
        return Ok(device);
    }
    
    // Try to enumerate all output devices and pick the first one
    match host.output_devices() {
        Ok(mut devices) => {
            if let Some(device) = devices.next() {
                info!("Using first available output device: {}", device.name().unwrap_or("Unknown".to_string()));
                return Ok(device);
            }
        }
        Err(e) => {
            warn!("Failed to enumerate output devices: {}", e);
        }
    }
    
    // Log all available output devices
    if let Ok(mut devices) = host.output_devices() {
        while let Some(device) = devices.next() {
            info!("Available output device: {}", device.name().unwrap_or("Unknown".to_string()));
        }
    } else {
        warn!("No output devices found");
    }

    // Fallback to input device if no output device available
    if let Some(device) = host.default_input_device() {
        warn!("Using input device as fallback for system audio capture");
        return Ok(device);
    }
    
    Err(anyhow!("No suitable audio device found for system audio capture"))
}

/// Start capturing system audio using WASAPI loopback
pub fn capture_audio() -> Result<()> {
    capture_audio_from_device(None, false)
}

/// Start capturing system audio specifically (for system audio button)
pub fn start_system_audio_capture() -> Result<()> {
    info!("Starting system audio capture...");
    
    // Try to find "Stereo Mix" or other loopback-capable devices first
    let devices = list_all_audio_devices()?;
    
    // Look for Stereo Mix device
    if let Some(stereo_mix) = devices.iter().find(|d| 
        d.name.to_lowercase().contains("stereo mix") || 
        d.name.to_lowercase().contains("what u hear")
    ) {
        info!("Found Stereo Mix device: {}", stereo_mix.name);
        return capture_audio_from_device(Some(stereo_mix.name.clone()), false);
    }
    
    // Fallback to loopback-capable devices
    let loopback_devices = list_loopback_devices()?;
    if let Some(device) = loopback_devices.first() {
        info!("Using loopback-capable device: {}", device.name);
        return capture_audio_from_device(Some(device.name.clone()), false);
    }
    
    // Final fallback - use default system audio capture
    warn!("No Stereo Mix or loopback devices found, using default capture");
    capture_audio_from_device(None, false)
}

/// Start capturing microphone audio specifically (for microphone button)
pub fn start_microphone_capture() -> Result<()> {
    info!("Starting microphone audio capture...");
    
    // Get all input devices
    let devices = list_all_audio_devices()?;
    let input_devices: Vec<_> = devices.into_iter()
        .filter(|d| d.device_type == "input")
        .collect();
    
    if input_devices.is_empty() {
        return Err(anyhow!("No microphone input devices found"));
    }
    
    // Try to find default microphone or use the first available input device
    if let Some(default_mic) = input_devices.iter().find(|d| d.is_default) {
        info!("Using default microphone: {}", default_mic.name);
        return capture_audio_from_device(Some(default_mic.name.clone()), true);
    }
    
    // Use first available input device
    let first_mic = &input_devices[0];
    info!("Using first available microphone: {}", first_mic.name);
    capture_audio_from_device(Some(first_mic.name.clone()), true)
}

/// Start capturing audio from a specific device
pub fn capture_audio_from_device(device_name: Option<String>, is_mic: bool) -> Result<()> {
    let state_arc = get_audio_state();
    let mut state = state_arc.lock().unwrap();
    
    if state.is_recording {
        warn!("Audio capture is already running");
        return Ok(());
    }
    
    if is_mic {
        info!("Starting microphone audio capture...");
    } else {
        info!("Starting system audio capture...");
    }
    
    // Create WASAPI loopback instance with or without specific device
    let mut wasapi_loopback = if is_mic {
        if let Some(device) = device_name {
            info!("Using specific input device for microphone: {}", device);
            WasapiLoopback::new_for_microphone(Some(device))
        } else {
            info!("Using default input device for microphone");
            WasapiLoopback::new_for_microphone(None)
        }
    } else {
        if let Some(device) = device_name {
            info!("Using specific device for system audio: {}", device);
            WasapiLoopback::new_with_device(device)
        } else {
            info!("Using default device selection for system audio");
            WasapiLoopback::new()
        }
    };
    
    // Start WASAPI capture
    match wasapi_loopback.start_capture() {
        Ok(_) => {
            info!("WASAPI capture started successfully");
            
            // Update state config with WASAPI settings
            state.config = AudioConfig {
                sample_rate: wasapi_loopback.get_sample_rate(),
                channels: wasapi_loopback.get_channels(),
                buffer_size: 4096,
            };
            
            state.is_recording = true;
            state.wasapi_loopback = Some(wasapi_loopback);
            
            info!("Audio capture started: {} Hz, {} channels", 
                  state.config.sample_rate, state.config.channels);
            
            Ok(())
        }
        Err(e) => {
            error!("Failed to start WASAPI capture: {}", e);
            Err(e)
        }
    }
}

/// Get captured audio samples from WASAPI loopback
pub fn get_captured_samples() -> Vec<f32> {
    let state = get_audio_state();
    let state = state.lock().unwrap();
    
    if let Some(ref wasapi_loopback) = state.wasapi_loopback {
        wasapi_loopback.get_captured_samples()
    } else {
        Vec::new()
    }
}


pub fn capture_audio_with_config(config: AudioConfig) -> Result<()> {
    let state = get_audio_state();
    let mut state = state.lock().unwrap();
    
    if state.is_recording {
        warn!("Audio capture is already running");
        return Ok(());
    }
    
    state.is_recording = true;
    state.config = config;
    info!("Audio capture started with custom config (mock implementation)");
    Ok(())
}

pub fn stop_capture() -> Result<()> {
    let state = get_audio_state();
    let mut state = state.lock().unwrap();
    
    if !state.is_recording {
        warn!("Audio capture is not running");
        return Ok(());
    }
    
    // Stop WASAPI loopback capture but keep the instance for sample retrieval
    if let Some(ref mut wasapi_loopback) = state.wasapi_loopback {
        match wasapi_loopback.stop_capture() {
            Ok(_) => info!("WASAPI loopback capture stopped successfully"),
            Err(e) => error!("Failed to stop WASAPI loopback capture: {}", e),
        }
    }
    
    state.is_recording = false;
    // Don't set wasapi_loopback to None yet - we need it for sample retrieval
    
    info!("Audio capture stopped successfully");
    Ok(())
}

/// Clean up the WASAPI loopback instance after audio file operations
pub fn cleanup_audio_capture() {
    let state = get_audio_state();
    let mut state = state.lock().unwrap();
    
    // Now it's safe to clean up the WASAPI loopback instance
    state.wasapi_loopback = None;
    info!("Audio capture cleanup completed");
}

pub fn is_recording() -> bool {
    let state = get_audio_state();
    let guard = state.lock().unwrap();
    guard.is_recording
}

pub fn set_audio_callback<F>(_callback: F) 
where 
    F: Fn(AudioData) + Send + Sync + 'static 
{
    info!("Audio callback set (mock implementation)");
}

pub fn get_audio_config() -> AudioConfig {
    let state = get_audio_state();
    let guard = state.lock().unwrap();
    guard.config.clone()
}

/// Convert audio data to base64 encoded WAV format
pub fn audio_data_to_base64_wav(audio_data: &AudioData) -> Result<String> {
    let mut wav_data = Vec::new();
    
    // WAV header
    let data_size = (audio_data.samples.len() * 2) as u32; // 16-bit samples
    let file_size = 36 + data_size;
    
    // RIFF header
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&file_size.to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");
    
    // Format chunk
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav_data.extend_from_slice(&audio_data.channels.to_le_bytes());
    wav_data.extend_from_slice(&audio_data.sample_rate.to_le_bytes());
    let byte_rate = audio_data.sample_rate * audio_data.channels as u32 * 2;
    wav_data.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = audio_data.channels * 2;
    wav_data.extend_from_slice(&block_align.to_le_bytes());
    wav_data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    
    // Data chunk
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&data_size.to_le_bytes());
    
    // Convert f32 samples to i16 and add to WAV data
    for &sample in &audio_data.samples {
        let i16_sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        wav_data.extend_from_slice(&i16_sample.to_le_bytes());
    }
    
    Ok(BASE64_STANDARD.encode(wav_data))
}

/// Test function to capture audio for a specified duration
pub fn test_capture_audio(duration_secs: u64) -> Result<()> {
    info!("Starting test audio capture for {} seconds", duration_secs);
    
    // Set up a callback to log audio data
    set_audio_callback(|audio_data| {
        let avg_amplitude = audio_data.samples.iter()
            .map(|&s| s.abs())
            .sum::<f32>() / audio_data.samples.len() as f32;
        
        debug!("Audio data: {} samples, avg amplitude: {:.4}", 
               audio_data.samples.len(), avg_amplitude);
    });
    
    // Start capture
    capture_audio()?;
    
    // Wait for specified duration
    thread::sleep(Duration::from_secs(duration_secs));
    
    // Stop capture
    stop_capture()?;
    
    info!("Test audio capture completed");
    Ok(())
}
