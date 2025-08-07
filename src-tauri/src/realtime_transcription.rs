use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use crossbeam::channel::RecvError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream};
use futures::channel::mpsc::{self, Receiver as FuturesReceiver};
use futures::stream::StreamExt;
use futures::SinkExt;
use log::{error, info, warn};
use parking_lot::Mutex;
use tokio::sync::Mutex as TokioMutex;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;
use std::sync::atomic::{AtomicBool, Ordering};

use deepgram::common::options::Encoding;
use deepgram::Deepgram;

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub is_microphone: bool, // true for mic, false for system audio
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            is_microphone: false,
        }
    }
}

/// Real-time transcription service that combines audio capture with Deepgram streaming
pub struct RealTimeTranscription {
    is_running: Arc<Mutex<bool>>,
    config: AudioConfig,
    app_handle: AppHandle,
    shutdown_tx: Option<oneshot::Sender<()>>,
    audio_stop_signal: Arc<AtomicBool>,
}

impl RealTimeTranscription {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            config: AudioConfig::default(),
            app_handle,
            shutdown_tx: None,
            audio_stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start real-time transcription with the specified configuration
    pub async fn start(&mut self, config: AudioConfig, api_key: String) -> Result<()> {
        let mut is_running = self.is_running.lock();
        if *is_running {
            warn!("Real-time transcription is already running");
            return Ok(());
        }

        info!("Starting real-time transcription with config: {:?}", config);
        *is_running = true;
        drop(is_running);

        // Reset stop signal for new session
        self.audio_stop_signal.store(false, Ordering::Relaxed);

        // Clone config before moving it
        let config_clone = config.clone();
        self.config = config_clone.clone();

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Clone necessary data for the async task
        let app_handle = self.app_handle.clone();
        let is_running_arc = self.is_running.clone();
        let audio_stop_signal = self.audio_stop_signal.clone();

        // Spawn the main transcription task
        let _handle = tokio::spawn(async move {
            if let Err(e) = Self::run_transcription_task(config, api_key, app_handle.clone(), shutdown_rx, audio_stop_signal).await {
                error!("Transcription task failed: {}", e);
                let _ = app_handle.emit("transcription-error", json!({
                    "error": e.to_string()
                }));
            }
            
            // Mark as not running when task completes
            *is_running_arc.lock() = false;
            
            let _ = app_handle.emit("transcription-status", json!({
                "status": "stopped"
            }));
        });

        // Emit started status using the cloned config
        let _ = self.app_handle.emit("transcription-status", json!({
            "status": "started",
            "config": {
                "sample_rate": config_clone.sample_rate,
                "channels": config_clone.channels,
                "is_microphone": config_clone.is_microphone
            }
        }));

        Ok(())
    }

    /// Stop real-time transcription
    pub fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.lock();
        if !*is_running {
            warn!("Real-time transcription is not running");
            return Ok(());
        }

        info!("Stopping real-time transcription");
        *is_running = false;

