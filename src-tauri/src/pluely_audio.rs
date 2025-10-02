// Pluely-style system audio capture for MockMate
// Direct WASAPI integration with Voice Activity Detection
// Based on Pluely's efficient implementation

use anyhow::Result;
use futures_util::Stream;
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;
use wasapi::{get_default_device, Direction, SampleType, StreamMode, WaveFormat};
use std::time::Duration;
use log::{info, error, warn, debug};
use tauri::{AppHandle, Emitter};
use hound::{WavSpec, WavWriter};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use std::io::Cursor;

// Ultra-low latency constants optimized for real-time interview AI transcription
const HOP_SIZE: usize = 512;               // Analysis chunk size (~11.6ms at 44.1kHz) - HALF for ultra-low latency
const VAD_SENSITIVITY_RMS: f32 = 0.015;    // More sensitive RMS for faster speech detection
const SPEECH_PEAK_THRESHOLD: f32 = 0.04;   // Lower threshold for faster speech detection
const SILENCE_CHUNKS: usize = 10;          // ~0.12s silence to end speech - ULTRA FAST for interviews
const MIN_SPEECH_CHUNKS: usize = 4;        // ~0.05s min speech duration - ULTRA FAST
const PRE_SPEECH_CHUNKS: usize = 4;        // ~0.05s pre-speech buffer - ULTRA FAST

/// Pluely-style speaker input for system audio capture
pub struct PluelySpeakerInput {}

impl PluelySpeakerInput {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Start the audio stream - returns a Pluely-style speaker stream
    pub fn stream(self) -> PluelySpeakerStream {
        let sample_queue = Arc::new(Mutex::new(VecDeque::new()));
        let waker_state = Arc::new(Mutex::new(WakerState {
            waker: None,
            has_data: false,
            shutdown: false,
        }));
        let (init_tx, init_rx) = mpsc::channel();

        let queue_clone = sample_queue.clone();
        let waker_clone = waker_state.clone();

        let capture_thread = thread::spawn(move || {
            if let Err(e) = PluelySpeakerStream::capture_audio_loop(queue_clone, waker_clone, init_tx) {
                error!("Pluely Audio capture loop failed: {}", e);
            }
        });

        // Wait for initialization with timeout
        if let Ok(Err(e)) = init_rx.recv_timeout(Duration::from_secs(5)) {
            error!("Pluely Audio initialization failed: {}", e);
        }

        PluelySpeakerStream {
            sample_queue,
            waker_state,
            capture_thread: Some(capture_thread),
        }
    }
}

/// Waker state for efficient async polling
struct WakerState {
    waker: Option<Waker>,
    has_data: bool,
    shutdown: bool,
}

/// Pluely-style speaker stream - implements Stream trait for efficient audio processing
pub struct PluelySpeakerStream {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    waker_state: Arc<Mutex<WakerState>>,
    capture_thread: Option<thread::JoinHandle<()>>,
}

impl PluelySpeakerStream {
    /// Get sample rate (fixed at 44.1kHz like Pluely)
    pub fn sample_rate(&self) -> u32 {
        44100
    }

    /// Main audio capture loop using direct WASAPI - based on Pluely's implementation
    fn capture_audio_loop(
        sample_queue: Arc<Mutex<VecDeque<f32>>>,
        waker_state: Arc<Mutex<WakerState>>,
        init_tx: mpsc::Sender<Result<()>>,
    ) -> Result<()> {
        info!("🎵 Starting Pluely-style WASAPI capture loop...");

        let init_result = (|| -> Result<_> {
            // Get default render device for loopback capture
            let device = get_default_device(&Direction::Render)?;
            let mut audio_client = device.get_iaudioclient()?;

            // Use Pluely's exact format configuration
            let desired_format = WaveFormat::new(32, 32, &SampleType::Float, 44100, 1, None);

            let (_def_time, min_time) = audio_client.get_device_period()?;

            let mode = StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: min_time,
            };

            // Initialize in capture mode for loopback
            audio_client.initialize_client(&desired_format, &Direction::Capture, &mode)?;

            let h_event = audio_client.set_get_eventhandle()?;
            let render_client = audio_client.get_audiocaptureclient()?;

            audio_client.start_stream()?;
            info!("✅ Pluely-style WASAPI capture initialized successfully");

            Ok((h_event, render_client))
        })();

