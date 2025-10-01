//! Native Windows WASAPI loopback audio capture
//! 
//! This module provides direct system audio capture on Windows using WASAPI loopback mode,
//! which captures all system audio without requiring Stereo Mix or any special configuration.
//! 
//! WASAPI loopback allows capturing the audio that's being played by all applications
//! on the system, which is exactly what we need for system sound capture.

use anyhow::{Result, anyhow};
use log::{info, error, warn};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::collections::VecDeque;
use std::time::Duration;
use std::thread;

#[cfg(windows)]
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Media::Audio::*,
        System::Com::*,
        System::Ole::*,
        UI::Shell::PropertiesSystem::*,
    },
};

#[cfg(windows)]
use winapi::shared::wtypes::VT_LPWSTR;
#[cfg(windows)] 
use winapi::um::combaseapi::PropVariantClear;
#[cfg(windows)]
use winapi::shared::mmreg::WAVE_FORMAT_IEEE_FLOAT;

#[cfg(windows)]
// Define the PKEY_Device_FriendlyName property key
const PKEY_Device_FriendlyName: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

/// Windows WASAPI loopback audio capturer
pub struct WindowsAudioCapture {
    is_recording: Arc<AtomicBool>,
    audio_samples: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
    channels: u16,
    format: AudioFormat,
}

#[derive(Debug, Clone)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            bits_per_sample: 32, // 32-bit float for WASAPI
        }
    }
}