        // Signal audio capture thread to stop
        self.audio_stop_signal.store(true, Ordering::Relaxed);

        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        Ok(())
    }

    /// Check if transcription is currently running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock()
    }

    /// Main transcription task that runs the entire pipeline
    async fn run_transcription_task(
        config: AudioConfig,
        api_key: String,
        app_handle: AppHandle,
        mut shutdown_rx: oneshot::Receiver<()>,
        audio_stop_signal: Arc<AtomicBool>,
    ) -> Result<()> {
        info!("Initializing Deepgram client");
        let dg_client = Deepgram::new(&api_key)
            .map_err(|e| anyhow::anyhow!("Failed to create Deepgram client: {}", e))?;

        info!("Starting audio stream");
        let audio_stream = Self::create_audio_stream(config.clone(), audio_stop_signal.clone())?;

        info!("Starting Deepgram streaming transcription");
        // Configure Deepgram for the processed audio format (mono 16kHz)
        let processed_sample_rate = 16000; // After resampling from 48kHz to 16kHz
        let processed_channels = 1;        // After converting from stereo to mono
        
        let mut results = dg_client
            .transcription()
            .stream_request()
            .keep_alive()
            .encoding(Encoding::Linear16)
            .sample_rate(processed_sample_rate)
            .channels(processed_channels)
            .interim_results(true)
            .stream(audio_stream)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start Deepgram stream: {}", e))?;

        info!("Deepgram streaming started, request ID: {}", results.request_id());
        
        let _ = app_handle.emit("transcription-status", json!({
            "status": "connected",
            "request_id": results.request_id()
        }));

        // Main processing loop
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = &mut shutdown_rx => {
                    info!("Received shutdown signal, stopping transcription");
                    break;
                }
                
                // Handle transcription results
                result = results.next() => {
                    match result {
                        Some(Ok(transcription_result)) => {
                            // Convert the result to JSON for easier parsing
                            let result_json = serde_json::to_value(transcription_result)
                                .unwrap_or_else(|_| serde_json::json!({ "error": "Failed to serialize result" }));
                            Self::handle_transcription_result(&app_handle, result_json).await?;
                        }
                        Some(Err(e)) => {
                            error!("Transcription error: {}", e);
                            let _ = app_handle.emit("transcription-error", json!({
                                "error": e.to_string()
                            }));
                            break;
                        }
                        None => {
                            info!("Transcription stream ended");
                            break;
                        }
                    }
                }
            }
        }

        info!("Transcription task completed");
        Ok(())
    }

    /// Create audio stream based on configuration
    fn create_audio_stream(config: AudioConfig, audio_stop_signal: Arc<AtomicBool>) -> Result<FuturesReceiver<Result<Bytes, RecvError>>> {
        info!("Creating audio stream: {:?}", config);

        let (sync_tx, sync_rx) = crossbeam::channel::bounded(500); // Larger buffer to prevent drops
        let (mut async_tx, async_rx) = mpsc::channel(200); // Larger async buffer

        // Spawn audio capture thread
        let config_clone = config.clone();
        let stop_signal_clone = audio_stop_signal.clone();
        thread::spawn(move || {
            if let Err(e) = Self::run_audio_capture_thread(config_clone, sync_tx, stop_signal_clone) {
                error!("Audio capture thread failed: {}", e);
            }
        });

        // Spawn bridge task to convert crossbeam channel to futures channel
        tokio::spawn(async move {
            loop {
                // Check stop signal periodically
                if audio_stop_signal.load(Ordering::Relaxed) {
                    info!("Audio stream bridge received stop signal");
                    break;
                }

                match sync_rx.recv_timeout(Duration::from_millis(50)) { // Reduced timeout for lower latency
                    Ok(data) => {
                        if async_tx.send(Ok(data)).await.is_err() {
                            break;
                        }
                    }
                    Err(crossbeam::channel::RecvTimeoutError::Timeout) => {
                        // Continue loop to check stop signal again
                        continue;
                    }
                    Err(crossbeam::channel::RecvTimeoutError::Disconnected) => {
                        info!("Audio capture channel closed");
                        break;
                    }
                }
            }
        });

        Ok(async_rx)
    }

    /// Audio capture thread function
    fn run_audio_capture_thread(
        config: AudioConfig,
        sync_tx: crossbeam::channel::Sender<Bytes>,
        stop_signal: Arc<AtomicBool>,
    ) -> Result<()> {
        let host = cpal::default_host();
        
        let device = if config.is_microphone {
            // For microphone, find actual microphone device, not stereo mix
            Self::find_microphone_device(&host)?
        } else {
            // For system audio, try to find a loopback-capable device
            Self::find_loopback_device(&host)?
        };

        let device_name = device.name().unwrap_or("Unknown".to_string());
        info!("Using audio device: {}", device_name);

        // Get device configuration
        let supported_config = if config.is_microphone {
            device.default_input_config()
                .map_err(|e| anyhow::anyhow!("Failed to get default input config: {}", e))?
        } else {
            // For loopback, we need input config from output device
            device.default_input_config()
                .map_err(|e| anyhow::anyhow!("Device doesn't support loopback: {}", e))?
        };

        info!("Device config: {} Hz, {} channels, {:?}", 
              supported_config.sample_rate().0, 
              supported_config.channels(), 
              supported_config.sample_format());

        // Build and start the audio stream
        let stream = Self::build_audio_stream(&device, &supported_config, sync_tx.clone(), stop_signal.clone())?;
        stream.play()
            .map_err(|e| anyhow::anyhow!("Failed to start audio stream: {}", e))?;

        info!("Audio capture started successfully");

        // Keep the stream alive until stop signal is received
        while !stop_signal.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }

        info!("Audio capture thread shutting down");
        drop(stream); // Explicitly drop the stream
        drop(sync_tx); // Close the channel
        
        Ok(())
    }

    /// Find actual microphone device (not stereo mix)
    fn find_microphone_device(host: &cpal::Host) -> Result<cpal::Device> {
        info!("Searching for microphone devices...");
        
        if let Ok(input_devices) = host.input_devices() {
            let mut potential_mics = Vec::new();
            
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    
                    // Skip stereo mix and similar loopback devices
                    if name_lower.contains("stereo mix") || 
                       name_lower.contains("what u hear") || 
                       name_lower.contains("wave out mix") {
                        info!("Skipping loopback device: {}", name);
                        continue;
                    }
                    
                    // Prioritize devices that contain "microphone" or "mic"
                    if name_lower.contains("microphone") || name_lower.contains("mic") {
                        info!("Found priority microphone device: {}", name);
                        return Ok(device);
                    }
                    
                    // Collect other potential input devices
                    if device.default_input_config().is_ok() {
                        info!("Found potential microphone device: {}", name);
                        potential_mics.push(device);
                    }
                }
            }
            
            // If we found potential microphones, use the first one
            if let Some(mic) = potential_mics.into_iter().next() {
                let name = mic.name().unwrap_or("Unknown".to_string());
                info!("Using first available input device as microphone: {}", name);
                return Ok(mic);
            }
        }
        
        // Fallback to default input device (but this might be stereo mix)
        if let Some(device) = host.default_input_device() {
            let name = device.name().unwrap_or("Unknown".to_string());
            warn!("Using default input device (might be stereo mix): {}", name);
            Ok(device)
        } else {
            Err(anyhow::anyhow!("No microphone devices found"))
        }
    }

    /// Find a device that supports loopback capture (for system audio)
    fn find_loopback_device(host: &cpal::Host) -> Result<cpal::Device> {
        // First try to find an explicit loopback device
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let name_lower = name.to_lowercase();
                    if name_lower.contains("stereo mix") || 
                       name_lower.contains("what u hear") || 
                       name_lower.contains("wave out mix") {
                        info!("Found dedicated loopback device: {}", name);
                        return Ok(device);
                    }
                }
            }
        }

        // Try output devices that support input (loopback mode)
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if device.supported_input_configs().is_ok() {
                    let name = device.name().unwrap_or("Unknown".to_string());
                    info!("Found output device with loopback capability: {}", name);
                    return Ok(device);
                }
            }
        }

        // Fallback to default input device
        host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No audio device suitable for system audio capture found"))
    }

    /// Build audio stream with proper sample format handling
    fn build_audio_stream(
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        sync_tx: crossbeam::channel::Sender<Bytes>,
        stop_signal: Arc<AtomicBool>,
    ) -> Result<Stream> {
        let stream_config = config.config();
        
        let stream = match config.sample_format() {
            SampleFormat::F32 => Self::build_stream::<f32>(device, &stream_config, sync_tx, stop_signal),
            SampleFormat::I16 => Self::build_stream::<i16>(device, &stream_config, sync_tx, stop_signal),
            SampleFormat::U16 => Self::build_stream::<u16>(device, &stream_config, sync_tx, stop_signal),
            sample_format => {
                return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format));
            }
        }?;

        Ok(stream)
    }

    /// Convert stereo to mono by averaging channels
    fn stereo_to_mono(stereo_data: &[i16]) -> Vec<i16> {
        let mut mono_data = Vec::with_capacity(stereo_data.len() / 2);
        
        for chunk in stereo_data.chunks_exact(2) {
            // Average left and right channels
            let left = chunk[0] as i32;
            let right = chunk[1] as i32;
            let mono = ((left + right) / 2) as i16;
            mono_data.push(mono);
        }
        
        mono_data
    }
    
    /// Better resampling from 48000 Hz to 16000 Hz with basic anti-aliasing
    fn resample_48k_to_16k(input_data: &[i16]) -> Vec<i16> {
        let mut output_data = Vec::with_capacity(input_data.len() / 3);
        
        // Better resampling with simple averaging for anti-aliasing
        // Instead of simple decimation, average 3 consecutive samples
        for chunk in input_data.chunks_exact(3) {
            // Average the 3 samples to reduce aliasing
            let avg = ((chunk[0] as i32 + chunk[1] as i32 + chunk[2] as i32) / 3) as i16;
            output_data.push(avg);
        }
        
        output_data
    }

    /// Generic stream builder for different sample formats
    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sync_tx: crossbeam::channel::Sender<Bytes>,
        stop_signal: Arc<AtomicBool>,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample + Send + 'static,
        i16: cpal::FromSample<T>,
    {
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;
        
        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // Check if we should stop before processing audio data
                    if stop_signal.load(Ordering::Relaxed) {
                        return; // Stop processing audio
                    }
                    
                    // Convert samples to i16 first
                    let mut i16_samples: Vec<i16> = Vec::with_capacity(data.len());
                    for sample in data {
                        let i16_sample: i16 = i16::from_sample(*sample);
                        i16_samples.push(i16_sample);
                    }
                    
                    // Convert stereo to mono if needed
                    let mono_samples = if channels == 2 {
                        Self::stereo_to_mono(&i16_samples)
                    } else {
                        i16_samples // Already mono
                    };
                    
                    // Resample if needed (48kHz -> 16kHz)
                    let final_samples = if sample_rate == 48000 {
                        Self::resample_48k_to_16k(&mono_samples)
                    } else {
                        mono_samples // Already at correct sample rate
                    };
                    
                    // Convert to bytes in little-endian format
                    let mut bytes = BytesMut::with_capacity(final_samples.len() * 2);
                    for sample in final_samples {
                        bytes.put_i16_le(sample);
                    }
                    
                    // Send to crossbeam channel (non-blocking)
                    if let Err(_) = sync_tx.try_send(bytes.freeze()) {
                        // Channel full or closed, skip this buffer
                        // Don't log warning every time when stopped to avoid spam
                        if !stop_signal.load(Ordering::Relaxed) {
                            warn!("Audio buffer dropped due to full channel");
                        }
                    }
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| anyhow::anyhow!("Failed to build input stream: {}", e))?;

        Ok(stream)
    }

    /// Handle transcription results and emit to frontend
    async fn handle_transcription_result(
        app_handle: &AppHandle,
        result: serde_json::Value,
    ) -> Result<()> {
        // Parse the Deepgram result format
        if let Some(channel) = result.get("channel") {
            if let Some(alternatives) = channel.get("alternatives") {
                if let Some(alternative) = alternatives.get(0) {
                    if let Some(transcript) = alternative.get("transcript").and_then(|t| t.as_str()) {
                        if !transcript.trim().is_empty() {
                            let is_final = result.get("is_final").and_then(|f| f.as_bool()).unwrap_or(false);
                            let confidence = alternative.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0);
                            
                            info!("Transcription: '{}' (final: {}, confidence: {:.2})", 
                                  transcript, is_final, confidence);
                            
                            let _ = app_handle.emit("transcription-result", json!({
                                "text": transcript,
                                "is_final": is_final,
                                "confidence": confidence,
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }));
                        } else if result.get("is_final").and_then(|f| f.as_bool()).unwrap_or(false) {
                            // Only log empty final results, skip interim empty results to reduce noise
                            let confidence = alternative.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0);
                            if confidence == 0.0 {
                                warn!("Received empty final transcription with 0 confidence - possible audio quality issue");
                            }
                        }
                    }
                } else {
                    // No alternatives found - could indicate audio processing issues
                    warn!("No transcription alternatives received - check audio input");
                }
            }
        } else {
            // This might be a status message or metadata, not a transcription result
            if result.get("type").and_then(|t| t.as_str()).unwrap_or("") != "Results" {
                info!("Received non-result message: {}", result.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
            }
        }

        Ok(())
    }
}