        match init_result {
            Ok((h_event, render_client)) => {
                let _ = init_tx.send(Ok(()));

                info!("🎵 Pluely audio capture loop running...");
                loop {
                    // Check shutdown signal
                    {
                        let state = waker_state.lock().unwrap();
                        if state.shutdown {
                            break;
                        }
                    }

                    // Wait for audio event (3 second timeout)
                    if h_event.wait_for_event(3000).is_err() {
                        debug!("Pluely audio event timeout, continuing...");
                        continue;
                    }

                    // Read audio data from device
                    let mut temp_queue = VecDeque::new();
                    if let Err(e) = render_client.read_from_device_to_deque(&mut temp_queue) {
                        warn!("Pluely failed to read audio data: {}", e);
                        continue;
                    }

                    if temp_queue.is_empty() {
                        continue;
                    }

                    // Convert raw bytes to f32 samples (Pluely's method)
                    let mut samples = Vec::new();
                    while temp_queue.len() >= 4 {
                        let bytes = [
                            temp_queue.pop_front().unwrap(),
                            temp_queue.pop_front().unwrap(),
                            temp_queue.pop_front().unwrap(),
                            temp_queue.pop_front().unwrap(),
                        ];
                        let sample = f32::from_le_bytes(bytes);
                        samples.push(sample);
                    }

                    if !samples.is_empty() {
                        // Add samples to queue with buffer management
                        {
                            let mut queue = sample_queue.lock().unwrap();
                            queue.extend(samples);

                            // Keep buffer size reasonable (like Pluely - 8192 samples)
                            let len = queue.len();
                            if len > 8192 {
                                queue.drain(0..(len - 8192));
                            }
                        }

                        // Wake up any waiting tasks
                        {
                            let mut state = waker_state.lock().unwrap();
                            if !state.has_data {
                                state.has_data = true;
                                if let Some(waker) = state.waker.take() {
                                    drop(state);
                                    waker.wake();
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let _ = init_tx.send(Err(e));
                return Ok(());
            }
        }

        info!("🛑 Pluely audio capture loop ended");
        Ok(())
    }
}

/// Clean shutdown for the speaker stream
impl Drop for PluelySpeakerStream {
    fn drop(&mut self) {
        {
            let mut state = self.waker_state.lock().unwrap();
            state.shutdown = true;
        }

        if let Some(thread) = self.capture_thread.take() {
            if let Err(e) = thread.join() {
                error!("Failed to join Pluely capture thread: {:?}", e);
            }
        }
    }
}

/// Stream implementation for async compatibility
impl Stream for PluelySpeakerStream {
    type Item = f32;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Check shutdown
        {
            let state = self.waker_state.lock().unwrap();
            if state.shutdown {
                return Poll::Ready(None);
            }
        }

        // Try to get a sample
        {
            let mut queue = self.sample_queue.lock().unwrap();
            if let Some(sample) = queue.pop_front() {
                return Poll::Ready(Some(sample));
            }
        }

        // No data available, register waker
        {
            let mut state = self.waker_state.lock().unwrap();
            if state.shutdown {
                return Poll::Ready(None);
            }
            state.has_data = false;
            state.waker = Some(cx.waker().clone());
            drop(state);
        }

        // Double check for data after registering waker
        {
            let mut queue = self.sample_queue.lock().unwrap();
            match queue.pop_front() {
                Some(sample) => Poll::Ready(Some(sample)),
                None => Poll::Pending,
            }
        }
    }
}

/// Pluely-style Voice Activity Detection and Audio Processing
pub struct PluelyAudioProcessor {
    app_handle: AppHandle,
    sample_buffer: VecDeque<f32>,
    pre_speech_buffer: VecDeque<f32>,
    speech_buffer: Vec<f32>,
    // New fields for real-time streaming
    streaming_buffer: Vec<f32>,
    streaming_enabled: bool,
    in_speech: bool,
    silence_chunks: usize,
    speech_chunks: usize,
    sample_rate: u32,
}

impl PluelyAudioProcessor {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            sample_buffer: VecDeque::new(),
            pre_speech_buffer: VecDeque::new(),
            speech_buffer: Vec::new(),
            // Initialize new streaming fields
            streaming_buffer: Vec::new(),
            streaming_enabled: true, // Enable by default for real-time transcription
            in_speech: false,
            silence_chunks: 0,
            speech_chunks: 0,
            sample_rate: 44100,
        }
    }

    /// Start Pluely-style system audio capture with VAD and Deepgram integration
pub async fn start_capture_with_transcription(&mut self) -> Result<()> {
        info!("🎵 Starting Pluely-style audio capture with transcription...");

        let input = PluelySpeakerInput::new()?;
        let mut stream = input.stream();
        let sr = stream.sample_rate();
        self.sample_rate = sr;

        // Emit debug: capture initialized
        let _ = self.app_handle.emit("pluely-audio-debug", serde_json::json!({
            "event": "capture-initialized",
            "sample_rate": sr,
            "hop_size": HOP_SIZE,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        }));

        let app_clone = self.app_handle.clone();
        let stop_flag = get_stop_flag();
        stop_flag.store(false, std::sync::atomic::Ordering::Relaxed);
        
        tokio::spawn(async move {
            let mut processor = PluelyAudioProcessor::new(app_clone.clone());
            processor.sample_rate = sr;

            use futures_util::StreamExt;
            while let Some(sample) = stream.next().await {
                // Check if we should stop
                if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    info!("🛑 System audio capture task stopping due to stop flag");
                    break;
                }
                processor.process_sample(sample).await;
            }
            info!("🛑 System audio capture task ended");
        });

        Ok(())
    }

