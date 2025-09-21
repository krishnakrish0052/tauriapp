use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::StreamExt;
use anyhow::Result;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use log::{info, error};
use crate::QuestionPayload;

static SERVER_URL: &str = "ws://localhost:3000";

pub async fn setup_socket(handle: &AppHandle) -> Result<()> {
    let url = SERVER_URL;
    let (socket, response) = connect_async(url).await?;

    info!("WebSocket connected: {}", response.status());
    let (_write, mut read) = socket.split();

    // Handle incoming messages from WebSocket
    let handle_clone = handle.clone();
    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        info!("Received text message: {}", text);
                        // Parse and emit to frontend
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            let _ = handle_clone.emit("websocket-message", parsed.clone());

                            // Example of checking for `join-session` type
                            if parsed["type"] == "join-session" {
                                let session_id = parsed["sessionId"].as_str().unwrap_or("");
                                connect(session_id.to_string());
                            }
                        }
                    },
                    Message::Close(close) => {
                        if let Some(reason) = close {
                            info!("Socket closed with reason: {}", reason);
                        }
                        let _ = handle_clone.emit("websocket-closed", json!({}));
                    }
                    _ => {}
                },
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    let _ = handle_clone.emit("websocket-error", json!({"error": e.to_string()}));
                }
            }
        }
    });

    Ok(())
}

pub fn send_question(payload: QuestionPayload) {
    info!("Sending manual question: {} for session: {}", payload.question, payload.session_id);
    // TODO: Implement actual WebSocket sending
}

pub fn connect(session_id: String) {
    info!("Connecting to session: {}", session_id);
    // TODO: Implement session connection
}

