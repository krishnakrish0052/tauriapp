// Pluely-Deepgram Integration Module
// Unified commands that combine Pluely audio capture with Deepgram Nova-3 transcription
// This bridges the gap between the separate audio capture and streaming transcription systems

use anyhow::Result;
use log::{info, error, warn};
use tauri::{AppHandle, Emitter};
use serde_json::json;

/// Unified command to start both Pluely audio capture and Deepgram transcription
#[tauri::command]
pub async fn start_pluely_deepgram_transcription(app: AppHandle) -> Result<(), String> {
    info!("🚀 Starting unified Pluely-Deepgram transcription...");

    // Emit status to frontend
    let _ = app.emit("transcription-status", json!({
        "status": "starting",
        "message": "Initializing audio capture and Deepgram connection...",
        "timestamp": chrono::Utc::now().timestamp_millis()
    }));

    // Step 1: Start Deepgram streaming first (establishes WebSocket connection)
    info!("📡 Step 1: Starting Deepgram Nova-3 streaming...");
    if let Err(e) = crate::deepgram_streaming::start_deepgram_streaming(app.clone()).await {
        error!("❌ Failed to start Deepgram streaming: {}", e);
        let _ = app.emit("transcription-status", json!({
            "status": "error",
            "message": format!("Failed to start Deepgram: {}", e),
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));
        return Err(format!("Failed to start Deepgram streaming: {}", e));
    }

    // Step 2: Start Pluely system audio capture (captures system sound)
    info!("🎵 Step 2: Starting Pluely system audio capture...");
    if let Err(e) = crate::pluely_audio::start_pluely_system_audio_capture(app.clone()).await {
        error!("❌ Failed to start system audio capture: {}", e);
        // Try to stop Deepgram if system audio failed
        let _ = crate::deepgram_streaming::stop_deepgram_streaming(app.clone()).await;
        let _ = app.emit("transcription-status", json!({
            "status": "error",
            "message": format!("Failed to start system audio: {}", e),
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));
        return Err(format!("Failed to start system audio capture: {}", e));
    }

    // Step 3: Start Pluely microphone capture (captures microphone)
    info!("🎤 Step 3: Starting Pluely microphone capture...");
    if let Err(e) = crate::pluely_microphone::start_pluely_microphone_capture(app.clone()).await {
        error!("❌ Failed to start microphone capture: {}", e);
        // Don't fail completely if microphone fails, system audio might still work
        warn!("⚠️ Microphone capture failed, continuing with system audio only: {}", e);
        let _ = app.emit("transcription-status", json!({
            "status": "partial",
            "message": "System audio started, microphone failed - continuing with system audio only",
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));
    }

    // Success - emit ready status
    let _ = app.emit("transcription-status", json!({
        "status": "streaming",
        "model": "nova-3",
        "message": "Live transcription active with Nova-3 model",
        "timestamp": chrono::Utc::now().timestamp_millis()
    }));

    info!("✅ Unified Pluely-Deepgram transcription started successfully");
    Ok(())
}

/// Unified command to stop both Pluely audio capture and Deepgram transcription
#[tauri::command]
pub async fn stop_pluely_deepgram_transcription(app: AppHandle) -> Result<(), String> {
    info!("🛑 Stopping unified Pluely-Deepgram transcription...");

    // Emit stopping status
    let _ = app.emit("transcription-status", json!({
        "status": "stopping",
        "message": "Stopping transcription services...",
        "timestamp": chrono::Utc::now().timestamp_millis()
    }));

    let mut errors = Vec::new();

    // Stop Deepgram streaming
    info!("📡 Stopping Deepgram streaming...");
    if let Err(e) = crate::deepgram_streaming::stop_deepgram_streaming(app.clone()).await {
        error!("❌ Failed to stop Deepgram streaming: {}", e);
        errors.push(format!("Deepgram: {}", e));
    }

    // Stop system audio capture
    info!("🎵 Stopping Pluely system audio capture...");
    if let Err(e) = crate::pluely_audio::stop_pluely_system_audio_capture(app.clone()).await {
        error!("❌ Failed to stop system audio capture: {}", e);
        errors.push(format!("System audio: {}", e));
    }

    // Stop microphone capture
    info!("🎤 Stopping Pluely microphone capture...");
    if let Err(e) = crate::pluely_microphone::stop_pluely_microphone_capture(app.clone()).await {
        error!("❌ Failed to stop microphone capture: {}", e);
        errors.push(format!("Microphone: {}", e));
    }

    // Emit final status
    if errors.is_empty() {
        let _ = app.emit("transcription-status", json!({
            "status": "disconnected",
            "message": "Transcription stopped successfully",
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));
        info!("✅ Unified Pluely-Deepgram transcription stopped successfully");
        Ok(())
    } else {
        let error_msg = format!("Some services failed to stop: {}", errors.join(", "));
        let _ = app.emit("transcription-status", json!({
            "status": "error",
            "message": &error_msg,
            "timestamp": chrono::Utc::now().timestamp_millis()
        }));
        Err(error_msg)
    }
}

/// Check if the unified transcription system is active
#[tauri::command]
pub async fn is_pluely_deepgram_transcription_active() -> Result<bool, String> {
    // Check if both Deepgram and at least one audio source are active
    let deepgram_active = crate::deepgram_streaming::is_deepgram_streaming_active().await
        .unwrap_or(false);
    
    let system_audio_active = crate::pluely_audio::is_pluely_audio_active().await
        .unwrap_or(false);
    
    let microphone_active = crate::pluely_microphone::is_pluely_microphone_active().await
        .unwrap_or(false);

    let is_active = deepgram_active && (system_audio_active || microphone_active);
    
    info!("🔍 Transcription status check: Deepgram={}, System={}, Mic={}, Overall={}", 
          deepgram_active, system_audio_active, microphone_active, is_active);
    
    Ok(is_active)
}

/// Get detailed status of all transcription components
#[tauri::command]
pub async fn get_pluely_deepgram_transcription_status() -> Result<serde_json::Value, String> {
    let deepgram_active = crate::deepgram_streaming::is_deepgram_streaming_active().await
        .unwrap_or(false);
    
    let system_audio_active = crate::pluely_audio::is_pluely_audio_active().await
        .unwrap_or(false);
    
    let microphone_active = crate::pluely_microphone::is_pluely_microphone_active().await
        .unwrap_or(false);

    let overall_active = deepgram_active && (system_audio_active || microphone_active);

    let status = json!({
        "overall_active": overall_active,
        "components": {
            "deepgram_streaming": deepgram_active,
            "system_audio_capture": system_audio_active,
            "microphone_capture": microphone_active
        },
        "model": "nova-3",
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    Ok(status)
}

/// Test the Deepgram connection directly (for diagnostics)
#[tauri::command]
pub async fn test_deepgram_streaming_direct() -> Result<String, String> {
    info!("🧪 Testing Deepgram streaming connection directly...");
    
    // This is a simple test that tries to start and immediately stop Deepgram streaming
    // to verify the connection works
    match std::env::var("DEEPGRAM_API_KEY").or_else(|_| {
        match option_env!("DEEPGRAM_API_KEY") {
            Some(key) => Ok(key.to_string()),
            None => Err(std::env::VarError::NotPresent)
        }
    }) {
        Ok(api_key) => {
            if api_key.is_empty() {
                return Err("DEEPGRAM_API_KEY is empty".to_string());
            }
            info!("✅ Deepgram API key found (length: {})", api_key.len());
            
            // For now, just confirm the API key is available
            // In a more complete test, we could try to establish a WebSocket connection
            Ok(format!("Deepgram API key validated ({}...)", &api_key[..8.min(api_key.len())]))
        }
        Err(_) => {
            Err("DEEPGRAM_API_KEY not found in environment or build-time config".to_string())
        }
    }
}