    /// Process individual audio samples with Pluely's VAD algorithm
    async fn process_sample(&mut self, sample: f32) {
        self.sample_buffer.push_back(sample);
        
        // Real-time streaming: accumulate samples for streaming transcription
        if self.streaming_enabled {
            self.streaming_buffer.push(sample);
            
            // Send streaming chunks every 2048 samples (~46ms at 44.1kHz for low latency)
            const STREAMING_CHUNK_SIZE: usize = 2048;
            if self.streaming_buffer.len() >= STREAMING_CHUNK_SIZE {
                if let Ok(b64_chunk) = self.samples_to_wav_b64(&self.streaming_buffer) {
                    // Emit real-time audio chunk for Deepgram streaming
                    let _ = self.app_handle.emit("audio-chunk", b64_chunk);
                }
                self.streaming_buffer.clear();
            }
        }
        
        // Debug logging every 100,000 samples (about every 2.3 seconds at 44.1kHz)
        static mut SAMPLE_COUNTER: usize = 0;
        unsafe {
            SAMPLE_COUNTER += 1;
            if SAMPLE_COUNTER % 100000 == 0 {
                info!("🎵 Audio samples processed: {} (sample value: {:.6})", SAMPLE_COUNTER, sample);
            }
        }

        // Process in chunks of HOP_SIZE (Pluely's method)
        while self.sample_buffer.len() >= HOP_SIZE {
            let mut chunk = Vec::with_capacity(HOP_SIZE);
            for _ in 0..HOP_SIZE {
                if let Some(s) = self.sample_buffer.pop_front() {
                    chunk.push(s);
                }
            }

            let (rms, peak) = Self::process_chunk(&chunk);
            let is_speech = rms > VAD_SENSITIVITY_RMS || peak > SPEECH_PEAK_THRESHOLD;
            
            // Debug logging every 100 chunks (about every 2.3 seconds)
            static mut DEBUG_COUNTER: usize = 0;
            unsafe {
                DEBUG_COUNTER += 1;
                if DEBUG_COUNTER % 100 == 0 {
                info!("📊 Audio activity: RMS={:.6}, Peak={:.6}, Speech={}, Threshold RMS={:.6}, Threshold Peak={:.6}", 
                      rms, peak, is_speech, VAD_SENSITIVITY_RMS, SPEECH_PEAK_THRESHOLD);
                
                // Emit audio level update for UI visualization
                if true {
                    let _ = self.app_handle.emit("audio-level", serde_json::json!({
                        "rms": rms,
                        "peak": peak,
                        "speech": is_speech,
                        "timestamp": chrono::Utc::now().timestamp_millis()
                    }));
                }
                }
            }

            if is_speech {
                if !self.in_speech {
                    // Speech started
                    self.in_speech = true;
                    self.speech_chunks = 0;
                    self.silence_chunks = 0;
                    
                    // Add pre-speech buffer to speech buffer
                    self.speech_buffer.extend(self.pre_speech_buffer.drain(..));
                    
                    // Emit speech start event
                    let _ = self.app_handle.emit("speech-start", ()).map_err(|e| {
                        error!("Failed to emit speech-start: {}", e);
                    });
                    
                    info!("🎙️ Speech detected - starting capture");
                }
                
                self.speech_chunks += 1;
                self.speech_buffer.extend_from_slice(&chunk);
                
                // Safety cap: 30 seconds max
                let max_samples = self.sample_rate as usize * 30;
                if self.speech_buffer.len() > max_samples {
                    if let Ok(b64) = self.samples_to_wav_b64(&self.speech_buffer) {
                        let _ = self.app_handle.emit("speech-detected", b64);
                        info!("🎵 Emitted speech segment (safety cap): {} samples", self.speech_buffer.len());
                    }
                    self.speech_buffer.clear();
                    self.in_speech = false;
                }
            } else {
                if self.in_speech {
                    self.silence_chunks += 1;
                    self.speech_buffer.extend_from_slice(&chunk);
                    
                    // Debug log silence accumulation
                    if self.silence_chunks % 10 == 0 {
                        info!("🔇 Silence detected: {} chunks (need {} to complete)", self.silence_chunks, SILENCE_CHUNKS);
                    }
                    
                    // Check if we have enough silence to end speech
                    if self.silence_chunks >= SILENCE_CHUNKS {
                        if self.speech_chunks >= MIN_SPEECH_CHUNKS && !self.speech_buffer.is_empty() {
                            // Trim trailing silence
                            let trim = (SILENCE_CHUNKS / 2) * HOP_SIZE;
                            if self.speech_buffer.len() > trim {
                                self.speech_buffer.truncate(self.speech_buffer.len() - trim);
                            }
                            
                            // Convert to WAV and emit
                            if let Ok(b64) = self.samples_to_wav_b64(&self.speech_buffer) {
                                let _ = self.app_handle.emit("speech-detected", b64);
                                info!("🎵 Emitted speech segment: {} samples ({:.2}s)", 
                                      self.speech_buffer.len(), 
                                      self.speech_buffer.len() as f32 / self.sample_rate as f32);
                            }
                        }
                        
                        // Reset for next speech segment
                        self.speech_buffer.clear();
                        self.in_speech = false;
                        self.silence_chunks = 0;
                        self.speech_chunks = 0;
                    }
                } else {
                    // Not in speech: maintain pre-speech buffer
                    self.pre_speech_buffer.extend(chunk.into_iter());
                    while self.pre_speech_buffer.len() > PRE_SPEECH_CHUNKS * HOP_SIZE {
                        self.pre_speech_buffer.pop_front();
                    }
                }
            }
        }
    }

