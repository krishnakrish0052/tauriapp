use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use log::{info, error, debug};
use anyhow::Result;
use tokio::sync::mpsc;
use std::sync::Arc;
use parking_lot::Mutex;
use once_cell::sync::Lazy;

static DEEPGRAM_CLIENT: Lazy<Arc<Mutex<Option<DeepgramClient>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct DeepgramClient {
    audio_sender: Option<mpsc::Sender<Vec<u8>>>,
    is_connected: bool,
}

impl DeepgramClient {
    pub fn new() -> Self {
        Self {
            audio_sender: None,
            is_connected: false,
        }
    }

    pub async fn connect(&mut self, api_key: &str, app_handle: AppHandle) -> Result<()> {
        let deepgram_url = format!(
            "wss://api.deepgram.com/v1/listen?model=nova-2&language=en-US&smart_format=true&interim_results=true&endpointing=300"
        );

        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&deepgram_url)
            .header("Authorization", format!("Token {}", api_key))
            .body(())?;

        let (ws_stream, _) = connect_async(request).await?;
        let (mut write, mut read) = ws_stream.split();

        // Create channel for audio data
        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(1000);
        self.audio_sender = Some(audio_tx);
        self.is_connected = true;

        info!("Connected to Deepgram");

        // Handle incoming transcription results
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        debug!("Deepgram response: {}", text);
                        
                        if let Ok(data) = serde_json::from_str::<Value>(&text) {
                            if let Some(channel) = data.get("channel") {
                                if let Some(alternatives) = channel.get("alternatives") {
                                    if let Some(alternative) = alternatives.get(0) {
                                        if let Some(transcript) = alternative.get("transcript") {
                                            let transcript_text = transcript.as_str().unwrap_or("").trim();
                                            if !transcript_text.is_empty() {
                                                let is_final = data.get("is_final").and_then(|v| v.as_bool()).unwrap_or(false);
                                                
                                                let _ = app_handle_clone.emit("transcription", json!({
                                                    "text": transcript_text,
                                                    "is_final": is_final
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Deepgram connection closed");
                        let _ = app_handle_clone.emit("transcription-closed", json!({}));
                        break;
                    }
                    Err(e) => {
                        error!("Deepgram error: {}", e);
                        let _ = app_handle_clone.emit("transcription-error", json!({"error": e.to_string()}));
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Handle outgoing audio data
        tokio::spawn(async move {
            while let Some(audio_data) = audio_rx.recv().await {
                if let Err(e) = write.send(Message::Binary(audio_data)).await {
                    error!("Failed to send audio to Deepgram: {}", e);
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn send_audio(&self, audio_data: Vec<u8>) -> Result<()> {
        if let Some(sender) = &self.audio_sender {
            sender.send(audio_data).await?;
        }
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub async fn disconnect(&mut self) {
        self.is_connected = false;
        self.audio_sender = None;
        info!("Disconnected from Deepgram");
    }
}

pub async fn start_transcription(api_key: String, app_handle: AppHandle) -> Result<()> {
    let mut client = DeepgramClient::new();
    client.connect(&api_key, app_handle).await?;
    
    *DEEPGRAM_CLIENT.lock() = Some(client);
    Ok(())
}

pub async fn stop_transcription() -> Result<()> {
    let client = DEEPGRAM_CLIENT.lock().take();
    if let Some(mut client) = client {
        client.disconnect().await;
    }
    Ok(())
}

pub async fn send_audio_data(audio_data: Vec<u8>) -> Result<()> {
    // First check if connected without holding the lock across await
    let client_connected = {
        let guard = DEEPGRAM_CLIENT.lock();
        guard.as_ref().map(|c| c.is_connected()).unwrap_or(false)
    };
    
    if client_connected {
        // Clone the sender outside the lock to avoid holding it across await
        let sender = {
            let guard = DEEPGRAM_CLIENT.lock();
            guard.as_ref().and_then(|c| c.audio_sender.clone())
        };
        
        if let Some(sender) = sender {
            sender.send(audio_data).await?;
        }
    }
    Ok(())
}
