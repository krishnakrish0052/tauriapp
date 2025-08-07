#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Builder, AppHandle, Window, State, Manager};
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
pub mod realtime_transcription;

use openai::{OpenAIClient, InterviewContext};

pub fn run() -> Result<()> {
    // Load environment variables from .env file
    if let Err(e) = dotenvy::dotenv() {
        warn!("Failed to load .env file: {}. Environment variables will be loaded from system.", e);
    } else {
        info!("Successfully loaded .env file");
    }

    Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_audio_stream,
            stop_audio_stream,
            start_system_audio_capture,
            start_microphone_capture,
            test_microphone_capture,
            get_audio_devices,
            check_audio_status,
            start_audio_with_config,
            test_audio_capture,
            send_manual_question,
            connect_to_session,
            close_application,
            minimize_window,
            toggle_always_on_top,
            create_ai_response_window,
            close_ai_response_window,
            resize_ai_response_window,
            show_ai_response_window,
            hide_ai_response_window,
            send_ai_response_data,
            // New real-time transcription commands
            realtime_transcription::start_microphone_transcription,
            realtime_transcription::start_system_audio_transcription,
            realtime_transcription::stop_transcription,
            realtime_transcription::get_transcription_status,
            // Keep old deepgram commands for backward compatibility during transition
            deepgram::start_deepgram_transcription,
            deepgram::stop_deepgram_transcription,
            generate_ai_answer,
            analyze_screen_content,
            update_interview_context,
            save_microphone_file,
            save_system_audio_file
        ])
        .manage(AppState::new())
        .setup(|app| {
            info!("MockMate application starting up...");
            
            // Initialize the real-time transcription service
            realtime_transcription::init_transcription_service(app.handle().clone());
            info!("✅ Real-time transcription service initialized");
            
            // Get the main window and set capture protection
            match app.get_webview_window("main") {
                Some(main_window) => {
                    info!("Main window found. Attempting to set capture protection.");
                    if let Err(e) = set_window_capture_protection(main_window, true) {
                        error!("Failed to set window capture protection on startup: {}", e);
                    }
                },
                None => {
                    error!("Main window not found on startup. Capture protection not applied.");
                }
            }
            
            // Create AI response window at startup (hidden by default)
            if let Err(e) = create_ai_response_window_at_startup(app.handle().clone()) {
                error!("Failed to create AI response window at startup: {}", e);
            } else {
                info!("✅ AI response window created at startup");
            }
            
            // List available audio devices on startup
            audio::list_all_devices();
            
            // Initialize environment variables if needed
            match std::env::var("DEEPGRAM_API_KEY") {
                Ok(_) => info!("✅ DEEPGRAM_API_KEY loaded successfully"),
                Err(_) => warn!("❌ DEEPGRAM_API_KEY not set in environment - transcription will not work")
            }
            match std::env::var("OPENAI_API_KEY") {
                Ok(_) => info!("✅ OPENAI_API_KEY loaded successfully"),
                Err(_) => warn!("❌ OPENAI_API_KEY not set in environment - AI answers will not work")
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
fn test_microphone_capture() -> Result<String, String> {
    info!("Testing microphone capture...");
    
    // List all audio devices first
    match audio::list_all_audio_devices() {
        Ok(devices) => {
            let input_devices: Vec<_> = devices.into_iter()
                .filter(|d| d.device_type == "input")
                .collect();
            
            info!("Found {} input devices:", input_devices.len());
            for device in &input_devices {
                info!("  - {} (default: {})", device.name, device.is_default);
            }
            
            if input_devices.is_empty() {
                return Err("No input devices found for microphone capture".to_string());
            }
            
            // Try to start microphone capture
            match audio::start_microphone_capture() {
                Ok(_) => {
                    info!("Microphone capture test started successfully");
                    Ok(format!("Microphone test started. Found {} input devices", input_devices.len()))
                }
                Err(e) => {
                    error!("Microphone capture test failed: {}", e);
                    Err(format!("Microphone test failed: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Failed to list audio devices: {}", e);
            Err(format!("Failed to list audio devices: {}", e))
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
fn save_microphone_file() -> Result<String, String> {
    info!("Saving microphone audio file...");
    save_audio_file_impl(true)
}

#[tauri::command]
fn save_system_audio_file() -> Result<String, String> {
    info!("Saving system audio file...");
    save_audio_file_impl(false)
}

fn save_audio_file_impl(is_mic: bool) -> Result<String, String> {
    info!("Saving audio file with timestamp...");
    
    // Generate timestamp filename
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let prefix = if is_mic { "mic" } else { "Sound" };
    let filename_prefix = if is_mic { "mic_capture" } else { "audio_capture" };
    let filename = format!("recordings/{}/{}_{}.wav", prefix, filename_prefix, timestamp);
    
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

#[tauri::command]
fn create_ai_response_window(app_handle: AppHandle) -> Result<String, String> {
    info!("Creating AI response window...");
    
    let main_window = match app_handle.get_webview_window("main") {
        Some(window) => window,
        None => {
            error!("Main window not found");
            return Err("Main window not found".to_string());
        }
    };
    
    // Get main window position and size
    let main_position = main_window.outer_position().map_err(|e| e.to_string())?;
    let main_size = main_window.outer_size().map_err(|e| e.to_string())?;
    
    // Calculate position for response window (below main window)
    let response_x = main_position.x;
    let response_y = main_position.y + main_size.height as i32 + 5; // 5px gap
    
    // Create response window configuration
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        tauri::WebviewUrl::App("ai-response.html".into())
    )
    .title("AI Response")
    .inner_size(1150.0, 150.0) // Start with minimal height
    .min_inner_size(1150.0, 100.0)
    .max_inner_size(1150.0, 500.0)
    .position(response_x as f64, response_y as f64)
    .resizable(false)
    .fullscreen(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .focused(true);
    
    match window_config.build() {
        Ok(window) => {
            info!("AI response window created successfully");
            
            // Set window capture protection for the response window too
            if let Err(e) = set_window_capture_protection(window, true) {
                error!("Failed to set window capture protection on AI response window: {}", e);
            }
            
            Ok("AI response window created".to_string())
        }
        Err(e) => {
            error!("Failed to create AI response window: {}", e);
            Err(format!("Failed to create AI response window: {}", e))
        }
    }
}

#[tauri::command]
fn close_ai_response_window(app_handle: AppHandle) -> Result<String, String> {
    info!("Closing AI response window...");
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        match window.close() {
            Ok(_) => {
                info!("AI response window closed successfully");
                Ok("AI response window closed".to_string())
            }
            Err(e) => {
                error!("Failed to close AI response window: {}", e);
                Err(format!("Failed to close AI response window: {}", e))
            }
        }
    } else {
        info!("AI response window not found - may already be closed");
        Ok("AI response window not found".to_string())
    }
}

#[tauri::command]
fn resize_ai_response_window(app_handle: AppHandle, height: u32) -> Result<String, String> {
    info!("Resizing AI response window to height: {}", height);
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        let clamped_height = height.max(100).min(500); // Clamp between 100 and 500
        
        match window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: 1150,
            height: clamped_height,
        })) {
            Ok(_) => {
                info!("AI response window resized successfully to height: {}", clamped_height);
                Ok(format!("Window resized to height: {}", clamped_height))
            }
            Err(e) => {
                error!("Failed to resize AI response window: {}", e);
                Err(format!("Failed to resize window: {}", e))
            }
        }
    } else {
        warn!("AI response window not found for resize");
        Err("AI response window not found".to_string())
    }
}

// Function to create AI response window at startup (hidden by default)
fn create_ai_response_window_at_startup(app_handle: AppHandle) -> Result<String, String> {
    info!("Creating AI response window at startup...");
    
    let main_window = match app_handle.get_webview_window("main") {
        Some(window) => window,
        None => {
            error!("Main window not found during startup");
            return Err("Main window not found".to_string());
        }
    };
    
    // Get main window position and size
    let main_position = main_window.outer_position().map_err(|e| e.to_string())?;
    let main_size = main_window.outer_size().map_err(|e| e.to_string())?;
    
    // Calculate position for response window (below main window)
    let response_x = main_position.x;
    let response_y = main_position.y + main_size.height as i32 + 5; // 5px gap
    
    // Create response window configuration (hidden by default)
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        tauri::WebviewUrl::App("ai-response.html".into())
    )
    .title("AI Response")
    .inner_size(1150.0, 150.0) // Start with minimal height
    .min_inner_size(1150.0, 100.0)
    .max_inner_size(1150.0, 500.0)
    .position(response_x as f64, response_y as f64)
    .resizable(false)
    .fullscreen(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false) // Start hidden
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .focused(false);
    
    match window_config.build() {
        Ok(window) => {
            info!("AI response window created at startup (hidden)");
            
            // Set window capture protection for the response window too
            if let Err(e) = set_window_capture_protection(window, true) {
                error!("Failed to set window capture protection on AI response window: {}", e);
            }
            
            Ok("AI response window created at startup".to_string())
        }
        Err(e) => {
            error!("Failed to create AI response window at startup: {}", e);
            Err(format!("Failed to create AI response window: {}", e))
        }
    }
}

#[tauri::command]
fn show_ai_response_window(app_handle: AppHandle) -> Result<String, String> {
    info!("Showing AI response window...");
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        match window.show() {
            Ok(_) => {
                info!("AI response window shown successfully");
                Ok("AI response window shown".to_string())
            }
            Err(e) => {
                error!("Failed to show AI response window: {}", e);
                Err(format!("Failed to show AI response window: {}", e))
            }
        }
    } else {
        warn!("AI response window not found - trying to create it");
        create_ai_response_window(app_handle)
    }
}

#[tauri::command]
fn hide_ai_response_window(app_handle: AppHandle) -> Result<String, String> {
    info!("Hiding AI response window...");
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        match window.hide() {
            Ok(_) => {
                info!("AI response window hidden successfully");
                Ok("AI response window hidden".to_string())
            }
            Err(e) => {
                error!("Failed to hide AI response window: {}", e);
                Err(format!("Failed to hide AI response window: {}", e))
            }
        }
    } else {
        info!("AI response window not found - already hidden or not created");
        Ok("AI response window not found".to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct AiResponseData {
    message_type: String, // "stream", "complete", "error"
    text: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn send_ai_response_data(app_handle: AppHandle, data: AiResponseData) -> Result<String, String> {
    info!("Sending AI response data: {:?}", data.message_type);
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        // First ensure the window is visible
        if let Err(e) = window.show() {
            error!("Failed to show AI response window: {}", e);
        }
        
        // Send data to the AI response window via JavaScript evaluation
        let js_code = match data.message_type.as_str() {
            "stream" => {
                let text = data.text.unwrap_or_default();
                let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                format!(r#"
                    if (window.updateContent) {{
                        window.updateContent('stream', {{ text: "{}" }});
                    }} else {{
                        console.log('updateContent function not found');
                    }}
                "#, escaped_text)
            }
            "complete" => {
                let text = data.text.unwrap_or_default();
                let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                format!(r#"
                    if (window.updateContent) {{
                        window.updateContent('complete', {{ text: "{}" }});
                    }}
                "#, escaped_text)
            }
            "error" => {
                let error_msg = data.error.unwrap_or_default();
                let escaped_error = error_msg.replace('\\', "\\\\").replace('"', "\\\"");
                format!(r#"
                    if (window.updateContent) {{
                        window.updateContent('error', {{ error: "{}" }});
                    }}
                "#, escaped_error)
            }
            _ => {
                return Err("Invalid message type".to_string());
            }
        };
        
        match window.eval(&js_code) {
            Ok(_) => {
                info!("AI response data sent successfully");
                Ok("Data sent to AI response window".to_string())
            }
            Err(e) => {
                error!("Failed to send data to AI response window: {}", e);
                Err(format!("Failed to send data: {}", e))
            }
        }
    } else {
        warn!("AI response window not found");
        Err("AI response window not found".to_string())
    }
}

#[tauri::command]
fn set_window_capture_protection(window: tauri::WebviewWindow, protect: bool) -> Result<(), String> {
    info!("Setting window capture protection to: {}", protect);
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::HWND;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
        let affinity = if protect { windows_sys::Win32::UI::WindowsAndMessaging::WDA_EXCLUDEFROMCAPTURE } else { windows_sys::Win32::UI::WindowsAndMessaging::WDA_NONE };
        unsafe {
            if windows_sys::Win32::UI::WindowsAndMessaging::SetWindowDisplayAffinity(hwnd, affinity) == 0 {
                let error_code = windows_sys::Win32::Foundation::GetLastError();
                error!("Failed to set window display affinity: {}", error_code);
                return Err(format!("Failed to set window display affinity: {}", error_code));
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        warn!("Window capture protection is only supported on Windows.");
    }
    Ok(())
}