    /// Process audio chunk for VAD (RMS and peak calculation) - Pluely's method
    fn process_chunk(chunk: &[f32]) -> (f32, f32) {
        let mut sumsq = 0.0f32;
        let mut peak = 0.0f32;
        
        for &sample in chunk {
            let abs_sample = sample.abs();
            peak = peak.max(abs_sample);
            sumsq += sample * sample;
        }
        
        let rms = (sumsq / chunk.len() as f32).sqrt();
        (rms, peak)
    }

    /// Convert samples to WAV base64 for Deepgram - Pluely's method
    fn samples_to_wav_b64(&self, samples: &[f32]) -> Result<String, String> {
        let mut cursor = Cursor::new(Vec::new());
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;

        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16).map_err(|e| e.to_string())?;
        }
        
        writer.finalize().map_err(|e| e.to_string())?;
        Ok(B64.encode(cursor.into_inner()))
    }

}

/// Global state management for audio capture
static AUDIO_STATE: once_cell::sync::OnceCell<Arc<Mutex<Option<PluelyAudioProcessor>>>> = once_cell::sync::OnceCell::new();

// Add a flag to signal the capture task to stop
static AUDIO_STOP_FLAG: once_cell::sync::OnceCell<Arc<std::sync::atomic::AtomicBool>> = once_cell::sync::OnceCell::new();

