// Deepgram Nova-3 Streaming Transcription Integration
// Real-time streaming transcription with Pluely audio capture
// Using Nova-3 model for maximum accuracy and lowest latency

use anyhow::Result;
use log::{info, error};
use tauri::{AppHandle, Emitter, Listener};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Deepgram API configuration from environment
fn get_deepgram_api_key() -> String {
    // Try build-time embedded key first, then runtime env var
    option_env!("DEEPGRAM_API_KEY")
        .unwrap_or("")
        .to_string()
}

fn get_deepgram_model() -> String {
    option_env!("DEEPGRAM_MODEL")
        .unwrap_or("nova-3")
        .to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepgramTranscriptionResult {
    pub text: String,
    pub is_final: bool,
    pub confidence: f32,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
struct DeepgramResponse {
    #[serde(default)]
    channel: Option<DeepgramChannel>,
    #[serde(default)]
    is_final: bool,
}

#[derive(Debug, Deserialize)]
struct DeepgramChannel {
    #[serde(default)]
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Debug, Deserialize)]
struct DeepgramAlternative {
    transcript: String,
    confidence: f64,
}

/// Deepgram streaming transcription manager
pub struct DeepgramStreamer {
    app_handle: AppHandle,
    is_connected: Arc<std::sync::atomic::AtomicBool>,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl DeepgramStreamer {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            is_connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            stop_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start Deepgram streaming transcription
    pub async fn start_streaming(&mut self) -> Result<()> {
        info!("🎙️ Starting Deepgram Nova-3 streaming transcription...");

        let api_key = get_deepgram_api_key();
        if api_key.is_empty() {
            return Err(anyhow::anyhow!("DEEPGRAM_API_KEY not set"));
        }

        let model = get_deepgram_model();
        info!("📡 Using Deepgram model: {}", model);

        // Build Deepgram WebSocket URL with Nova-3 optimized parameters
        let ws_url = format!(
            "wss://api.deepgram.com/v1/listen?model={}&language=en-US&encoding=linear16&sample_rate=44100&channels=1&endpointing=50&interim_results=true&smart_format=true&punctuate=true&numerals=true",
            model
        );

        info!("🔗 Connecting to Deepgram: {}", ws_url);

        // Connect to Deepgram WebSocket with proper headers
        let request = tungstenite::http::Request::builder()
            .method("GET")
            .uri(&ws_url)
            .header("Host", "api.deepgram.com")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tungstenite::handshake::client::generate_key())
            .header("Authorization", format!("Token {}", api_key))
            .body(())
            .map_err(|e| anyhow::anyhow!("Failed to build request: {}", e))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Deepgram: {}", e))?;

        info!("✅ Connected to Deepgram WebSocket");
        self.is_connected.store(true, std::sync::atomic::Ordering::Relaxed);

        // Emit connection status
        let _ = self.app_handle.emit("deepgram-status", serde_json::json!({
            "status": "connected",
            "model": model,
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));

        let (write, mut read) = ws_stream.split();

        let app_clone = self.app_handle.clone();
        let stop_flag = self.stop_flag.clone();
        let is_connected = self.is_connected.clone();

        // Spawn task to handle incoming transcription results
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                // Check stop flag
                if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    info!("🛑 Deepgram reader stopping due to stop flag");
                    break;
                }

                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(response) = serde_json::from_str::<DeepgramResponse>(&text) {
                            if let Some(channel) = response.channel {
                                if let Some(alternative) = channel.alternatives.first() {
                                    let transcript = alternative.transcript.trim();
                                    
                                    if !transcript.is_empty() {
                                        let result = DeepgramTranscriptionResult {
                                            text: transcript.to_string(),
                                            is_final: response.is_final,
                                            confidence: alternative.confidence as f32,
                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                        };

                                        // Emit transcription result to frontend
                                        let _ = app_clone.emit("transcription-result", &result);

                                        if response.is_final {
                                            info!("📝 FINAL: \"{}\" ({:.1}%)", transcript, alternative.confidence * 100.0);
                                        } else {
                                            info!("⏳ INTERIM: \"{}\"", transcript);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("🔌 Deepgram connection closed");
                        is_connected.store(false, std::sync::atomic::Ordering::Relaxed);
                        let _ = app_clone.emit("deepgram-status", serde_json::json!({
                            "status": "disconnected",
                            "timestamp": chrono::Utc::now().timestamp_millis()
                        }));
                        break;
                    }
                    Err(e) => {
                        error!("❌ Deepgram WebSocket error: {}", e);
                        is_connected.store(false, std::sync::atomic::Ordering::Relaxed);
                        let _ = app_clone.emit("transcription-error", serde_json::json!({
                            "error": e.to_string()
                        }));
                        break;
                    }
                    _ => {}
                }
            }
            is_connected.store(false, std::sync::atomic::Ordering::Relaxed);
            info!("🛑 Deepgram reader task ended");
        });

        // Store the write half for sending audio
        let write_arc = Arc::new(Mutex::new(write));
        
        // Listen for audio chunks from Pluely capture
        let app_handle = self.app_handle.clone();
        let stop_flag_clone = self.stop_flag.clone();
        let is_connected_clone = self.is_connected.clone();
        
        tokio::spawn(async move {
            // Listen for system audio chunks
            let app_handle_clone = app_handle.clone();
            app_handle_clone.listen("audio-chunk", {
                let write_clone = write_arc.clone();
                let stop_flag = stop_flag_clone.clone();
                let is_connected = is_connected_clone.clone();
                move |event| {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    
                    // Check if WebSocket is still connected before sending
                    if !is_connected.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    
                    if let Ok(payload_str) = serde_json::from_str::<String>(event.payload()) {
                        // Payload is base64 encoded WAV
                        if let Ok(wav_data) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &payload_str) {
                            // Extract raw PCM data from WAV (skip header)
                            if wav_data.len() > 44 {
                                let pcm_data = wav_data[44..].to_vec();
                                
                                let write_clone2 = write_clone.clone();
                                tokio::spawn(async move {
                                    let mut write_guard = write_clone2.lock().await;
                                    if let Err(e) = write_guard.send(Message::Binary(pcm_data)).await {
                                        error!("Failed to send audio to Deepgram: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            });

            // Listen for microphone audio chunks
            let app_handle_clone2 = app_handle.clone();
            app_handle_clone2.listen("mic-audio-chunk", {
                let write_clone = write_arc.clone();
                let stop_flag = stop_flag_clone.clone();
                let is_connected = is_connected_clone.clone();
                move |event| {
                    if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    
                    // Check if WebSocket is still connected before sending
                    if !is_connected.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    
                    if let Ok(payload_str) = serde_json::from_str::<String>(event.payload()) {
                        // Payload is base64 encoded WAV
                        if let Ok(wav_data) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &payload_str) {
                            // Extract raw PCM data from WAV (skip header)
                            if wav_data.len() > 44 {
                                let pcm_data = wav_data[44..].to_vec();
                                
                                let write_clone2 = write_clone.clone();
                                tokio::spawn(async move {
                                    let mut write_guard = write_clone2.lock().await;
                                    if let Err(e) = write_guard.send(Message::Binary(pcm_data)).await {
                                        error!("Failed to send mic audio to Deepgram: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            });

            // Wait for stop signal
            loop {
                if stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    info!("🛑 Deepgram audio sender stopping");
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Send close frame to Deepgram
            let mut write_guard = write_arc.lock().await;
            let _ = write_guard.send(Message::Close(None)).await;
            info!("🛑 Deepgram audio sender task ended");
        });

        Ok(())
    }

    /// Stop Deepgram streaming
    pub async fn stop_streaming(&mut self) -> Result<()> {
        info!("🛑 Stopping Deepgram streaming...");
        
        self.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        self.is_connected.store(false, std::sync::atomic::Ordering::Relaxed);

        // Emit disconnection status
        let _ = self.app_handle.emit("deepgram-status", serde_json::json!({
            "status": "stopped",
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));

        // Wait a bit for tasks to stop
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        info!("✅ Deepgram streaming stopped");
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Global state management for Deepgram streaming
static DEEPGRAM_STATE: once_cell::sync::OnceCell<Arc<tokio::sync::Mutex<Option<DeepgramStreamer>>>> = once_cell::sync::OnceCell::new();

fn get_deepgram_streamer() -> Arc<tokio::sync::Mutex<Option<DeepgramStreamer>>> {
    DEEPGRAM_STATE.get_or_init(|| Arc::new(tokio::sync::Mutex::new(None))).clone()
}

/// Tauri command to start Deepgram streaming transcription
#[tauri::command]
pub async fn start_deepgram_streaming(app: AppHandle) -> Result<(), String> {
    info!("🚀 Starting Deepgram streaming transcription...");

    let streamer_arc = get_deepgram_streamer();
    
    // Stop existing streamer if running
    {
        let mut streamer_guard = streamer_arc.lock().await;
        if let Some(mut existing) = streamer_guard.take() {
            info!("Stopping existing Deepgram streamer...");
            let _ = existing.stop_streaming().await;
        }
    }
    
    // Create new streamer
    let mut streamer = DeepgramStreamer::new(app.clone());
    
    // Start streaming
    if let Err(e) = streamer.start_streaming().await {
        error!("Failed to start Deepgram streaming: {}", e);
        return Err(e.to_string());
    }
    
    // Store the streamer
    {
        let mut streamer_guard = streamer_arc.lock().await;
        *streamer_guard = Some(streamer);
    }
    
    info!("✅ Deepgram streaming started successfully");
    Ok(())
}

/// Tauri command to stop Deepgram streaming
#[tauri::command]
pub async fn stop_deepgram_streaming(_app: AppHandle) -> Result<(), String> {
    info!("🛑 Stopping Deepgram streaming...");

    let streamer_arc = get_deepgram_streamer();
    let mut streamer_guard = streamer_arc.lock().await;
    
    if let Some(mut streamer) = streamer_guard.take() {
        if let Err(e) = streamer.stop_streaming().await {
            error!("Failed to stop Deepgram streaming: {}", e);
            return Err(e.to_string());
        }
    }
    
    info!("✅ Deepgram streaming stopped");
    Ok(())
}

/// Check if Deepgram streaming is active
#[tauri::command]
pub async fn is_deepgram_streaming_active() -> Result<bool, String> {
    let streamer_arc = get_deepgram_streamer();
    let streamer_guard = streamer_arc.lock().await;
    
    if let Some(streamer) = streamer_guard.as_ref() {
        Ok(streamer.is_connected())
    } else {
        Ok(false)
    }
}