/// Global instance of the transcription service
static TRANSCRIPTION_SERVICE: std::sync::OnceLock<TokioMutex<Option<RealTimeTranscription>>> = std::sync::OnceLock::new();

fn get_transcription_service() -> &'static TokioMutex<Option<RealTimeTranscription>> {
    TRANSCRIPTION_SERVICE.get_or_init(|| TokioMutex::new(None))
}

/// Initialize the global transcription service
pub fn init_transcription_service(app_handle: AppHandle) {
    let service = get_transcription_service();
    // We need to initialize this in a blocking manner since we're in setup context
    let rt = tokio::runtime::Handle::try_current();
    match rt {
        Ok(handle) => {
            handle.spawn(async move {
                let mut guard = service.lock().await;
                *guard = Some(RealTimeTranscription::new(app_handle));
            });
        }
        Err(_) => {
            // If no tokio runtime exists, we'll initialize lazily on first use
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut guard = service.lock().await;
                    *guard = Some(RealTimeTranscription::new(app_handle));
                });
            });
        }
    }
}

/// Tauri command to start microphone transcription
#[tauri::command]
pub async fn start_microphone_transcription(app_handle: AppHandle) -> Result<String, String> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")
        .map_err(|_| "DEEPGRAM_API_KEY environment variable not set".to_string())?;

    let config = AudioConfig {
        sample_rate: 16000,  // Optimized for speech recognition
        channels: 1,         // Mono for better speech processing
        is_microphone: true,
    };

    let service = get_transcription_service();
    let mut service_guard = service.lock().await;
    if service_guard.is_none() {
        // Lazy initialization if not already done
        *service_guard = Some(RealTimeTranscription::new(app_handle.clone()));
    }
    
    if let Some(ref mut service) = *service_guard {
        service.start(config, api_key).await.map_err(|e| e.to_string())?;
    } else {
        return Err("Failed to initialize transcription service".to_string());
    }
    
    Ok("Microphone transcription started".to_string())
}

