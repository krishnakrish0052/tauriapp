use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use parking_lot::Mutex;
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use crate::audio;

pub static DEEPGRAM_CLIENT: Mutex<Option<DeepgramClient>> = Mutex::new(None);

pub struct DeepgramClient {
    audio_sender: Option<mpsc::Sender<Vec<u8>>>,
    join_handle: Option<tokio::task::JoinHandle<()>>,
}

impl DeepgramClient {
    pub fn new() -> Self {
        Self {
            audio_sender: None,
            join_handle: None,
        }
    }

    pub async fn connect(&mut self, api_key: &str, app_handle: AppHandle) -> Result<()> {
        info!("Connecting to Deepgram WebSocket API...");

        // Create WebSocket URL with optimized parameters for speech recognition
        let ws_url = format!(
            "wss://api.deepgram.com/v1/listen?model=nova-2&language=en-US&smart_format=true&interim_results=true&punctuate=true&sample_rate=16000&channels=1&encoding=linear16&endpointing=100&vad_events=true"
        );

        let url = Url::parse(&ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        info!("Connected to Deepgram WebSocket");

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(1000);
        self.audio_sender = Some(audio_tx.clone());

        let app_handle_clone = app_handle.clone();
        let api_key_clone = api_key.to_string();

        // Set up audio callback to receive audio data from WASAPI
        let audio_tx_clone = audio_tx.clone();
        audio::set_audio_callback(move |audio_data| {
            if !audio_data.is_empty() {
                let _ = audio_tx_clone.try_send(audio_data);
            }
        });

        let join_handle = tokio::spawn(async move {
            // Send authentication message
            let auth_msg = json!({
                "type": "auth",
                "token": api_key_clone
            });
            if let Err(e) = ws_sender.send(Message::Text(auth_msg.to_string())).await {
                error!("Failed to send auth message: {}", e);
                return;
            }

            // Start tasks for sending audio and receiving transcriptions
            let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));
            let _app_handle_for_sender = app_handle_clone.clone();
            let ws_sender_clone = ws_sender.clone();

            // Task for sending audio data
            let sender_task = tokio::spawn(async move {
                while let Some(audio_data) = audio_rx.recv().await {
                    let mut sender = ws_sender_clone.lock().await;
                    if let Err(e) = sender.send(Message::Binary(audio_data)).await {
                        error!("Failed to send audio data to Deepgram: {}", e);
                        break;
                    }
                }
            });

            // Task for receiving transcription results
            let receiver_task = tokio::spawn(async move {
                while let Some(msg) = ws_receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            match serde_json::from_str::<serde_json::Value>(&text) {
                                Ok(json_msg) => {
                                    if let Some(channel) = json_msg.get("channel") {
                                        if let Some(alternatives) = channel.get("alternatives") {
                                            if let Some(alternative) = alternatives.get(0) {
                                                if let Some(transcript) = alternative.get("transcript").and_then(|t| t.as_str()) {
                                                    if !transcript.trim().is_empty() {
                                                        let is_final = json_msg.get("is_final").and_then(|f| f.as_bool()).unwrap_or(false);
                                                        let confidence = alternative.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0);
                                                        
                                                        info!("Transcription: {} (final: {}, confidence: {:.2})", transcript, is_final, confidence);
                                                        
                                                        let _ = app_handle_clone.emit(
                                                            "transcription-result",
                                                            json!({
                                                                "text": transcript,
                                                                "is_final": is_final,
                                                                "confidence": confidence
                                                            }),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse Deepgram response: {}", e);
                                }
                            }
                        }
                        Ok(Message::Binary(_)) => {
                            // Handle binary messages if needed
                        }
                        Ok(Message::Close(_)) => {
                            info!("Deepgram WebSocket connection closed");
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            let _ = app_handle_clone.emit(
                                "transcription-error",
                                json!({ "error": e.to_string() }),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            });

            // Wait for either task to complete
            tokio::select! {
                _ = sender_task => {
                    info!("Audio sender task completed");
                }
                _ = receiver_task => {
                    info!("Transcription receiver task completed");
                }
            }
        });

        self.join_handle = Some(join_handle);

        // Emit connection success
        let _ = app_handle.emit(
            "transcription-status",
            json!({ "status": "connected", "request_id": "deepgram-session" }),
        );

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            handle.abort();
            info!("Deepgram transcription task aborted");
        }
        self.audio_sender = None;
        info!("Disconnected from Deepgram");
    }
}

#[tauri::command]
pub async fn start_deepgram_transcription(app_handle: AppHandle) -> Result<String, String> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")
        .map_err(|_| "DEEPGRAM_API_KEY not set".to_string())?;

    let mut client = DeepgramClient::new();
    client.connect(&api_key, app_handle).await.map_err(|e| e.to_string())?;

    *DEEPGRAM_CLIENT.lock() = Some(client);

    Ok("Transcription started".to_string())
}

#[tauri::command]
pub async fn stop_deepgram_transcription() -> Result<String, String> {
    let client_to_disconnect = DEEPGRAM_CLIENT.lock().take();
    if let Some(mut client) = client_to_disconnect {
        client.disconnect().await;
    }
    Ok("Transcription stopped".to_string())
}