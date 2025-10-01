// realtime_transcription.rs - DEPRECATED
// This module has been replaced with JavaScript SDK implementation in the frontend
// Functions are stubbed to maintain compatibility with existing code

use anyhow::Result;
use log::{info, warn};
use serde_json::json;
use tauri::AppHandle;

/// Audio capture configuration - DEPRECATED (stub)
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub is_microphone: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1,
            is_microphone: false,
        }
    }
}

/// DEPRECATED - Use JavaScript SDK instead
#[tauri::command]
pub async fn start_microphone_transcription(app: AppHandle) -> Result<(), String> {
    warn!("start_microphone_transcription is deprecated - use JavaScript SDK instead");
    Ok(())
}

/// DEPRECATED - Use JavaScript SDK instead  
#[tauri::command]
pub async fn start_system_audio_transcription(app: AppHandle) -> Result<(), String> {
    warn!("start_system_audio_transcription is deprecated - use JavaScript SDK instead");
    Ok(())
}

/// DEPRECATED - Use JavaScript SDK instead
#[tauri::command]
pub async fn stop_transcription() -> Result<String, String> {
    warn!("stop_transcription is deprecated - use JavaScript SDK instead");
    Ok("Transcription stopped (deprecated)".to_string())
}

/// DEPRECATED - Use JavaScript SDK instead
#[tauri::command]
pub async fn get_transcription_status() -> Result<serde_json::Value, String> {
    warn!("get_transcription_status is deprecated - use JavaScript SDK instead");
    Ok(json!({
        "status": "deprecated",
        "message": "Use JavaScript SDK instead"
    }))
}

/// DEPRECATED - Use JavaScript SDK instead
#[tauri::command]
pub async fn get_deepgram_config() -> Result<serde_json::Value, String> {
    warn!("get_deepgram_config is deprecated - use JavaScript SDK instead");
    Ok(json!({
        "model": "nova-3",
        "language": "en-US",
        "deprecated": true
    }))
}

/// DEPRECATED - Use JavaScript SDK instead
#[tauri::command]
pub async fn test_deepgram_connection() -> Result<String, String> {
    warn!("test_deepgram_connection is deprecated - use JavaScript SDK instead");
    Ok("Connection test deprecated - use JavaScript SDK instead".to_string())
}

// Internal implementation struct - kept for compatibility
pub struct RealTimeTranscription {
    config: AudioConfig,
}

impl RealTimeTranscription {
    pub fn new(_config: AudioConfig) -> Self {
        Self {
            config: AudioConfig::default(),
        }
    }

    pub fn is_active(&self) -> bool {
        false
    }

    pub async fn start_microphone(&mut self, _app_handle: AppHandle) -> Result<()> {
        warn!("RealTimeTranscription::start_microphone is deprecated");
        Ok(())
    }

    pub async fn start_system_audio(&mut self, _app_handle: AppHandle) -> Result<()> {
        warn!("RealTimeTranscription::start_system_audio is deprecated");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        warn!("RealTimeTranscription::stop is deprecated");
        Ok(())
    }
}