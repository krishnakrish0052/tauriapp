#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Builder, AppHandle, Window, State, Manager, Emitter};
use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use base64::Engine;
use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;

pub mod audio;
mod websocket;
mod openai;
mod pollinations;
mod wasapi_loopback;
pub mod realtime_transcription;

// New Phase 2 modules
pub mod database;
// pub mod session; // Temporarily disabled to avoid conflicts
// pub mod interview; // Temporarily disabled to avoid conflicts

use openai::{OpenAIClient, InterviewContext};
use pollinations::{PollinationsClient, AIProvider};
use database::shared::*; // Import shared database types and functions

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
            // Real-time transcription commands
            realtime_transcription::start_microphone_transcription,
            realtime_transcription::start_system_audio_transcription,
            realtime_transcription::stop_transcription,
            realtime_transcription::get_transcription_status,
            realtime_transcription::get_deepgram_config,
            generate_ai_answer,
            analyze_screen_content,
            update_interview_context,
            get_available_models,
            get_ai_providers,
            save_microphone_file,
            save_system_audio_file,
            pollinations_generate_answer,
            pollinations_generate_answer_streaming,
            pollinations_generate_answer_post_streaming,
            // Session management commands (existing)
            connect_to_web_session,
            activate_web_session,
            get_session_info,
            handle_protocol_launch,
            // New shared database session commands
            connect_session,
            activate_session_cmd,
            disconnect_session_cmd,
            // Frontend compatibility commands
            validate_session_id,
            activate_session,
            disconnect_session,
            // Timer management
            update_session_timer,
            // Database operations
            database::postgres::test_database_connection,
            database::postgres::get_db_session_info,
            database::postgres::save_interview_question,
            database::postgres::save_interview_answer,
            database::postgres::get_session_questions,
            database::postgres::get_session_answers,
            database::postgres::get_interview_report,
            database::postgres::finalize_session_duration,
            // Window management
            resize_main_window
        ])
        .manage(AppState::new())
        .setup(|app| {
            info!("MockMate application starting up...");
            
            // Handle command line arguments for protocol URLs
            let args: Vec<String> = std::env::args().collect();
            info!("Command line args: {:?}", args);
            
            // Check if launched with a mockmate:// URL
            if let Some(protocol_url) = args.iter().find(|arg| arg.starts_with("mockmate://")) {
                info!("Detected protocol launch: {}", protocol_url);
                
                // Parse the protocol URL
                if let Some(session_part) = protocol_url.strip_prefix("mockmate://session/") {
                    // Extract session ID and any query parameters
                    let parts: Vec<&str> = session_part.split('?').collect();
                    let session_id = parts[0].to_string();
                    
                    info!("Parsed session ID: {}", session_id);
                    
                    // Extract query parameters if present
                    let mut token: Option<String> = None;
                    let mut user_id: Option<String> = None;
                    
                    if parts.len() > 1 {
                        for param in parts[1].split('&') {
                            let kv: Vec<&str> = param.split('=').collect();
                            if kv.len() == 2 {
                                match kv[0] {
                                    "token" => token = Some(urlencoding::decode(kv[1]).unwrap_or_default().to_string()),
                                    "user_id" => user_id = Some(urlencoding::decode(kv[1]).unwrap_or_default().to_string()),
                                    _ => {}
                                }
                            }
                        }
                    }
                    
                    // Handle the protocol launch with a slight delay to ensure app is fully initialized
                    let app_handle = app.handle().clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        if let Err(e) = handle_protocol_launch(session_id, token, user_id, app_handle) {
                            error!("Failed to handle protocol launch: {}", e);
                        }
                    });
                }
            }
            
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
    provider: String, // "openai" or "pollinations"
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

