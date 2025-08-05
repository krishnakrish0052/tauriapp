#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Builder, AppHandle, Window, State};
use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use base64::Engine;
use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;

pub mod audio;
mod websocket;
mod deepgram;
mod openai;
mod wasapi_loopback;

use openai::{OpenAIClient, InterviewContext};

pub fn run() -> Result<()> {

    Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_audio_stream,
            stop_audio_stream,
            start_system_audio_capture,
            start_microphone_capture,
            get_audio_devices,
            check_audio_status,
            start_audio_with_config,
            test_audio_capture,
            send_manual_question,
            connect_to_session,
            close_application,
            minimize_window,
            toggle_always_on_top,
            start_deepgram_transcription,
            stop_deepgram_transcription,
            generate_ai_answer,
            analyze_screen_content,
            update_interview_context,
            save_audio_file
        ])
        .manage(AppState::new())
        .setup(|_app| {
            info!("MockMate application starting up...");
            
            // List available audio devices on startup
            audio::list_all_devices();
            
            // Initialize environment variables if needed
            if std::env::var("DEEPGRAM_API_KEY").is_err() {
                info!("DEEPGRAM_API_KEY not set in environment");
            }
            if std::env::var("OPENAI_API_KEY").is_err() {
                info!("OPENAI_API_KEY not set in environment");
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
    
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct QuestionPayload {
    session_id: String,
    question: String,
}

#[derive(Serialize, Deserialize)]
struct GenerateAnswerPayload {
    question: String,
    model: String,
    company: Option<String>,
    position: Option<String>,
    job_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AnalyzeScreenPayload {
    screen_content: String,
    model: String,
    company: Option<String>,
    position: Option<String>,
    job_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct InterviewContextPayload {
    company: Option<String>,
    position: Option<String>,
    job_description: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AudioConfigPayload {
    sample_rate: u32,
    channels: u16,
    buffer_size: u32,
}

// Global application state
#[derive(Default)]
struct AppState {
    openai_client: Arc<Mutex<Option<OpenAIClient>>>,
    interview_context: Arc<Mutex<InterviewContext>>,
}

impl AppState {
    fn new() -> Self {
        Self::default()
    }
    
    fn ensure_openai_client(&self) -> Result<(), String> {
        let mut client_guard = self.openai_client.lock();
        if client_guard.is_none() {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;
            *client_guard = Some(OpenAIClient::new(api_key));
        }
        Ok(())
    }
}

#[tauri::command]
fn start_audio_stream() -> Result<String, String> {
    info!("Starting audio stream...");
    match audio::capture_audio() {
        Ok(_) => Ok("Audio capture started successfully".to_string()),
        Err(e) => {
            error!("Failed to start audio capture: {}", e);
            Err(format!("Failed to start audio capture: {}", e))
        }
    }
}

#[tauri::command]
fn stop_audio_stream() -> Result<String, String> {
    info!("Stopping audio stream...");
    match audio::stop_capture() {
        Ok(_) => Ok("Audio capture stopped successfully".to_string()),
        Err(e) => {
            error!("Failed to stop audio capture: {}", e);
            Err(format!("Failed to stop audio capture: {}", e))
        }
    }
}

#[tauri::command]
fn send_manual_question(payload: QuestionPayload) {
    info!("Sending manual question: {}", payload.question);
    websocket::send_question(payload);
}

#[tauri::command]
fn connect_to_session(session_id: String) {
    info!("Connecting to session: {}", session_id);
    websocket::connect(session_id);
}

#[tauri::command]
fn close_application(app_handle: AppHandle) {
    info!("Closing application...");
    app_handle.exit(0);
}

#[tauri::command]
fn minimize_window(window: Window) {
    info!("Minimizing window...");
    if let Err(e) = window.minimize() {
        error!("Failed to minimize window: {}", e);
    }
}

#[tauri::command]
fn toggle_always_on_top(window: Window) -> Result<bool, String> {
    info!("Toggling always on top...");
    let is_always_on_top = window.is_always_on_top().map_err(|e| e.to_string())?;
    window.set_always_on_top(!is_always_on_top).map_err(|e| e.to_string())?;
    Ok(!is_always_on_top)
}

#[tauri::command]
async fn start_deepgram_transcription(app_handle: AppHandle) -> Result<String, String> {
    info!("Starting Deepgram transcription...");
    let api_key = std::env::var("DEEPGRAM_API_KEY")
        .map_err(|_| "DEEPGRAM_API_KEY environment variable not set".to_string())?;
    
    deepgram::start_transcription(api_key, app_handle)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok("Transcription started".to_string())
}

#[tauri::command]
async fn stop_deepgram_transcription() -> Result<String, String> {
    info!("Stopping Deepgram transcription...");
    deepgram::stop_transcription()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok("Transcription stopped".to_string())
}

#[tauri::command]
async fn generate_ai_answer(
    payload: GenerateAnswerPayload,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Generating AI answer for question: {}", payload.question);
    
    state.ensure_openai_client()?;
    
    let client = {
        let client_guard = state.openai_client.lock();
        client_guard.as_ref().unwrap().clone()
    };
    
    let mut context = {
        let context_guard = state.interview_context.lock();
        context_guard.clone()
    };
    
    // Update context with payload data if provided
    if let Some(company) = payload.company {
        context.company = Some(company);
    }
    if let Some(position) = payload.position {
        context.position = Some(position);
    }
    if let Some(job_description) = payload.job_description {
        context.job_description = Some(job_description);
    }
    
    let model = openai::OpenAIModel::from_string(&payload.model)
        .map_err(|e| format!("Invalid model: {}", e))?;
    
    let answer = client.generate_answer(&payload.question, &context, model)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(answer)
}

#[tauri::command]
async fn analyze_screen_content(
    payload: AnalyzeScreenPayload,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Analyzing screen content");
    
    state.ensure_openai_client()?;
    
    let client = {
        let client_guard = state.openai_client.lock();
        client_guard.as_ref().unwrap().clone()
    };
    
    let mut context = {
        let context_guard = state.interview_context.lock();
        context_guard.clone()
    };
    
    // Update context with payload data if provided
    if let Some(company) = payload.company {
        context.company = Some(company);
    }
    if let Some(position) = payload.position {
        context.position = Some(position);
    }
    if let Some(job_description) = payload.job_description {
        context.job_description = Some(job_description);
    }
    
    let model = openai::OpenAIModel::from_string(&payload.model)
        .map_err(|e| format!("Invalid model: {}", e))?;
    
    let analysis = client.analyze_screen_content(&payload.screen_content, &context, model)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(analysis)
}

#[tauri::command]
fn update_interview_context(
    payload: InterviewContextPayload,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Updating interview context");
    
    let mut context_guard = state.interview_context.lock();
    
    if let Some(company) = payload.company {
        context_guard.company = Some(company);
    }
    if let Some(position) = payload.position {
        context_guard.position = Some(position);
    }
    if let Some(job_description) = payload.job_description {
        context_guard.job_description = Some(job_description);
    }
    
    Ok("Interview context updated".to_string())
}

// New Audio Commands

#[tauri::command]
fn get_audio_devices() -> Result<Vec<String>, String> {
    info!("Getting audio devices...");
    match audio::list_input_devices() {
        Ok(devices) => {
            info!("Found {} audio devices", devices.len());
            Ok(devices)
        }
        Err(e) => {
            error!("Failed to get audio devices: {}", e);
            Err(format!("Failed to get audio devices: {}", e))
        }
    }
}

#[tauri::command]
fn check_audio_status() -> Result<serde_json::Value, String> {
    info!("Checking audio status...");
    let is_recording = audio::is_recording();
    let config = audio::get_audio_config();
    
    let status = serde_json::json!({
        "is_recording": is_recording,
        "config": {
            "sample_rate": config.sample_rate,
            "channels": config.channels,
            "buffer_size": config.buffer_size
        }
    });
    
    Ok(status)
}

#[tauri::command]
fn start_audio_with_config(config: AudioConfigPayload) -> Result<String, String> {
    info!("Starting audio with custom config: {} Hz, {} channels, {} buffer", 
          config.sample_rate, config.channels, config.buffer_size);
    
    let audio_config = audio::AudioConfig {
        sample_rate: config.sample_rate,
        channels: config.channels,
        buffer_size: config.buffer_size,
    };
    
    match audio::capture_audio_with_config(audio_config) {
        Ok(_) => Ok("Audio capture started with custom config".to_string()),
        Err(e) => {
            error!("Failed to start audio capture with config: {}", e);
            Err(format!("Failed to start audio capture: {}", e))
        }
    }
}

#[tauri::command]
async fn test_audio_capture(duration: u64) -> Result<String, String> {
    info!("Starting test audio capture for {} seconds", duration);
    
    match audio::test_capture_audio(duration) {
        Ok(_) => Ok(format!("Test audio capture completed for {} seconds", duration)),
        Err(e) => {
            error!("Test audio capture failed: {}", e);
            Err(format!("Test audio capture failed: {}", e))
        }
    }
}

#[tauri::command]
fn start_system_audio_capture() -> Result<String, String> {
    info!("Starting system audio capture...");
    match audio::start_system_audio_capture() {
        Ok(_) => Ok("System audio capture started successfully".to_string()),
        Err(e) => {
            error!("Failed to start system audio capture: {}", e);
            Err(format!("Failed to start system audio capture: {}", e))
        }
    }
}

#[tauri::command]
fn start_microphone_capture() -> Result<String, String> {
    info!("Starting microphone capture...");
    match audio::start_microphone_capture() {
        Ok(_) => Ok("Microphone capture started successfully".to_string()),
        Err(e) => {
            error!("Failed to start microphone capture: {}", e);
            Err(format!("Failed to start microphone capture: {}", e))
        }
    }
}

#[tauri::command]
fn save_audio_file() -> Result<String, String> {
    info!("Saving audio file with timestamp...");
    
    // Generate timestamp filename
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("audio_capture_{}.wav", timestamp);
    
    // Get captured audio samples
    let captured_samples = audio::get_captured_samples();
    let audio_config = audio::get_audio_config();
    
    if captured_samples.is_empty() {
        warn!("No audio samples captured");
        return Err("No audio samples available to save".to_string());
    }
    
    info!("Saving {} audio samples", captured_samples.len());
    
    let audio_data = audio::AudioData {
        samples: captured_samples,
        sample_rate: audio_config.sample_rate,
        channels: audio_config.channels,
        timestamp: std::time::SystemTime::now(),
    };
    
    // Convert to WAV format
    match audio::audio_data_to_base64_wav(&audio_data) {
        Ok(base64_wav) => {
            // Decode base64 and write to file
            match base64::prelude::BASE64_STANDARD.decode(base64_wav) {
                Ok(wav_data) => {
                    match std::fs::write(&filename, wav_data) {
                        Ok(_) => {
                            info!("Audio file saved successfully: {}", filename);
                            // Clean up the WASAPI loopback instance after successful save
                            audio::cleanup_audio_capture();
                            Ok(format!("Audio file saved: {} ({} samples)", filename, audio_data.samples.len()))
                        }
                        Err(e) => {
                            error!("Failed to write audio file: {}", e);
                            // Clean up even on error
                            audio::cleanup_audio_capture();
                            Err(format!("Failed to write audio file: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to decode base64 audio data: {}", e);
                    // Clean up on error
                    audio::cleanup_audio_capture();
                    Err(format!("Failed to decode audio data: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to convert audio to WAV format: {}", e);
            // Clean up on error
            audio::cleanup_audio_capture();
            Err(format!("Failed to convert audio to WAV: {}", e))
        }
    }
}