/// Tauri command to start system audio transcription
#[tauri::command]
pub async fn start_system_audio_transcription(app_handle: AppHandle) -> Result<String, String> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")
        .map_err(|_| "DEEPGRAM_API_KEY environment variable not set".to_string())?;

    let config = AudioConfig {
        sample_rate: 16000,  // Optimized for speech recognition
        channels: 1,         // Mono for better speech processing  
        is_microphone: false,
    };

    let service = get_transcription_service();
    let mut service_guard = service.lock().await;
    if service_guard.is_none() {
        // Lazy initialization if not already done
        *service_guard = Some(RealTimeTranscription::new(app_handle.clone()));
    }
    
    if let Some(ref mut service) = *service_guard {
        service.start(config, api_key).await.map_err(|e| e.to_string())?;
    } else {
        return Err("Failed to initialize transcription service".to_string());
    }
    
    Ok("System audio transcription started".to_string())
}

/// Tauri command to stop transcription
#[tauri::command]
pub async fn stop_transcription() -> Result<String, String> {
    let service = get_transcription_service();
    let mut service_guard = service.lock().await;
    if let Some(ref mut service) = *service_guard {
        service.stop().map_err(|e| e.to_string())?;
        Ok("Transcription stopped".to_string())
    } else {
        Err("Transcription service not initialized".to_string())
    }
}

/// Tauri command to check transcription status
#[tauri::command]
pub async fn get_transcription_status() -> Result<bool, String> {
    let service = get_transcription_service();
    let service_guard = service.lock().await;
    if let Some(ref service) = *service_guard {
        Ok(service.is_running())
    } else {
        // If not initialized, it's definitely not running
        Ok(false)
    }
}