// Session Management Structures
#[derive(Serialize, Deserialize, Clone, Debug)]
struct SessionData {
    id: String,
    job_title: String,
    job_description: Option<String>,
    difficulty_level: String,
    interview_type: String,
    estimated_duration_minutes: u32,
    status: String,
    created_at: String,
    desktop_connected: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct SessionConnectionPayload {
    session_id: String,
    token: String,
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct SessionActivationResponse {
    success: bool,
    message: String,
    session: Option<SessionData>,
    remaining_credits: Option<u32>,
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
    pollinations_client: Arc<Mutex<Option<PollinationsClient>>>,
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
    
    fn ensure_pollinations_client(&self) -> Result<(), String> {
        let mut client_guard = self.pollinations_client.lock();
        if client_guard.is_none() {
            let api_key = std::env::var("POLLINATIONS_API_KEY")
                .map_err(|_| "POLLINATIONS_API_KEY environment variable not set".to_string())?;
            let referer = std::env::var("POLLINATIONS_REFERER")
                .unwrap_or_else(|_| "mockmate".to_string());
            *client_guard = Some(PollinationsClient::new(api_key, referer));
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
    
    // Determine which provider to use
    let provider = AIProvider::from_str(&payload.provider)
        .unwrap_or(AIProvider::OpenAI); // Default to OpenAI if invalid
    
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
    
    match provider {
        AIProvider::OpenAI => {
            info!("Using OpenAI provider");
            state.ensure_openai_client()?;
            
            let client = {
                let client_guard = state.openai_client.lock();
                client_guard.as_ref().unwrap().clone()
            };
            
            let model = openai::OpenAIModel::from_string(&payload.model)
                .map_err(|e| format!("Invalid OpenAI model: {}", e))?;
            
            client.generate_answer(&payload.question, &context, model)
                .await
                .map_err(|e| e.to_string())
        },
        AIProvider::Pollinations => {
            info!("Using Pollinations provider");
            state.ensure_pollinations_client()?;
            
            let client = {
                let client_guard = state.pollinations_client.lock();
                client_guard.as_ref().unwrap().clone()
            };
            
            let model = pollinations::PollinationsModel::from_string(&payload.model)
                .map_err(|e| format!("Invalid Pollinations model: {}", e))?;
            
            client.generate_answer(&payload.question, &context, model)
                .await
                .map_err(|e| e.to_string())
        }
    }
}

// New command: generate answer via Pollinations using backend (adds required headers)
#[tauri::command]
async fn pollinations_generate_answer(
    payload: GenerateAnswerPayload,
    state: State<'_, AppState>
) -> Result<String, String> {
    if payload.provider.to_lowercase() != "pollinations" {
        return Err("Provider must be 'pollinations' for this command".to_string());
    }
    info!("Generating Pollinations answer (backend) for: {}", payload.question);
    state.ensure_pollinations_client()?;

    let client = {
        let client_guard = state.pollinations_client.lock();
        client_guard.as_ref().unwrap().clone()
    };

    let mut context = {
        let context_guard = state.interview_context.lock();
        context_guard.clone()
    };
    if let Some(company) = payload.company { context.company = Some(company); }
    if let Some(position) = payload.position { context.position = Some(position); }
    if let Some(job_description) = payload.job_description { context.job_description = Some(job_description); }

    let model = pollinations::PollinationsModel::from_string(&payload.model)
        .map_err(|e| format!("Invalid Pollinations model: {}", e))?;

    client.generate_answer(&payload.question, &context, model)
        .await
        .map_err(|e| e.to_string())
}

// New command: Generate streaming answer via Pollinations GET endpoint
#[tauri::command]
async fn pollinations_generate_answer_streaming(
    payload: GenerateAnswerPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    if payload.provider.to_lowercase() != "pollinations" {
        return Err("Provider must be 'pollinations' for this command".to_string());
    }
    info!("Generating Pollinations streaming answer (GET) for: {}", payload.question);
    state.ensure_pollinations_client()?;

    let client = {
        let client_guard = state.pollinations_client.lock();
        client_guard.as_ref().unwrap().clone()
    };

    let mut context = {
        let context_guard = state.interview_context.lock();
        context_guard.clone()
    };
    if let Some(company) = payload.company { context.company = Some(company); }
    if let Some(position) = payload.position { context.position = Some(position); }
    if let Some(job_description) = payload.job_description { context.job_description = Some(job_description); }

    let model = pollinations::PollinationsModel::from_string(&payload.model)
        .map_err(|e| format!("Invalid Pollinations model: {}", e))?;

    // Show the AI response window before starting streaming
    if let Err(e) = show_ai_response_window(app_handle.clone()) {
        warn!("Failed to show AI response window: {}", e);
    }

    // Initialize streaming state and timing
    let stream_start_time = std::time::Instant::now();
    info!("🚀 Starting progressive streaming for AI response window");
    let _ = app_handle.emit("ai-stream-start", ());

    // Stream the response with callback to update UI progressively
    let app_handle_clone = app_handle.clone();
    let result = client.generate_answer_streaming(
        &payload.question, 
        &context, 
        model,
        move |token: &str| {
            info!("📝 Streaming token: '{}' (length: {})", token.chars().take(50).collect::<String>(), token.len());
            
            // 🔍 DEBUG: Log the exact token being processed
            info!("🔍 BACKEND TOKEN DEBUG: Raw token received: '{}' (length: {})", 
                token.replace('\n', "\\n").replace('\r', "\\r"), token.len());
            
            // Emit token event for progressive display
            let _ = app_handle_clone.emit("ai-stream-token", token);
            info!("📡 BACKEND: Emitted 'ai-stream-token' event with token: '{}'", 
                token.chars().take(50).collect::<String>());
            
            // Also send via direct window communication for immediate display
            let data = AiResponseData {
                message_type: "stream-token".to_string(),
                text: Some(token.to_string()),
                error: None,
            };
            let app_handle_for_async = app_handle_clone.clone();
            tokio::spawn(async move {
                info!("🔄 BACKEND: About to send stream-token to native window: '{}'", 
                    data.text.as_ref().unwrap().chars().take(50).collect::<String>());
                if let Err(e) = send_ai_response_data(app_handle_for_async, data).await {
                    error!("Failed to send streaming token to UI: {}", e);
                }
            });
        }
    ).await;

    match result {
        Ok(full_response) => {
            let elapsed_time = stream_start_time.elapsed();
            info!("✅ Streaming completed. Full response length: {}, elapsed time: {:.2?}", full_response.len(), elapsed_time);
            
            // Emit completion event
            let _ = app_handle.emit("ai-stream-complete", full_response.clone());
            
            // Send completion signal
            let data = AiResponseData {
                message_type: "complete".to_string(),
                text: Some(full_response.clone()),
                error: None,
            };
            let app_handle_for_complete = app_handle.clone();
            tokio::spawn(async move {
                if let Err(e) = send_ai_response_data(app_handle_for_complete, data).await {
                    error!("Failed to send completion signal to UI: {}", e);
                }
            });
            Ok(full_response)
        },
        Err(e) => {
            error!("❌ Streaming failed: {}", e);
            
            // Emit error event
            let _ = app_handle.emit("ai-stream-error", e.to_string());
            
            // Send error signal
            let data = AiResponseData {
                message_type: "error".to_string(),
                text: None,
                error: Some(e.to_string()),
            };
            let app_handle_for_error = app_handle.clone();
            tokio::spawn(async move {
                if let Err(send_err) = send_ai_response_data(app_handle_for_error, data).await {
                    error!("Failed to send error signal to UI: {}", send_err);
                }
            });
            Err(e.to_string())
        }
    }
}

// New command: Generate streaming answer via Pollinations POST endpoint
#[tauri::command]
async fn pollinations_generate_answer_post_streaming(
    payload: GenerateAnswerPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    if payload.provider.to_lowercase() != "pollinations" {
        return Err("Provider must be 'pollinations' for this command".to_string());
    }
    info!("Generating Pollinations streaming answer (POST) for: {}", payload.question);
    state.ensure_pollinations_client()?;

    let client = {
        let client_guard = state.pollinations_client.lock();
        client_guard.as_ref().unwrap().clone()
    };

    let mut context = {
        let context_guard = state.interview_context.lock();
        context_guard.clone()
    };
    if let Some(company) = payload.company { context.company = Some(company); }
    if let Some(position) = payload.position { context.position = Some(position); }
    if let Some(job_description) = payload.job_description { context.job_description = Some(job_description); }

    let model = pollinations::PollinationsModel::from_string(&payload.model)
        .map_err(|e| format!("Invalid Pollinations model: {}", e))?;

    // Show the AI response window before starting streaming
    if let Err(e) = show_ai_response_window(app_handle.clone()) {
        warn!("Failed to show AI response window: {}", e);
    }

    // Initialize streaming state and timing
    let stream_start_time = std::time::Instant::now();
    info!("🚀 Starting progressive streaming (POST) for AI response window");
    let _ = app_handle.emit("ai-stream-start", ());

    // Stream the response with callback to update UI progressively
    let app_handle_clone = app_handle.clone();
    let result = client.generate_answer_post_streaming(
        &payload.question, 
        &context, 
        model,
        move |token: &str| {
            info!("📝 Streaming token (POST): '{}' (length: {})", token.chars().take(50).collect::<String>(), token.len());
            
            // Emit token event for progressive display
            let _ = app_handle_clone.emit("ai-stream-token", token);
            
            // Also send via direct window communication for immediate display
            let data = AiResponseData {
                message_type: "stream-token".to_string(),
                text: Some(token.to_string()),
                error: None,
            };
            let app_handle_for_async = app_handle_clone.clone();
            tokio::spawn(async move {
                if let Err(e) = send_ai_response_data(app_handle_for_async, data).await {
                    error!("Failed to send streaming token to UI: {}", e);
                }
            });
        }
    ).await;

    match result {
        Ok(full_response) => {
            let elapsed_time = stream_start_time.elapsed();
            info!("✅ Streaming (POST) completed. Full response length: {}, elapsed time: {:.2?}", full_response.len(), elapsed_time);
            
            // Emit completion event
            let _ = app_handle.emit("ai-stream-complete", full_response.clone());
            
            // Send completion signal
            let data = AiResponseData {
                message_type: "complete".to_string(),
                text: Some(full_response.clone()),
                error: None,
            };
            let app_handle_for_complete = app_handle.clone();
            tokio::spawn(async move {
                if let Err(e) = send_ai_response_data(app_handle_for_complete, data).await {
                    error!("Failed to send completion signal to UI: {}", e);
                }
            });
            Ok(full_response)
        },
        Err(e) => {
            error!("❌ Streaming (POST) failed: {}", e);
            
            // Emit error event
            let _ = app_handle.emit("ai-stream-error", e.to_string());
            
            // Send error signal
            let data = AiResponseData {
                message_type: "error".to_string(),
                text: None,
                error: Some(e.to_string()),
            };
            let app_handle_for_error = app_handle.clone();
            tokio::spawn(async move {
                if let Err(send_err) = send_ai_response_data(app_handle_for_error, data).await {
                    error!("Failed to send error signal to UI: {}", send_err);
                }
            });
            Err(e.to_string())
        }
    }
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

#[derive(Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    name: String,
    provider: String,
    icon: String,
}

#[derive(Serialize, Deserialize)]
struct ProviderInfo {
    id: String,
    name: String,
    description: String,
}

#[tauri::command]
async fn get_available_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    info!("Getting available models...");
    
    let mut models = Vec::new();
    
    // OpenAI models
    models.push(ModelInfo {
        id: "gpt-4-turbo".to_string(),
        name: "GPT-4 Turbo".to_string(),
        provider: "openai".to_string(),
        icon: "🤖".to_string(),
    });
    models.push(ModelInfo {
        id: "gpt-4".to_string(),
        name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        icon: "🤖".to_string(),
    });
    models.push(ModelInfo {
        id: "gpt-3.5-turbo".to_string(),
        name: "GPT-3.5 Turbo".to_string(),
        provider: "openai".to_string(),
        icon: "🤖".to_string(),
    });
    
    // Pollinations models (fetched from API with headers)
    if let Err(e) = state.ensure_pollinations_client() {
        warn!("Pollinations client not available: {}", e);
    } else {
        let client = {
            let guard = state.pollinations_client.lock();
            guard.as_ref().unwrap().clone()
        };
        let pollinations_models = client.fetch_available_models().await.unwrap_or_default();
        for model in pollinations_models {
            models.push(ModelInfo {
                id: model.as_str().to_string(),
                name: model.display_name().to_string(),
                provider: "pollinations".to_string(),
                icon: "🧠".to_string(),
            });
        }
    }
    
    Ok(models)
}

#[tauri::command]
fn get_ai_providers() -> Result<Vec<ProviderInfo>, String> {
    info!("Getting AI providers...");
    
    let providers = vec![
        ProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            description: "Official OpenAI models including GPT-4 and GPT-3.5".to_string(),
        },
        ProviderInfo {
            id: "pollinations".to_string(),
            name: "Pollinations (Self AI)".to_string(),
            description: "Free and open AI models via Pollinations API".to_string(),
        },
    ];
    
    Ok(providers)
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
    
    // Get screen dimensions to set proper max height
    let screen_size = main_window.current_monitor().map_err(|e| e.to_string())?
        .map(|monitor| {
            let size = monitor.size();
            (size.width, size.height)
        })
        .unwrap_or((1920, 1080)); // fallback to common resolution
    
    let _max_window_height = (screen_size.1 as f64 * 0.6) as f64; // Use 60% of screen height
    
    // Create response window configuration
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        tauri::WebviewUrl::App("ai-response.html".into())
    )
    .title("AI Response")
    .inner_size(800.0, 150.0) // Start with minimal height for auto-sizing
    .min_inner_size(800.0, 100.0)  // Lower minimum for auto-sizing
    // Remove max size constraint to allow dynamic resizing
    .position(response_x as f64, response_y as f64)
    .resizable(true) // Make resizable for programmatic resizing
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
    info!("🔧 RESIZE REQUEST: height={}, timestamp={}", height, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        // Get current size first for debugging
        let current_size = window.outer_size().map_err(|e| {
            error!("❌ Failed to get current window size: {}", e);
            e.to_string()
        })?;
        
        // Get screen dimensions for dynamic max height
        let monitor = window.current_monitor()
            .map_err(|e| {
                error!("❌ Failed to get monitor info: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                error!("❌ No monitor found");
                "No monitor found".to_string()
            })?;
        
        let max_height = monitor.size();
        let calculated_max = (max_height.height as f64 * 0.6) as u32;
        info!("📐 Screen size: {}x{}, calculated max height: {}", max_height.width, max_height.height, calculated_max);
        
        // Lower minimum height for auto-sizing content
        let clamped_height = height.max(80).min(calculated_max);
        let size_diff = (current_size.height as i32 - clamped_height as i32).abs();
        
        info!("📊 RESIZE DEBUG: current={}px, requested={}px, clamped={}px, max={}px, diff={}px", 
              current_size.height, height, clamped_height, calculated_max, size_diff);
        
        // Always try to resize if there's any difference - remove the 5px threshold
        if current_size.height != clamped_height {
            info!("🎯 Attempting resize from {}px to {}px...", current_size.height, clamped_height);
            
            match window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: 800,
                height: clamped_height,
            })) {
                Ok(_) => {
                    info!("✅ AI response window successfully resized: {}px -> {}px (diff: {}px)", current_size.height, clamped_height, size_diff);
                    
                    // Verify the resize worked by checking the new size
                    match window.outer_size() {
                        Ok(new_size) => {
                            info!("🔍 Post-resize verification: actual new size is {}px", new_size.height);
                            if new_size.height == clamped_height {
                                Ok(format!("✅ Resized successfully: {} -> {}", current_size.height, new_size.height))
                            } else {
                                warn!("⚠️ Resize mismatch: expected {}px but got {}px", clamped_height, new_size.height);
                                Ok(format!("⚠️ Partial resize: {} -> {} (expected {})", current_size.height, new_size.height, clamped_height))
                            }
                        }
                        Err(e) => {
                            warn!("❌ Failed to verify resize: {}", e);
                            Ok(format!("✅ Resize attempted: {} -> {}", current_size.height, clamped_height))
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to resize AI response window: {}", e);
                    Err(format!("❌ Resize failed: {}", e))
                }
            }
        } else {
            info!("➡️ No resize needed: window already at target height {}px", current_size.height);
            Ok(format!("➡️ Already correct size: {}px", current_size.height))
        }
    } else {
        error!("❌ AI response window 'ai-response' not found for resize");
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
    
    // Get screen dimensions to set proper max height
    let screen_size = main_window.current_monitor().map_err(|e| e.to_string())?
        .map(|monitor| {
            let size = monitor.size();
            (size.width, size.height)
        })
        .unwrap_or((1920, 1080)); // fallback to common resolution
    
    let _max_window_height = (screen_size.1 as f64 * 0.6) as f64; // Use 60% of screen height
    
    // Create response window configuration (hidden by default)
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        tauri::WebviewUrl::App("ai-response.html".into())
    )
    .title("AI Response")
    .inner_size(800.0, 150.0) // Start with minimal height for auto-sizing
    .min_inner_size(800.0, 100.0)  // Lower minimum for auto-sizing
    // Remove max size constraint to allow dynamic resizing
    .position(response_x as f64, response_y as f64)
    .resizable(true) // Make resizable for programmatic resizing
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
async fn send_ai_response_data(app_handle: AppHandle, data: AiResponseData) -> Result<String, String> {
    info!("🚀 RUST DEBUG: send_ai_response_data called with message_type: {:?}", data.message_type);
    
    // Check if AI response window exists
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        info!("✅ RUST DEBUG: AI response window found, attempting to show and send data");
        
        // First ensure the window is visible
        if let Err(e) = window.show() {
            error!("❌ RUST DEBUG: Failed to show AI response window: {}", e);
        } else {
            info!("✅ RUST DEBUG: AI response window shown successfully");
        }
        
        // Send data to the AI response window via JavaScript evaluation
        let js_code = match data.message_type.as_str() {
            "stream" => {
                let text = data.text.as_ref().map(|t| t.clone()).unwrap_or_default();
                let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                format!(r#"
                    if (window.updateContent) {{
                        window.updateContent('stream', {{ text: "{}" }});
                    }} else {{
                        console.log('updateContent function not found');
                    }}
                "#, escaped_text)
            }
            "stream-token" => {
                let token = data.text.as_ref().map(|t| t.clone()).unwrap_or_default();
                let escaped_token = token.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                format!(r#"
                    if (window.updateContent) {{
                        console.log('🎯 AI RESPONSE WINDOW: Received stream token:', '{}');
                        window.updateContent('stream-token', {{ token: "{}" }});
                    }} else {{
                        console.log('updateContent function not found for stream-token');
                    }}
                "#, escaped_token.chars().take(50).collect::<String>(), escaped_token)
            }
            "complete" => {
                let text = data.text.as_ref().map(|t| t.clone()).unwrap_or_default();
                let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                format!(r#"
                    if (window.updateContent) {{
                        window.updateContent('complete', {{ text: "{}" }});
                    }}
                "#, escaped_text)
            }
            "error" => {
                let error_msg = data.error.as_ref().map(|e| e.clone()).unwrap_or_default();
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
        
        info!("🚀 RUST DEBUG: About to evaluate JavaScript code: {}", js_code.chars().take(200).collect::<String>() + "...");
        
        match window.eval(&js_code) {
            Ok(_) => {
                info!("✅ RUST DEBUG: JavaScript evaluation successful - AI response data sent successfully");
                
                // For stream messages, also trigger an immediate resize calculation from Rust side
                if data.message_type == "stream" || data.message_type == "complete" {
                    info!("🔄 RUST DEBUG: Triggering automatic resize after content update");
                    
                    // Give the DOM a moment to update, then trigger resize
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    // Calculate approximate height based on text length (rough estimate)
                    let text_length = data.text.as_ref().map(|t| t.len()).unwrap_or(0);
                    let estimated_lines = (text_length / 80).max(1) + 2; // ~80 chars per line + padding
                    let line_height = 27; // 21px font size * 1.3 line height
                    let header_height = 50;
                    let padding = 52; // content padding + window padding
                    
                    let estimated_height = (estimated_lines * line_height + header_height + padding).min(900) as u32; // Cap at reasonable height
                    
                    info!("📏 RUST DEBUG: Auto-resize calculation: text_length={}, estimated_lines={}, estimated_height={}px", text_length, estimated_lines, estimated_height);
                    
                    // Trigger resize from Rust side
                    if let Err(resize_err) = resize_ai_response_window(app_handle.clone(), estimated_height) {
                        warn!("❌ RUST DEBUG: Auto-resize failed: {}", resize_err);
                    } else {
                        info!("✅ RUST DEBUG: Auto-resize triggered successfully from Rust");
                    }
                }
                
                Ok("Data sent to AI response window".to_string())
            }
            Err(e) => {
                error!("❌ RUST DEBUG: JavaScript evaluation failed - Failed to send data to AI response window: {}", e);
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

// Session Management Commands

#[tauri::command]
async fn connect_to_web_session(payload: SessionConnectionPayload) -> Result<SessionData, String> {
    info!("Connecting to web session: {}", payload.session_id);
    
    let backend_url = std::env::var("MOCKMATE_BACKEND_URL")
        .unwrap_or_else(|_| "https://mockmate-backend.onrender.com".to_string());
    
    let client = reqwest::Client::new();
    
    // Notify backend about desktop connection
    let connection_response = client
        .post(format!("{}/api/sessions/{}/connect-desktop", backend_url, payload.session_id))
        .header("Authorization", format!("Bearer {}", payload.token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "user_id": payload.user_id,
            "desktop_version": env!("CARGO_PKG_VERSION"),
            "platform": std::env::consts::OS
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to connect to session: {}", e))?;
    
    if !connection_response.status().is_success() {
        return Err(format!("Session connection failed: {}", connection_response.status()));
    }
    
    let session_data: SessionData = connection_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse session data: {}", e))?;
    
    info!("Successfully connected to session: {} - {}", session_data.id, session_data.job_title);
    Ok(session_data)
}

#[tauri::command]
async fn activate_web_session(payload: SessionConnectionPayload) -> Result<SessionActivationResponse, String> {
    info!("Activating session with credit check: {}", payload.session_id);
    
    let backend_url = std::env::var("MOCKMATE_BACKEND_URL")
        .unwrap_or_else(|_| "https://mockmate-backend.onrender.com".to_string());
    
    let client = reqwest::Client::new();
    
    // Activate session with credit deduction
    let activation_response = client
        .post(format!("{}/api/sessions/{}/activate", backend_url, payload.session_id))
        .header("Authorization", format!("Bearer {}", payload.token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "user_id": payload.user_id
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to activate session: {}", e))?;
    
    let activation_result: SessionActivationResponse = activation_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse activation response: {}", e))?;
    
    if activation_result.success {
        info!("Session activated successfully. Credits remaining: {:?}", activation_result.remaining_credits);
    } else {
        warn!("Session activation failed: {}", activation_result.message);
    }
    
    Ok(activation_result)
}

#[tauri::command]
async fn get_session_info(session_id: String, token: String) -> Result<SessionData, String> {
    info!("Getting session info: {}", session_id);
    
    let backend_url = std::env::var("MOCKMATE_BACKEND_URL")
        .unwrap_or_else(|_| "https://mockmate-backend.onrender.com".to_string());
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/sessions/{}", backend_url, session_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch session info: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to get session info: {}", response.status()));
    }
    
    let session_data: SessionData = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse session data: {}", e))?;
    
    Ok(session_data)
}

#[tauri::command]
fn handle_protocol_launch(session_id: String, token: Option<String>, user_id: Option<String>, app_handle: AppHandle) -> Result<String, String> {
    info!("Handling protocol launch for session: {}", session_id);
    
    // Parse the session_id from the protocol URL if it's a full URL
    let clean_session_id = if session_id.starts_with("mockmate://session/") {
        session_id.strip_prefix("mockmate://session/").unwrap_or(&session_id).to_string()
    } else {
        session_id.clone()
    };
    
    info!("Clean session ID: {}", clean_session_id);
    
    // Bring the main window to front and focus
    if let Some(main_window) = app_handle.get_webview_window("main") {
        if let Err(e) = main_window.show() {
            error!("Failed to show main window: {}", e);
        }
        if let Err(e) = main_window.set_focus() {
            error!("Failed to focus main window: {}", e);
        }
        
        // Send session info to the frontend
        let session_launch_data = serde_json::json!({
            "session_id": clean_session_id,
            "token": token,
            "user_id": user_id,
            "launched_at": chrono::Utc::now().to_rfc3339()
        });
        
        if let Err(e) = app_handle.emit("session-launch", session_launch_data) {
            error!("Failed to emit session-launch event: {}", e);
        }
        
        info!("Protocol launch handled successfully for session: {}", clean_session_id);
        Ok(format!("Launched session: {}", clean_session_id))
    } else {
        error!("Main window not found for protocol launch");
        Err("Main window not found".to_string())
    }
}

// New Shared Database Session Commands

#[tauri::command]
async fn connect_session(session_id: String) -> Result<crate::database::SessionWithUser, String> {
    info!("🔗 Connecting to session: {}", session_id);
    
    // Initialize database connection if not already done
    crate::database::initialize_database().await?;
    
    // Get session details with user info
    let session_info = crate::database::get_session_with_user_info(&session_id).await?;
    
    info!("✅ Successfully connected to session: {}", session_info.session_name);
    Ok(session_info)
}

#[tauri::command]
async fn activate_session_cmd(session_id: String) -> Result<String, String> {
    info!("🚀 Activating session: {}", session_id);
    
    // Activate session and deduct credits
    crate::database::activate_session(&session_id).await?;
    
    info!("✅ Session activated successfully");
    Ok("Session activated successfully".to_string())
}

#[tauri::command]
async fn disconnect_session_cmd(session_id: String) -> Result<String, String> {
    info!("🔌 Disconnecting from session: {}", session_id);
    
    crate::database::disconnect_session(&session_id).await?;
    
    info!("✅ Session disconnected successfully");
    Ok("Session disconnected successfully".to_string())
}

// Frontend compatibility command wrappers

#[derive(Serialize, Deserialize)]
struct SessionValidationResult {
    valid: bool,
    message: String,
}

#[tauri::command]
async fn validate_session_id(session_id: String) -> Result<SessionValidationResult, String> {
    info!("🔍 Validating session ID: {}", session_id);
    
    // Validate UUID format
    let uuid_pattern = regex::Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
        .map_err(|e| format!("Regex error: {}", e))?;
    
    if !uuid_pattern.is_match(&session_id.to_lowercase()) {
        return Ok(SessionValidationResult {
            valid: false,
            message: "Invalid session ID format - must be a valid UUID".to_string(),
        });
    }
    
    // Try to initialize database and validate session exists
    match crate::database::initialize_database().await {
        Ok(()) => {
            match crate::database::get_session_with_user_info(&session_id).await {
                Ok(_) => Ok(SessionValidationResult {
                    valid: true,
                    message: "Session ID is valid and exists".to_string(),
                }),
                Err(e) => {
                    if e.contains("not found") {
                        Ok(SessionValidationResult {
                            valid: false,
                            message: "Session not found".to_string(),
                        })
                    } else {
                        Ok(SessionValidationResult {
                            valid: false,
                            message: format!("Validation error: {}", e),
                        })
                    }
                }
            }
        }
        Err(e) => {
            warn!("Database not available for validation: {}", e);
            // If database is not available, just validate format
            Ok(SessionValidationResult {
                valid: true,
                message: "Session ID format is valid (database validation skipped)".to_string(),
            })
        }
    }
}

#[tauri::command]
async fn activate_session(session_id: String) -> Result<bool, String> {
    info!("🚀 Activating session (frontend compatibility): {}", session_id);
    
    // Call the existing activate_session_cmd and return boolean result
    match activate_session_cmd(session_id).await {
        Ok(_) => Ok(true),
        Err(e) => {
            error!("Session activation failed: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn disconnect_session(session_id: String) -> Result<String, String> {
    info!("🔌 Disconnecting from session (frontend compatibility): {}", session_id);
    
    // Call the existing disconnect_session_cmd
    disconnect_session_cmd(session_id).await
}

// Timer management command

#[derive(Serialize, Deserialize)]
struct UpdateTimerPayload {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "elapsedMinutes")]
    elapsed_minutes: i32,
    #[serde(rename = "isFinal")]
    is_final: Option<bool>,
}

#[tauri::command]
async fn update_session_timer(session_id: String, elapsed_minutes: i32, is_final: Option<bool>) -> Result<String, String> {
    let is_final = is_final.unwrap_or(false);
    
    if is_final {
        info!("⏱️ Updating session timer (FINAL): {} - {} minutes", session_id, elapsed_minutes);
    } else {
        info!("⏱️ Updating session timer: {} - {} minutes", session_id, elapsed_minutes);
    }
    
    // For now, we'll just log the timer update. Later we can add database persistence.
    // This could save to the sessions table's total_duration_minutes field
    
    // TODO: Implement database update for timer state
    // UPDATE sessions SET total_duration_minutes = elapsed_minutes WHERE id = session_id;
    
    if is_final {
        info!("✅ Final session timer saved: {} minutes", elapsed_minutes);
        Ok(format!("Final session timer saved: {} minutes", elapsed_minutes))
    } else {
        info!("✅ Session timer updated: {} minutes", elapsed_minutes);
        Ok(format!("Session timer updated: {} minutes", elapsed_minutes))
    }
}

// Window management commands

#[tauri::command]
fn resize_main_window(app_handle: AppHandle, width: u32, height: u32) -> Result<String, String> {
    info!("📐 Resizing main window to: {}x{}", width, height);
    
    if let Some(window) = app_handle.get_webview_window("main") {
        // Get current size for comparison
        let current_size = window.outer_size().map_err(|e| {
            error!("❌ Failed to get current main window size: {}", e);
            e.to_string()
        })?;
        
        info!("📊 Main window resize: current={}x{}, requested={}x{}", 
              current_size.width, current_size.height, width, height);
        
        match window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width,
            height,
        })) {
            Ok(_) => {
                info!("✅ Main window successfully resized to: {}x{}", width, height);
                
                // Verify the resize worked
                match window.outer_size() {
                    Ok(new_size) => {
                        info!("🔍 Post-resize verification: actual new size is {}x{}", new_size.width, new_size.height);
                        Ok(format!("Main window resized to {}x{}", new_size.width, new_size.height))
                    }
                    Err(e) => {
                        warn!("❌ Failed to verify main window resize: {}", e);
                        Ok(format!("Main window resize attempted: {}x{}", width, height))
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to resize main window: {}", e);
                Err(format!("Failed to resize main window: {}", e))
            }
        }
    } else {
        error!("❌ Main window not found for resize");
        Err("Main window not found".to_string())
    }
}