impl WindowsAudioCapture {
    /// Create a new Windows audio capture instance
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            audio_samples: Arc::new(Mutex::new(VecDeque::new())),
            sample_rate: 44100,
            channels: 2,
            format: AudioFormat::default(),
        }
    }

    /// Start capturing system audio using WASAPI loopback
    #[cfg(windows)]
    pub fn start_system_audio_capture(&mut self) -> Result<()> {
        if self.is_recording.load(Ordering::Relaxed) {
            info!("WASAPI system audio capture already running");
            return Ok(());
        }

        info!("🎵 Starting WASAPI system audio capture...");
        
        // Initialize COM
        unsafe {
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            if hr.is_err() && hr != HRESULT(0x80010106u32 as i32) { // RPC_E_CHANGED_MODE - already initialized
                return Err(anyhow!("Failed to initialize COM: {:?}", hr));
            }
        }

        // Get the default audio endpoint (speakers/headphones)
        let device_enumerator = self.create_device_enumerator()?;
        let default_device = self.get_default_audio_device(&device_enumerator)?;
        
        // Get the device name for logging
        let device_name = self.get_device_name(&default_device)?;
        info!("Using audio device: {}", device_name);

        // Activate the audio client for loopback capture
        let audio_client = self.activate_audio_client(&default_device)?;
        
        // Get the device's current format
        let mix_format = self.get_mix_format(&audio_client)?;
        info!("Device format: {} Hz, {} channels", mix_format.sample_rate, mix_format.channels);

        // Initialize the audio client in loopback mode
        self.initialize_audio_client_loopback(&audio_client, &mix_format)?;
        
        // Get the capture client
        let capture_client = self.get_capture_client(&audio_client)?;
        
        // Update our format info
        self.sample_rate = mix_format.sample_rate;
        self.channels = mix_format.channels;
        self.format = mix_format;

        // Start the audio client
        unsafe {
            audio_client.Start()?;
        }

        self.is_recording.store(true, Ordering::Relaxed);

        // Start capture thread with COM initialization inside the thread
        let is_recording = self.is_recording.clone();
        let audio_samples = self.audio_samples.clone();
        let channels = self.channels;
        let format = self.format.clone();
        
        thread::spawn(move || {
            // Initialize COM in this thread
            unsafe {
                let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
                if hr.is_err() && hr != HRESULT(0x80010106u32 as i32) {
                    error!("Failed to initialize COM in capture thread: {:?}", hr);
                    return;
                }
            }
            
            // Create device enumerator and audio client in this thread
            let result = Self::run_capture_thread_internal(is_recording, audio_samples, channels, format);
            if let Err(e) = result {
                error!("WASAPI capture thread error: {}", e);
            }
            
            // Cleanup COM
            unsafe {
                CoUninitialize();
            }
        });
        
        info!("✅ WASAPI system audio capture started successfully");
        Ok(())
    }
    
    fn run_capture_thread_internal(
        is_recording: Arc<AtomicBool>,
        audio_samples: Arc<Mutex<VecDeque<f32>>>,
        channels: u16,
        format: AudioFormat,
    ) -> Result<()> {
        info!("🎵 WASAPI capture thread started");
        
        // Create all COM objects within this thread
        let device_enumerator = unsafe {
            CoCreateInstance::<_, IMMDeviceEnumerator>(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL,
            ).map_err(|e| anyhow!("Failed to create device enumerator: {}", e))?
        };
        
        let default_device = unsafe {
            device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| anyhow!("Failed to get default audio device: {}", e))?
        };
        
        let audio_client = unsafe {
            default_device.Activate::<IAudioClient>(
                CLSCTX_ALL,
                None,
            ).map_err(|e| anyhow!("Failed to activate audio client: {}", e))?
        };
        
        // Get mix format and initialize client
        let mix_format_ptr = unsafe { audio_client.GetMixFormat()? };
        let wave_format = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_IEEE_FLOAT as u16,
            nChannels: channels,
            nSamplesPerSec: format.sample_rate,
            nAvgBytesPerSec: format.sample_rate * channels as u32 * 4,
            nBlockAlign: channels * 4,
            wBitsPerSample: 32,
            cbSize: 0,
        };
        
        unsafe {
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                10_000_000, // 1 second buffer
                0,
                &wave_format,
                None,
            ).map_err(|e| anyhow!("Failed to initialize audio client: {}", e))?;
            
            CoTaskMemFree(Some(mix_format_ptr as *const _ as *const _));
        }
        
        let capture_client = unsafe {
            audio_client.GetService::<IAudioCaptureClient>()
                .map_err(|e| anyhow!("Failed to get capture client: {}", e))?
        };
        
        unsafe { audio_client.Start()? };
        
        let mut buffer_frame_count: u32 = 0;
        let mut packet_length: u32 = 0;
        
        while is_recording.load(Ordering::Relaxed) {
            unsafe {
                if let Ok(next_packet_size) = capture_client.GetNextPacketSize() {
                    packet_length = next_packet_size;
                } else {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
            }
            
            while packet_length != 0 {
                let mut buffer_data: *mut u8 = std::ptr::null_mut();
                let mut flags: u32 = 0;
                
                unsafe {
                    match capture_client.GetBuffer(
                        &mut buffer_data,
                        &mut buffer_frame_count,
                        &mut flags,
                        None,
                        None,
                    ) {
                        Ok(_) => {
                            if !buffer_data.is_null() && buffer_frame_count > 0 {
                                let sample_count = (buffer_frame_count * channels as u32) as usize;
                                let samples: &[f32] = std::slice::from_raw_parts(
                                    buffer_data as *const f32,
                                    sample_count
                                );
                                
                                if let Ok(mut sample_buffer) = audio_samples.lock() {
                                    for &sample in samples {
                                        sample_buffer.push_back(sample);
                                        if sample_buffer.len() > 44100 * 2 * 30 {
                                            sample_buffer.pop_front();
                                        }
                                    }
                                }
                            }
                            
                            capture_client.ReleaseBuffer(buffer_frame_count).ok();
                        }
                        Err(e) => {
                            error!("Failed to get audio buffer: {}", e);
                            break;
                        }
                    }
                }
                
                unsafe {
                    if let Ok(next_packet_size) = capture_client.GetNextPacketSize() {
                        packet_length = next_packet_size;
                    } else {
                        break;
                    }
                }
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        unsafe {
            audio_client.Stop().ok();
        }
        
        info!("🎵 WASAPI capture thread stopped");
        Ok(())
    }

    /// Stop the audio capture
    pub fn stop_capture(&mut self) -> Result<()> {
        if !self.is_recording.load(Ordering::Relaxed) {
            info!("WASAPI audio capture is not running");
            return Ok(());
        }

        info!("🛑 Stopping WASAPI system audio capture...");
        self.is_recording.store(false, Ordering::Relaxed);
        
        // Give the thread time to stop
        thread::sleep(Duration::from_millis(100));

        info!("✅ WASAPI system audio capture stopped");
        Ok(())
    }

    /// Get captured audio samples
    pub fn get_samples(&self) -> Vec<f32> {
        if let Ok(samples) = self.audio_samples.lock() {
            samples.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Clear captured samples
    pub fn clear_samples(&self) {
        if let Ok(mut samples) = self.audio_samples.lock() {
            samples.clear();
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    /// Get the sample rate
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the channel count
    pub fn get_channels(&self) -> u16 {
        self.channels
    }

    /// Get the audio format
    pub fn get_format(&self) -> &AudioFormat {
        &self.format
    }

    // Windows-specific WASAPI implementation methods
    #[cfg(windows)]
    fn create_device_enumerator(&self) -> Result<IMMDeviceEnumerator> {
        unsafe {
            CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL,
            ).map_err(|e| anyhow!("Failed to create device enumerator: {}", e))
        }
    }

    #[cfg(windows)]
    fn get_default_audio_device(&self, enumerator: &IMMDeviceEnumerator) -> Result<IMMDevice> {
        unsafe {
            enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| anyhow!("Failed to get default audio device: {}", e))
        }
    }

    #[cfg(windows)]
    fn get_device_name(&self, device: &IMMDevice) -> Result<String> {
        unsafe {
            let property_store: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
            
            match property_store.GetValue(&PKEY_Device_FriendlyName) {
                Ok(prop_value) => {
                    // Try to convert PROPVARIANT to string using the new Windows crate API
                    let name = prop_value.to_string();
                    if name.is_empty() {
                        warn!("Device name is empty, using fallback");
                        Ok("Unknown Device".to_string())
                    } else {
                        Ok(name)
                    }
                }
                Err(_) => Ok("Unknown Device".to_string())
            }
        }
    }

    #[cfg(windows)]
    fn activate_audio_client(&self, device: &IMMDevice) -> Result<IAudioClient> {
        unsafe {
            device.Activate::<IAudioClient>(
                CLSCTX_ALL,
                None,
            ).map_err(|e| anyhow!("Failed to activate audio client: {}", e))
        }
    }

    #[cfg(windows)]
    fn get_mix_format(&self, audio_client: &IAudioClient) -> Result<AudioFormat> {
        unsafe {
            let mix_format_ptr = audio_client.GetMixFormat()?;
            let mix_format = &*mix_format_ptr;

            let format = AudioFormat {
                sample_rate: mix_format.nSamplesPerSec,
                channels: mix_format.nChannels,
                bits_per_sample: mix_format.wBitsPerSample,
            };

            CoTaskMemFree(Some(mix_format_ptr as *const _ as *const _));
            Ok(format)
        }
    }

    #[cfg(windows)]
    fn initialize_audio_client_loopback(&self, audio_client: &IAudioClient, format: &AudioFormat) -> Result<()> {
        unsafe {
            // Create WAVEFORMATEX for loopback capture
            let wave_format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_IEEE_FLOAT as u16,
                nChannels: format.channels,
                nSamplesPerSec: format.sample_rate,
                nAvgBytesPerSec: format.sample_rate * format.channels as u32 * 4, // 4 bytes per sample (32-bit float)
                nBlockAlign: format.channels * 4, // 4 bytes per sample
                wBitsPerSample: 32, // 32-bit float
                cbSize: 0,
            };

            // Initialize the audio client in loopback mode
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                10_000_000, // 1 second buffer
                0,
                &wave_format,
                None,
            ).map_err(|e| anyhow!("Failed to initialize audio client in loopback mode: {}", e))
        }
    }

    #[cfg(windows)]
    fn get_capture_client(&self, audio_client: &IAudioClient) -> Result<IAudioCaptureClient> {
        unsafe {
            audio_client.GetService()
                .map_err(|e| anyhow!("Failed to get capture client: {}", e))
        }
    }

    // Non-Windows stub implementation
    #[cfg(not(windows))]
    pub fn start_system_audio_capture(&mut self) -> Result<()> {
        Err(anyhow!("Windows WASAPI capture is only supported on Windows"))
    }
}

impl Drop for WindowsAudioCapture {
    fn drop(&mut self) {
        if self.is_recording() {
            let _ = self.stop_capture();
        }
        
        #[cfg(windows)]
        unsafe {
            CoUninitialize();
        }
    }
}

/// Test the Windows WASAPI capture functionality
pub fn test_system_audio_capture() -> Result<()> {
    info!("🧪 Testing Windows WASAPI system audio capture...");
    
    let mut capture = WindowsAudioCapture::new();
    
    // Start capture
    capture.start_system_audio_capture()?;
    
    // Capture for 5 seconds
    info!("Recording for 5 seconds...");
    thread::sleep(Duration::from_secs(5));
    
    // Get samples
    let samples = capture.get_samples();
    info!("Captured {} audio samples", samples.len());
    
    // Stop capture
    capture.stop_capture()?;
    
    if samples.len() > 0 {
        info!("✅ Windows WASAPI system audio capture test successful!");
        
        // Calculate some basic audio statistics
        let max_amplitude = samples.iter().map(|&s| s.abs()).fold(0.0f32, |a, b| a.max(b));
        info!("Max amplitude: {:.6}", max_amplitude);
        
        Ok(())
    } else {
        Err(anyhow!("❌ No audio samples captured - system may be silent or capture failed"))
    }
}