fn get_audio_processor() -> Arc<Mutex<Option<PluelyAudioProcessor>>> {
    AUDIO_STATE.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

fn get_stop_flag() -> Arc<std::sync::atomic::AtomicBool> {
    AUDIO_STOP_FLAG.get_or_init(|| Arc::new(std::sync::atomic::AtomicBool::new(false))).clone()
}

/// Tauri command to start Pluely-style system audio capture
#[tauri::command]
pub async fn start_pluely_system_audio_capture(app: AppHandle) -> Result<(), String> {
    info!("🚀 Starting Pluely-style system audio capture...");

    // Emit debug event to UI
    let _ = app.emit("pluely-audio-debug", serde_json::json!({
        "event": "start-requested",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }));
    
    let processor_arc = get_audio_processor();
    
    // Stop existing processor if running (scope the MutexGuard)
    {
        let mut processor_guard = processor_arc.lock().unwrap();
        if processor_guard.is_some() {
            info!("Stopping existing audio processor...");
            *processor_guard = None;
        }
    }
    
    // Create new processor
    let mut processor = PluelyAudioProcessor::new(app.clone());
    
    // Start capture with transcription
    if let Err(e) = processor.start_capture_with_transcription().await.map_err(|e| e.to_string()) {
        let _ = app.emit("pluely-audio-debug", serde_json::json!({
            "event": "start-error",
            "error": e,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        }));
        return Err(e);
    }
    
    // Store the processor (scope the MutexGuard)
    {
        let mut processor_guard = processor_arc.lock().unwrap();
        *processor_guard = Some(processor);
    }

    // Emit started event
    let _ = app.emit("pluely-audio-debug", serde_json::json!({
        "event": "started",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }));
    
    info!("✅ Pluely-style system audio capture started successfully");
    Ok(())
}

/// Tauri command to stop system audio capture
#[tauri::command]
pub async fn stop_pluely_system_audio_capture(app: AppHandle) -> Result<(), String> {
    info!("🛑 Stopping Pluely-style system audio capture...");

    // Emit debug event to UI
    let _ = app.emit("pluely-audio-debug", serde_json::json!({
        "event": "stop-requested",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }));
    
    // Set the stop flag to signal the capture task to stop
    let stop_flag = get_stop_flag();
    stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    info!("🛑 Stop flag set to true");
    
    // Wait a bit for the task to stop
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let processor_arc = get_audio_processor();
    let mut processor_guard = processor_arc.lock().unwrap();
    
    *processor_guard = None;

    // Emit stopped event
    let _ = app.emit("pluely-audio-debug", serde_json::json!({
        "event": "stopped",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }));
    
    info!("✅ Pluely-style system audio capture stopped");
    Ok(())
}

/// Check if system audio capture is active
#[tauri::command]
pub async fn is_pluely_audio_active() -> Result<bool, String> {
    let processor_arc = get_audio_processor();
    let processor_guard = processor_arc.lock().unwrap();
    Ok(processor_guard.is_some())
}

/// Test command to verify Pluely system audio capture works
#[tauri::command]
pub async fn test_pluely_system_audio_capture(app: AppHandle, duration_seconds: u64) -> Result<String, String> {
    info!("🧑‍🔬 Testing Pluely system audio capture for {} seconds", duration_seconds);
    
    // Start capture
    if let Err(e) = start_pluely_system_audio_capture(app.clone()).await {
        return Err(format!("Failed to start capture: {}", e));
    }
    
    info!("Capture started, waiting {} seconds...", duration_seconds);
    tokio::time::sleep(tokio::time::Duration::from_secs(duration_seconds)).await;
    
    // Stop capture
    if let Err(e) = stop_pluely_system_audio_capture(app.clone()).await {
        return Err(format!("Failed to stop capture: {}", e));
    }
    
    Ok(format!("Pluely system audio capture test completed successfully for {} seconds", duration_seconds))
}
