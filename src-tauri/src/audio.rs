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
// use crate::windows_audio_capture::{WindowsAudioCapture, test_system_audio_capture}; // Temporarily disabled

static AUDIO_STATE: std::sync::OnceLock<Arc<Mutex<AudioCaptureState>>> = std::sync::OnceLock::new();

struct AudioCaptureState {
    is_recording: bool,
    config: AudioConfig,
    captured_samples: VecDeque<f32>,
    wasapi_loopback: Option<WasapiLoopback>,
    windows_audio_capture: Option<WindowsAudioCapture>, // Native Windows WASAPI capture
    is_mic_recording: bool,
    audio_callback: Option<Box<dyn Fn(Vec<u8>) + Send + Sync>>,
}

fn get_audio_state() -> Arc<Mutex<AudioCaptureState>> {
    AUDIO_STATE.get_or_init(|| {
        Arc::new(Mutex::new(AudioCaptureState {
            is_recording: false,
            config: AudioConfig::default(),
            captured_samples: VecDeque::new(),
            wasapi_loopback: None,
            windows_audio_capture: None,
            is_mic_recording: false,
            audio_callback: None,
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
            sample_rate: 16000,  // Optimized for speech recognition
            channels: 1,         // Mono for better speech processing
            buffer_size: 2048,   // Smaller buffer for lower latency
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

pub fn list_all_audio_devices() -> Result<Vec<AudioDevice>> {
    WasapiLoopback::list_all_devices()
}

pub fn list_input_devices() -> Result<Vec<String>> {
    let devices = list_all_audio_devices()?;
    Ok(devices.into_iter()
        .filter(|d| d.device_type == "input")
        .map(|d| d.name)
        .collect())
}

pub fn list_loopback_devices() -> Result<Vec<AudioDevice>> {
    let devices = list_all_audio_devices()?;
    Ok(devices.into_iter()
        .filter(|d| d.supports_loopback)
        .collect())
}

pub fn get_device_supported_configs(device_name: &str) -> Result<Vec<String>> {
    WasapiLoopback::get_device_supported_configs(device_name)
}

pub fn get_default_device_info() -> Result<String> {
    let host = cpal::default_host();
    
    if let Some(device) = host.default_input_device() {
        let name = device.name().unwrap_or("Unknown".to_string());
        Ok(format!("Default input device: {}", name))
    } else {
        Err(anyhow!("No default input device available"))
    }
}

pub fn list_all_devices() {
    let host = cpal::default_host();
    
    info!("=== Audio Device Information ===");
    
    info!("Input Devices:");
    match host.input_devices() {
        Ok(devices) => {
            let mut count = 0;
            for device in devices {
                count += 1;
                let name = device.name().unwrap_or("Unknown".to_string());
                info!("  [{}] Input: {}", count, name);
                
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

fn get_system_audio_device() -> Result<Device> {
    let host = cpal::default_host();
    
    if let Some(device) = host.default_output_device() {
        info!("Found default output device: {}", device.name().unwrap_or("Unknown".to_string()));
        return Ok(device);
    }
    
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
    
    if let Ok(mut devices) = host.output_devices() {
        while let Some(device) = devices.next() {
            info!("Available output device: {}", device.name().unwrap_or("Unknown".to_string()));
        }
    } else {
        warn!("No output devices found");
    }

    if let Some(device) = host.default_input_device() {
        warn!("Using input device as fallback for system audio capture");
        return Ok(device);
    }
    
    Err(anyhow!("No suitable audio device found for system audio capture"))
}

pub fn capture_audio() -> Result<()> {
    capture_audio_from_device(None, false)
}

pub fn start_system_audio_capture() -> Result<()> {
    info!("Starting system audio capture...");
    let state = get_audio_state();
    let mut state = state.lock().unwrap();
    
    // If already recording, stop first
    if state.is_recording {
        warn!("Audio capture is already running, stopping first");
        drop(state); // Drop lock before calling stop_capture
        stop_capture()?;
        state = get_audio_state().lock().unwrap(); // Re-acquire lock
    }
    
    // First try to find the default output device and use it for loopback
    // This should work on Windows without requiring Stereo Mix
    info!("Attempting direct loopback from default output device...");
    
    let devices = list_all_audio_devices()?;
    
    // Try to find the default output device that supports loopback
    let default_output_device = devices.iter().find(|d| 
        d.device_type == "output" && d.is_default && d.supports_loopback
    );
    
    if let Some(device) = default_output_device {
        info!("✅ Found default output device with loopback support: {}", device.name);
        drop(state); // Drop lock before calling another function
        return capture_audio_from_device(Some(device.name.clone()), false);
    } else {
        info!("⚠️ Default output device doesn't support loopback, trying other methods...");
    }
    
    // Fallback to old method using stereo mix or other devices
    info!("Falling back to Stereo Mix or other loopback devices...");
    
    let devices = list_all_audio_devices()?;
    
    // Try Stereo Mix first (requires it to be enabled)
    if let Some(stereo_mix) = devices.iter().find(|d| 
        d.name.to_lowercase().contains("stereo mix") || 
        d.name.to_lowercase().contains("what u hear")
    ) {
        info!("Found Stereo Mix device: {}", stereo_mix.name);
        drop(state); // Drop lock before calling another function
        return capture_audio_from_device(Some(stereo_mix.name.clone()), false);
    }
    
    // Then try other loopback-capable devices
    let loopback_devices = list_loopback_devices()?;
    if let Some(device) = loopback_devices.first() {
        info!("Using loopback-capable device: {}", device.name);
        drop(state); // Drop lock before calling another function
        return capture_audio_from_device(Some(device.name.clone()), false);
    }
    
    warn!("No Stereo Mix or loopback devices found, using default capture");
    drop(state); // Drop lock before calling another function
    capture_audio_from_device(None, false)
}

pub fn start_microphone_capture() -> Result<()> {
    info!("Starting microphone audio capture...");
    
    let devices = list_all_audio_devices()?;
    let input_devices: Vec<_> = devices.into_iter()
        .filter(|d| d.device_type == "input")
        .collect();
    
    if input_devices.is_empty() {
        return Err(anyhow!("No microphone input devices found"));
    }
    
    if let Some(mic_array) = input_devices.iter().find(|d| d.name.contains("Microphone Array")) {
        info!("Using Microphone Array: {}", mic_array.name);
        return capture_audio_from_device(Some(mic_array.name.clone()), true);
    }

    if let Some(default_mic) = input_devices.iter().find(|d| d.is_default) {
        info!("Using default microphone: {}", default_mic.name);
        return capture_audio_from_device(Some(default_mic.name.clone()), true);
    }
    
    let first_mic = &input_devices[0];
    info!("Using first available microphone: {}", first_mic.name);
    capture_audio_from_device(Some(first_mic.name.clone()), true)
}

pub fn capture_audio_from_device(device_name: Option<String>, is_mic: bool) -> Result<()> {
    let state_arc = get_audio_state();
    let mut state = state_arc.lock().unwrap();
    
    if state.is_recording {
        warn!("Audio capture is already running");
        return Ok(());
    }

    state.is_mic_recording = is_mic;

    if is_mic {
        info!("Starting microphone audio capture...");
    } else {
        info!("Starting system audio capture...");
    }
    
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
    
    match wasapi_loopback.start_capture() {
        Ok(_) => {
            info!("WASAPI capture started successfully");
            
            state.config = AudioConfig {
                sample_rate: wasapi_loopback.get_sample_rate(),
                channels: wasapi_loopback.get_channels(),
                buffer_size: 4096,
            };
            
            state.is_recording = true;
            state.wasapi_loopback = Some(wasapi_loopback);
            
            info!("Audio capture started: {} Hz, {} channels", 
                  state.config.sample_rate, state.config.channels);

            // Start a background thread to handle audio processing
            let audio_state = get_audio_state();
            std::thread::spawn(move || {
                loop {
                    let (is_recording, callback) = {
                        let mut state = audio_state.lock().unwrap();
                        (state.is_recording, state.audio_callback.take())
                    };

                    if !is_recording {
                        break;
                    }

                    let samples = {
                        let mut state = audio_state.lock().unwrap();
                        if let Some(ref mut wasapi_loopback) = state.wasapi_loopback {
                            wasapi_loopback.get_captured_samples()
                        } else {
                            Vec::new()
                        }
                    };

                    if !samples.is_empty() {
                        let audio_data = f32_to_i16_bytes(&samples);
                        if let Some(callback) = &callback {
                            callback(audio_data);
                        }
                    }

                    if let Some(callback) = callback {
                        let mut state = audio_state.lock().unwrap();
                        state.audio_callback = Some(callback);
                    }

                    std::thread::sleep(Duration::from_millis(100));
                }
            });
            
            Ok(())
        }
        Err(e) => {
            error!("Failed to start WASAPI capture: {}", e);
            Err(e)
        }
    }
}

pub fn get_captured_samples() -> Vec<f32> {
    let state = get_audio_state();
    let state = state.lock().unwrap();
    
    // First try to get samples from native Windows capture
    if let Some(ref windows_capture) = state.windows_audio_capture {
        let samples = windows_capture.get_samples();
        if !samples.is_empty() {
            return samples;
        }
    }
    
    // Fall back to wasapi_loopback if Windows capture is not active
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
    
    // Stop Windows WASAPI capture if active
    if let Some(ref mut windows_capture) = state.windows_audio_capture {
        match windows_capture.stop_capture() {
            Ok(_) => info!("Windows WASAPI capture stopped successfully"),
            Err(e) => error!("Failed to stop Windows WASAPI capture: {}", e),
        }
    }
    
    // Stop regular WASAPI loopback capture if active
    if let Some(ref mut wasapi_loopback) = state.wasapi_loopback {
        match wasapi_loopback.stop_capture() {
            Ok(_) => info!("WASAPI loopback capture stopped successfully"),
            Err(e) => error!("Failed to stop WASAPI loopback capture: {}", e),
        }
    }
    
    state.is_recording = false;
    
    info!("Audio capture stopped successfully");
    Ok(())
}

pub fn cleanup_audio_capture() {
    let state = get_audio_state();
    let mut state = state.lock().unwrap();
    
    state.wasapi_loopback = None;
    state.windows_audio_capture = None;
    info!("Audio capture cleanup completed");
}

pub fn is_recording() -> bool {
    let state = get_audio_state();
    let guard = state.lock().unwrap();
    guard.is_recording
}

pub fn is_mic_recording() -> bool {
    let state = get_audio_state();
    let guard = state.lock().unwrap();
    guard.is_mic_recording
}

pub fn set_audio_callback<F>(callback: F) 
where 
    F: Fn(Vec<u8>) + Send + Sync + 'static 
{
    let state = get_audio_state();
    let mut guard = state.lock().unwrap();
    guard.audio_callback = Some(Box::new(callback));
}

pub fn get_audio_config() -> AudioConfig {
    let state = get_audio_state();
    let guard = state.lock().unwrap();
    guard.config.clone()
}

fn f32_to_i16_bytes(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        let i16_sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&i16_sample.to_le_bytes());
    }
    bytes
}

pub fn audio_data_to_base64_wav(audio_data: &AudioData) -> Result<String> {
    let mut wav_data = Vec::new();
    
    let data_size = (audio_data.samples.len() * 2) as u32;
    let file_size = 36 + data_size;
    
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&file_size.to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");
    
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes());
    wav_data.extend_from_slice(&1u16.to_le_bytes());
    wav_data.extend_from_slice(&audio_data.channels.to_le_bytes());
    wav_data.extend_from_slice(&audio_data.sample_rate.to_le_bytes());
    let byte_rate = audio_data.sample_rate * audio_data.channels as u32 * 2;
    wav_data.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = audio_data.channels * 2;
    wav_data.extend_from_slice(&block_align.to_le_bytes());
    wav_data.extend_from_slice(&16u16.to_le_bytes());
    
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&data_size.to_le_bytes());
    
    for &sample in &audio_data.samples {
        let i16_sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        wav_data.extend_from_slice(&i16_sample.to_le_bytes());
    }
    
    Ok(BASE64_STANDARD.encode(wav_data))
}

pub fn test_capture_audio(duration_secs: u64) -> Result<()> {
    info!("Starting test audio capture for {} seconds", duration_secs);
    
    set_audio_callback(|audio_data| {
        debug!("Audio data: {} bytes", audio_data.len());
    });
    
    capture_audio()?;
    
    thread::sleep(Duration::from_secs(duration_secs));
    
    stop_capture()?;
    
    info!("Test audio capture completed");
    Ok(())
}

/// Test the native Windows WASAPI system audio capture directly
pub fn test_native_windows_system_audio_capture(duration_secs: u64) -> Result<String> {
    info!("🧪 Testing native Windows WASAPI system audio capture for {} seconds...", duration_secs);
    
    #[cfg(target_os = "windows")]
    {
        let mut windows_capture = WindowsAudioCapture::new();
        
        // Start capture
        windows_capture.start_system_audio_capture()
            .map_err(|e| anyhow!("Failed to start Windows WASAPI capture: {}", e))?;
        
        info!("⏱️ Recording system audio for {} seconds...", duration_secs);
        thread::sleep(Duration::from_secs(duration_secs));
        
        // Get samples
        let samples = windows_capture.get_samples();
        let sample_count = samples.len();
        
        // Stop capture
        windows_capture.stop_capture()
            .map_err(|e| anyhow!("Failed to stop Windows WASAPI capture: {}", e))?;
        
        if sample_count > 0 {
            // Calculate some basic audio statistics
            let max_amplitude = samples.iter().map(|&s| s.abs()).fold(0.0f32, |a, b| a.max(b));
            let avg_amplitude = samples.iter().map(|&s| s.abs()).sum::<f32>() / sample_count as f32;
            
            let result_msg = format!(
                "✅ Native Windows WASAPI capture successful!\n📊 Captured {} samples ({:.2} seconds of audio)\n🔊 Max amplitude: {:.6}\n📈 Avg amplitude: {:.6}\n🎵 Sample rate: {} Hz, Channels: {}",
                sample_count,
                sample_count as f32 / (windows_capture.get_sample_rate() * windows_capture.get_channels() as u32) as f32,
                max_amplitude,
                avg_amplitude,
                windows_capture.get_sample_rate(),
                windows_capture.get_channels()
            );
            
            info!("{}", result_msg);
            Ok(result_msg)
        } else {
            let error_msg = "❌ No audio samples captured - system may be silent or capture failed";
            error!("{}", error_msg);
            Err(anyhow!(error_msg))
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let error_msg = "❌ Native Windows WASAPI capture is only supported on Windows";
        error!("{}", error_msg);
        Err(anyhow!(error_msg))
    }
}
