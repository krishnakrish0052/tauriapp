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
pub mod accessibility_reader; // Windows Accessibility API text reader
pub mod window_manager; // DPI-aware window management
pub mod permissions; // Permission management for audio access
pub mod stereo_mix_manager; // Windows Stereo Mix automatic enablement

// New Phase 2 modules
pub mod database;
// pub mod session; // Temporarily disabled to avoid conflicts
// pub mod interview; // Temporarily disabled to avoid conflicts

use openai::{OpenAIClient, InterviewContext};
use pollinations::{PollinationsClient, AIProvider};
// use database::shared::*; // Import shared database types and functions - commented out to avoid unused import warning

pub fn run() -> Result<()> {
    // Environment variables are now embedded at build time via build.rs
    // We'll use env!() macro to access them, with fallbacks to runtime env::var() for development
    info!("MockMate starting with embedded environment configuration...");
    
    // Log which environment variables are available
    log_environment_status();

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
            reset_ai_response_window_size,
            create_ai_response_window_enhanced_below,
            reset_ai_response_window_enhanced_below_size,
            // Real-time transcription commands
            realtime_transcription::start_microphone_transcription,
            realtime_transcription::start_system_audio_transcription,
            realtime_transcription::stop_transcription,
            realtime_transcription::get_transcription_status,
            realtime_transcription::get_deepgram_config,
            realtime_transcription::test_deepgram_connection,
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
            // AI Analysis commands (accessibility-based)
            analyze_screen_with_ai,
            analyze_screen_with_ai_streaming,
            // Windows Accessibility API commands (Primary text reading solution)
            accessibility_reader::read_text_from_applications,
            accessibility_reader::read_text_from_focused_window,
            accessibility_reader::read_text_from_background_windows,
            accessibility_reader::read_text_from_current_window,
            // NEW: Commands for targeting window behind MockMate (interviewer's window)
            accessibility_reader::read_text_from_window_behind_mockmate,
            accessibility_reader::capture_previous_focused_window,
            // Real-time monitoring commands
            accessibility_reader::start_realtime_monitoring,
            accessibility_reader::stop_realtime_monitoring,
            accessibility_reader::get_monitoring_status,
            // Hybrid approach commands
            accessibility_reader::extract_text_hybrid_approach,
            accessibility_reader::update_accessibility_config,
            // Accessibility-based AI analysis commands
            analyze_applications_with_ai_streaming,
            analyze_focused_window_with_ai_streaming,
            // Session management commands (existing)
            connect_to_web_session,
            activate_web_session,
            get_session_info,
            handle_protocol_launch,
            // Temporary token authentication
            connect_with_temp_token,
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
            database::postgres::mark_session_started,
            // Window management
            resize_main_window,
            move_window_relative,
            resize_window_scale,
            show_main_window,
            hide_main_window,
            // DPI-aware window management
            setup_dpi_aware_positioning,
            get_window_info,
            get_monitors_info,
            lock_window_size,
            ensure_window_visible,
            // Database diagnostics
            diagnose_database,
            test_session_query,
            // Permission management
            permissions::check_permissions,
            permissions::request_permissions,
            permissions::initialize_first_run,
            // Stereo Mix management
            stereo_mix_manager::check_stereo_mix_enabled,
            stereo_mix_manager::enable_stereo_mix,
            stereo_mix_manager::open_recording_devices,
            stereo_mix_manager::get_stereo_mix_capabilities,
            stereo_mix_manager::get_stereo_mix_instructions
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
                    let mut temp_token: Option<String> = None;
                    let mut user_id: Option<String> = None;
                    let mut auto_connect: Option<bool> = None;
                    let mut auto_fill: Option<bool> = None;
                    
                    if parts.len() > 1 {
                        for param in parts[1].split('&') {
                            let kv: Vec<&str> = param.split('=').collect();
                            if kv.len() == 2 {
                                match kv[0] {
                                    "token" => token = Some(urlencoding::decode(kv[1]).unwrap_or_default().to_string()),
                                    "temp_token" => temp_token = Some(urlencoding::decode(kv[1]).unwrap_or_default().to_string()),
                                    "user_id" => user_id = Some(urlencoding::decode(kv[1]).unwrap_or_default().to_string()),
                                    "auto_connect" => auto_connect = Some(kv[1] == "true"),
                                    "auto_fill" => auto_fill = Some(kv[1] == "true"),
                                    _ => {}
                                }
                            }
                        }
                    }
                    
                    info!("Protocol launch parameters: temp_token={}, auto_connect={:?}, auto_fill={:?}", 
                          temp_token.is_some(), auto_connect, auto_fill);
                    
                    // Handle the protocol launch with a slight delay to ensure app is fully initialized
                    let app_handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        if let Err(e) = handle_protocol_launch_with_temp_token(session_id, token, temp_token, user_id, auto_connect, auto_fill, app_handle).await {
                            error!("Failed to handle protocol launch: {}", e);
                        }
                    });
                }
            }
            
            // Initialize the real-time transcription service
            realtime_transcription::init_transcription_service(app.handle().clone());
            info!("✅ Real-time transcription service initialized");
            
            // Initialize the real-time accessibility monitoring service
            accessibility_reader::init_realtime_monitoring(app.handle().clone());
            info!("✅ Real-time accessibility monitoring service initialized");
            
            // Get the main window and set capture protection + DPI-aware positioning
            match app.get_webview_window("main") {
                Some(main_window) => {
                    info!("Main window found. Attempting to set capture protection.");
                    if let Err(e) = set_window_capture_protection(&main_window, true) {
                        error!("Failed to set window capture protection on startup: {}", e);
                    }
                    
                    // Initialize DPI-aware positioning for the main window
                    info!("🖥️ Initializing DPI-aware positioning for main window...");
                    if let Err(e) = window_manager::setup_main_window_dpi_aware(app.handle()) {
                        error!("Failed to setup DPI-aware positioning on startup: {}", e);
                    } else {
                        info!("✅ DPI-aware positioning initialized successfully");
                    }
                    
                    // Log current window configuration for debugging
                    if let Ok(config) = window_manager::get_window_configuration(&main_window) {
                        info!("📊 Window configuration: {}x{} at ({}, {}) on monitor '{}'", 
                              config.width, config.height, config.x, config.y, config.monitor_name);
                    }
                },
                None => {
                    error!("Main window not found on startup. Capture protection and DPI setup not applied.");
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
            
            // Initialize permissions on first run - defer to avoid runtime context issues
            let app_handle_perms = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = permissions::PermissionManager::initialize_permissions_on_first_run() {
                    error!("Failed to initialize first-run permissions: {}", e);
                } else {
                    info!("✅ First-run permission initialization completed");
                }
            });
            
            // Initialize environment variables if needed
            match get_env_var("DEEPGRAM_API_KEY") {
                Some(_) => info!("✅ DEEPGRAM_API_KEY loaded successfully"),
                None => warn!("❌ DEEPGRAM_API_KEY not set in environment - transcription will not work")
            }
            match get_env_var("OPENAI_API_KEY") {
                Some(_) => info!("✅ OPENAI_API_KEY loaded successfully"),
                None => warn!("❌ OPENAI_API_KEY not set in environment - AI answers will not work")
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

// Temporary token authentication structures
#[derive(Serialize, Deserialize)]
struct TempTokenAuthPayload {
    session_id: String,
    temp_token: String,
}

#[derive(Serialize, Deserialize)]
struct TempTokenAuthResponse {
    success: bool,
    message: String,
    user_id: Option<String>,
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
            let api_key = get_env_var("OPENAI_API_KEY")
                .ok_or_else(|| "OPENAI_API_KEY environment variable not set".to_string())?;
            *client_guard = Some(OpenAIClient::new(api_key));
        }
        Ok(())
    }
    
    fn ensure_pollinations_client(&self) -> Result<(), String> {
        let mut client_guard = self.pollinations_client.lock();
        if client_guard.is_none() {
            let api_key = get_env_var("POLLINATIONS_API_KEY")
                .ok_or_else(|| "POLLINATIONS_API_KEY environment variable not set".to_string())?;
            let referer = get_env_var("POLLINATIONS_REFERER")
                .unwrap_or_else(|| "mockmate".to_string());
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
    let model_clone = model.clone(); // Clone for fallback use

    // Immediately show the AI response window for faster response (no await)
    let app_handle_show = app_handle.clone();
    tokio::spawn(async move {
        if let Err(e) = show_ai_response_window_async(app_handle_show).await {
            warn!("Failed to show AI response window: {}", e);
        }
    });

    // Initialize streaming state and timing
    let stream_start_time = std::time::Instant::now();
    info!("⚡ SPEED OPTIMIZED: Starting progressive streaming for AI response window");
    let _ = app_handle.emit("ai-stream-start", ());
    
    // Skip ready signal for maximum speed - start immediately

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
            
            // Try non-streaming fallback
            info!("🔄 Attempting non-streaming fallback...");
            match client.generate_answer(&payload.question, &context, model_clone).await {
                Ok(fallback_response) => {
                    info!("✅ Non-streaming fallback succeeded: {} chars", fallback_response.len());
                    
                    // Send the fallback response
                    let data = AiResponseData {
                        message_type: "complete".to_string(),
                        text: Some(fallback_response.clone()),
                        error: None,
                    };
                    let app_handle_fallback = app_handle.clone();
                    tokio::spawn(async move {
                        if let Err(e) = send_ai_response_data(app_handle_fallback, data).await {
                            error!("Failed to send fallback response to UI: {}", e);
                        }
                    });
                    
                    let _ = app_handle.emit("ai-stream-complete", fallback_response.clone());
                    Ok(fallback_response)
                }
                Err(fallback_err) => {
                    error!("❌ Non-streaming fallback also failed: {}", fallback_err);
                    
                    // Send error signal for both failures
                    let error_message = format!("Streaming failed: {}. Fallback failed: {}", e, fallback_err);
                    let _ = app_handle.emit("ai-stream-error", error_message.clone());
                    
                    let data = AiResponseData {
                        message_type: "error".to_string(),
                        text: None,
                        error: Some(error_message.clone()),
                    };
                    let app_handle_for_error = app_handle.clone();
                    tokio::spawn(async move {
                        if let Err(send_err) = send_ai_response_data(app_handle_for_error, data).await {
                            error!("Failed to send error signal to UI: {}", send_err);
                        }
                    });
                    Err(error_message)
                }
            }
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

    // Immediately show the AI response window for faster response (no await)
    let app_handle_show = app_handle.clone();
    tokio::spawn(async move {
        if let Err(e) = show_ai_response_window_async(app_handle_show).await {
            warn!("Failed to show AI response window: {}", e);
        }
    });

    // Initialize streaming state and timing
    let stream_start_time = std::time::Instant::now();
    info!("⚡ SPEED OPTIMIZED: Starting progressive streaming (POST) for AI response window");
    let _ = app_handle.emit("ai-stream-start", ());
    
    // Send immediate status to show window is ready
    let ready_data = AiResponseData {
        message_type: "stream-token".to_string(),
        text: Some("[READY] AI response window ready, generating answer...".to_string()),
        error: None,
    };
    let app_handle_ready = app_handle.clone();
    tokio::spawn(async move {
        if let Err(e) = send_ai_response_data(app_handle_ready, ready_data).await {
            warn!("Failed to send ready signal: {}", e);
        }
    });

    // Stream the response with callback to update UI progressively
    let app_handle_clone = app_handle.clone();
    let model_clone = model.clone(); // Clone model to avoid ownership issues
    let result = client.generate_answer_post_streaming(
        &payload.question, 
        &context, 
        model_clone,
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
    
    // Get DPI scale factor first for proper scaling
    let monitor = main_window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    let scale_factor = monitor.scale_factor();
    
    info!("🔍 DEBUG: DPI Scale factor: {}", scale_factor);
    
    // Get main window measurements - use INNER for consistent sizing
    let main_inner_position = main_window.inner_position().map_err(|e| e.to_string())?;
    let main_inner_size = main_window.inner_size().map_err(|e| e.to_string())?;
    let main_outer_size = main_window.outer_size().map_err(|e| e.to_string())?;
    
    info!("🔍 DEBUG: Main window - inner: {}x{} at ({}, {}), outer: {}x{}", 
          main_inner_size.width, main_inner_size.height, main_inner_position.x, main_inner_position.y,
          main_outer_size.width, main_outer_size.height);
    
    // Calculate AI response window size - SAME WIDTH as main window's INNER content
    let max_height = 500u32; // Maximum height constraint
    
    // Use main window INNER width for exact content width match
    let ai_response_width = main_inner_size.width;
    let ai_response_height = 500u32; // Fixed default height of 500px
    
    info!("🔍 DEBUG: AI Response size calculated: {}x{} (SAME INNER WIDTH as main)", ai_response_width, ai_response_height);
    
    // Get screen size first for positioning calculations
    let screen_size = monitor.size();
    
    // FIXED DPI-aware positioning: work in logical coordinates, then convert once at the end
    // Convert screen size to logical coordinates
    let logical_screen_width = (screen_size.width as f64) / scale_factor;
    let logical_screen_height = (screen_size.height as f64) / scale_factor;
    
    // Window dimensions are already in physical pixels, convert to logical
    let logical_window_width = ai_response_width as f64 / scale_factor;
    let logical_window_height = ai_response_height as f64 / scale_factor;
    
    // Calculate center position in LOGICAL coordinates
    let logical_x = (logical_screen_width - logical_window_width) / 2.0;
    let logical_y = (logical_screen_height - logical_window_height) / 2.0;
    
    // Now convert logical position to physical for Tauri (only once!)
    let physical_x = (logical_x * scale_factor) as i32;
    let physical_y = (logical_y * scale_factor) as i32;
    
    // Window size stays as-is (already in physical pixels)
    let physical_width = ai_response_width;
    let physical_height = ai_response_height;
    
    // Use the physical coordinates for positioning
    let response_x = physical_x;
    let response_y = physical_y;
    
    info!("🔍 DEBUG: FIXED DPI-aware centering calculation:");
    info!("  - Screen size (physical): {}x{}", screen_size.width, screen_size.height);
    info!("  - DPI Scale factor: {:.2}", scale_factor);
    info!("  - Screen size (logical): {:.0}x{:.0}", logical_screen_width, logical_screen_height);
    info!("  - AI window logical size: {:.1}x{:.1}", logical_window_width, logical_window_height);
    info!("  - Logical center position: ({:.1}, {:.1})", logical_x, logical_y);
    info!("  - Physical window size: {}x{}", physical_width, physical_height);
    info!("  - Physical center position: ({}, {})", response_x, response_y);
    info!("  - FIXED: Single DPI conversion at the end");
    
    info!("📐 AI Response Window Position: main_inner={}x{} at ({}, {}), AI={}x{} at ({}, {}) [ALIGNED X & SAME WIDTH, BELOW MAIN]", 
          main_inner_size.width, main_inner_size.height, main_inner_position.x, main_inner_position.y,
          ai_response_width, ai_response_height, response_x, response_y);
    
    // Screen bounds check for safety (screen_size already obtained above)
    info!("🔍 DEBUG: Screen size: {}x{} (scale: {})", screen_size.width, screen_size.height, scale_factor);
    
    // Ensure AI response window fits on screen (positioned below main window)
    if response_x + ai_response_width as i32 > screen_size.width as i32 ||
       response_y + ai_response_height as i32 > screen_size.height as i32 {
        warn!("⚠️ AI response window would go off-screen when positioned below main window");
    }
    
    // Create response window configuration
    // Use a full URL to ensure it bypasses any React routing
    let window_url = if cfg!(debug_assertions) {
        // Development mode - serve from localhost with proper path
        tauri::WebviewUrl::External("http://localhost:1420/ai-response.html".parse().unwrap())
    } else {
        // Production mode - use app protocol with proper path
        tauri::WebviewUrl::App("ai-response.html".into())
    };
    
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        window_url
    )
    .title("AI Response")
    .inner_size(physical_width as f64, physical_height as f64)
    .min_inner_size(200.0, 100.0)  // Conservative minimum size
    .max_inner_size(physical_width as f64, 800.0 * scale_factor)  // Allow up to 800px max height with DPI scaling
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
            if let Err(e) = set_window_capture_protection(&window, true) {
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
        
        // Get main window size to calculate centered AI response width
        let main_window = app_handle.get_webview_window("main")
            .ok_or_else(|| "Main window not found for width calculation".to_string())?;
        let main_outer_size = main_window.outer_size().map_err(|e| e.to_string())?;
        let ai_response_width = main_outer_size.width; // Use SAME WIDTH as main window outer size
        
        info!("🔍 DEBUG (resize): Main outer size: {}x{}, AI width: {} (SAME WIDTH)", main_outer_size.width, main_outer_size.height, ai_response_width);
        
        // Lower minimum height for auto-sizing content, max height 500px
        let clamped_height = height.max(80).min(500); // Use 500px as max height for content-based sizing
        let size_diff = (current_size.height as i32 - clamped_height as i32).abs();
        
        info!("📊 RESIZE DEBUG: current={}px, requested={}px, clamped={}px, max={}px, diff={}px", 
              current_size.height, height, clamped_height, calculated_max, size_diff);
        
        // Always try to resize if there's any difference - remove the 5px threshold
        if current_size.height != clamped_height {
            info!("🎯 Attempting resize from {}px to {}px...", current_size.height, clamped_height);
            
            match window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: ai_response_width,  // Use fixed AI response window width
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
    
    // Get DPI scale factor first for proper scaling (startup)
    let monitor = main_window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    let scale_factor = monitor.scale_factor();
    
    info!("🔍 DEBUG (startup): DPI Scale factor: {}", scale_factor);
    
    // Get main window measurements
    let main_outer_position = main_window.outer_position().map_err(|e| e.to_string())?;
    let main_outer_size = main_window.outer_size().map_err(|e| e.to_string())?;
    let main_inner_size = main_window.inner_size().map_err(|e| e.to_string())?;
    
    info!("🔍 DEBUG (startup): Main window - outer: {}x{} at ({}, {}), inner: {}x{}", 
          main_outer_size.width, main_outer_size.height, main_outer_position.x, main_outer_position.y,
          main_inner_size.width, main_inner_size.height);
    
    // Calculate AI response window size - SAME WIDTH as main window (startup)
    let max_height = 500u32; // Maximum height constraint
    
    // Use main window outer width for exact width match
    let ai_response_width = main_outer_size.width;
    let ai_response_height = max_height.min(500); // Default height 500px
    
    info!("🔍 DEBUG (startup): AI Response size calculated: {}x{} (SAME WIDTH as main)", ai_response_width, ai_response_height);
    
    // Calculate position: CENTER X on screen, positioned below main window (startup)
    let screen_size = monitor.size();
    let gap_between_windows = 10i32;
    let response_x = (screen_size.width as i32 - ai_response_width as i32) / 2; // CENTER on screen horizontally
    let response_y = main_outer_position.y + main_outer_size.height as i32 + gap_between_windows; // Below main window
    
    info!("🔍 DEBUG (startup): Positioning calculation:");
    info!("  - Main outer width: {}, AI width: {} (EXACT MATCH)", main_outer_size.width, ai_response_width);
    info!("  - Main window position: ({}, {}), AI window position: ({}, {}) (SAME X, BELOW Y)", main_outer_position.x, main_outer_position.y, response_x, response_y);
    info!("  - Gap between windows: {}px", gap_between_windows);
    info!("  - Final position: ({}, {})", response_x, response_y);
    
    info!("📐 AI Response Window Position (startup): main={}x{} at ({}, {}), AI={}x{} at ({}, {}) [SAME X & SAME WIDTH, BELOW]", 
          main_outer_size.width, main_outer_size.height, main_outer_position.x, main_outer_position.y,
          ai_response_width, ai_response_height, response_x, response_y);
    
    // Screen bounds check for safety (startup)
    let screen_size = monitor.size();
    info!("🔍 DEBUG (startup): Screen size: {}x{} (scale: {})", screen_size.width, screen_size.height, scale_factor);
    
    // Ensure AI response window fits on screen (positioned below main window at startup)
    if response_x + ai_response_width as i32 > screen_size.width as i32 ||
       response_y + ai_response_height as i32 > screen_size.height as i32 {
        warn!("⚠️ AI response window would go off-screen at startup when positioned below main window");
    }
    
    // Create response window configuration (hidden by default)
    // Use the same URL logic as the main create_ai_response_window function
    let window_url = if cfg!(debug_assertions) {
        // Development mode - serve from localhost with proper path
        tauri::WebviewUrl::External("http://localhost:1420/ai-response.html".parse().unwrap())
    } else {
        // Production mode - use app protocol with proper path
        tauri::WebviewUrl::App("ai-response.html".into())
    };
    
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        window_url
    )
    .title("AI Response")
    .inner_size(ai_response_width as f64, ai_response_height as f64)
    .min_inner_size(200.0, 100.0)  // Conservative minimum size (startup)
    .max_inner_size(ai_response_width as f64, max_height as f64)  // Respect max height constraint
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
            if let Err(e) = set_window_capture_protection(&window, true) {
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
    info!("⚡ FAST SHOW: AI response window...");
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        // Use concurrent operations for faster display
        match window.show() {
            Ok(_) => {
                // Ensure window is focused and visible for immediate use
                let _ = window.set_focus();
                info!("✅ FAST SHOW: AI response window shown and focused successfully");
                Ok("AI response window shown".to_string())
            }
            Err(e) => {
                error!("❌ FAST SHOW: Failed to show AI response window: {}", e);
                Err(format!("Failed to show AI response window: {}", e))
            }
        }
    } else {
        warn!("⚠️ FAST SHOW: AI response window not found - creating new one immediately");
        create_ai_response_window(app_handle)
    }
}

// Async version for non-blocking show operations
async fn show_ai_response_window_async(app_handle: AppHandle) -> Result<String, String> {
    info!("⚡ ASYNC FAST SHOW: AI response window...");
    
    if let Some(window) = app_handle.get_webview_window("ai-response") {
        match window.show() {
            Ok(_) => {
                // Ensure window is focused for immediate use
                let _ = window.set_focus();
                info!("✅ ASYNC FAST SHOW: AI response window shown and focused");
                Ok("AI response window shown".to_string())
            }
            Err(e) => {
                error!("❌ ASYNC FAST SHOW: Failed to show window: {}", e);
                Err(format!("Failed to show AI response window: {}", e))
            }
        }
    } else {
        warn!("⚠️ ASYNC FAST SHOW: Window not found - creating new one");
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
                
                // Keep window at fixed 500px height - no automatic content-based resizing
                if data.message_type == "complete" {
                    info!("🔄 RUST DEBUG: Content complete - ensuring window stays at 500px height");
                    
                    // Always maintain 500px height for consistent experience
                    let fixed_height = 500u32;
                    
                    info!("📏 RUST DEBUG: Maintaining fixed height: {}px (no content-based auto-resize)", fixed_height);
                    
                    // Only resize if current height is different from 500px
                    if let Some(window) = app_handle.get_webview_window("ai-response") {
                        if let Ok(current_size) = window.outer_size() {
                            if current_size.height != fixed_height {
                                if let Err(resize_err) = resize_ai_response_window(app_handle.clone(), fixed_height) {
                                    warn!("❌ RUST DEBUG: Failed to maintain 500px height: {}", resize_err);
                                } else {
                                    info!("✅ RUST DEBUG: Window height maintained at 500px");
                                }
                            }
                        }
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

/// NEW: Create AI response window positioned below main window with proper DPI-aware centering
#[tauri::command]
fn create_ai_response_window_enhanced_below(app_handle: AppHandle) -> Result<String, String> {
    info!("✨ Creating AI response window positioned below main (enhanced DPI-aware)...");
    
    let main_window = match app_handle.get_webview_window("main") {
        Some(window) => window,
        None => {
            error!("Main window not found");
            return Err("Main window not found".to_string());
        }
    };
    
    // Get DPI scale factor for proper scaling
    let monitor = main_window.current_monitor().map_err(|e| e.to_string())?
        .ok_or_else(|| "No monitor found".to_string())?;
    let scale_factor = monitor.scale_factor();
    
    info!("🔍 DPI Scale factor: {:.2}", scale_factor);
    
    // Get main window measurements in physical pixels
    let main_outer_position = main_window.outer_position().map_err(|e| e.to_string())?;
    let main_outer_size = main_window.outer_size().map_err(|e| e.to_string())?;
    
    info!("🔍 Main window: {}x{} at ({}, {})", 
          main_outer_size.width, main_outer_size.height, 
          main_outer_position.x, main_outer_position.y);
    
    // AI response window dimensions - SAME WIDTH as main window
    let ai_width = main_outer_size.width;
    let ai_height = 500u32; // Increased default height for better visibility
    
    // CRITICAL FIX: Position calculation for proper centering with DPI awareness
    // Calculate center X position of main window in logical coordinates first
    let main_center_x_logical = (main_outer_position.x as f64 / scale_factor) + (main_outer_size.width as f64 / scale_factor / 2.0);
    let ai_width_logical = ai_width as f64 / scale_factor;
    
    // Calculate AI window position to center it below main window (logical coordinates)
    let ai_x_logical = main_center_x_logical - (ai_width_logical / 2.0);
    // Adjust gap based on scale factor to compensate for DPI scaling issues
    let gap_adjustment = if scale_factor < 1.5 { -2.0 / scale_factor } else { 0.0 };
    let ai_y_logical = (main_outer_position.y as f64 / scale_factor) + (main_outer_size.height as f64 / scale_factor) + gap_adjustment;
    
    // Convert back to physical coordinates ONLY ONCE
    let ai_x_physical = (ai_x_logical * scale_factor) as i32;
    let ai_y_physical = (ai_y_logical * scale_factor) as i32;
    
    info!("🎯 DPI-FIXED Positioning:");
    info!("  - Scale factor: {:.2} (gap adjustment: {:.1}px)", scale_factor, gap_adjustment);
    info!("  - Main center logical: {:.1}", main_center_x_logical);
    info!("  - AI window logical: {:.1}x{:.1} at ({:.1}, {:.1})", ai_width_logical, ai_height as f64 / scale_factor, ai_x_logical, ai_y_logical);
    info!("  - AI window physical: {}x{} at ({}, {})", ai_width, ai_height, ai_x_physical, ai_y_physical);
    info!("  - Gap calculation: scale < 1.5? {}, adjustment: {:.2}px logical", scale_factor < 1.5, gap_adjustment);
    
    // Create response window configuration
    let window_url = if cfg!(debug_assertions) {
        tauri::WebviewUrl::External("http://localhost:1420/ai-response.html".parse().unwrap())
    } else {
        tauri::WebviewUrl::App("ai-response.html".into())
    };
    
    let window_config = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "ai-response",
        window_url
    )
    .title("AI Response")
    .inner_size(ai_width as f64, ai_height as f64)
    .min_inner_size(200.0, 100.0)
    .position(ai_x_physical as f64, ai_y_physical as f64)
    .resizable(true)
    .fullscreen(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)  // Disable shadow to prevent visual gaps
    .focused(true);
    
    match window_config.build() {
        Ok(window) => {
            info!("✅ AI response window created below main with DPI-aware centering");
            
            // Set window capture protection
            if let Err(e) = set_window_capture_protection(&window, true) {
                error!("Failed to set window capture protection: {}", e);
            }
            
            Ok("AI response window created below main".to_string())
        }
        Err(e) => {
            error!("Failed to create AI response window: {}", e);
            Err(format!("Failed to create AI response window: {}", e))
        }
    }
}

/// NEW: Reset AI response window size and position it below main window with DPI-aware centering
#[tauri::command]
fn reset_ai_response_window_enhanced_below_size(app_handle: AppHandle) -> Result<String, String> {
    info!("🔄 Resetting AI response window below main with DPI-aware centering...");
    
    if let Some(ai_window) = app_handle.get_webview_window("ai-response") {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            // Get DPI scale factor
            let monitor = main_window.current_monitor().map_err(|e| e.to_string())?
                .ok_or_else(|| "No monitor found".to_string())?;
            let scale_factor = monitor.scale_factor();
            
            // Get main window measurements
            let main_outer_position = main_window.outer_position().map_err(|e| e.to_string())?;
            let main_outer_size = main_window.outer_size().map_err(|e| e.to_string())?;
            
            // AI response window dimensions - SAME WIDTH as main window
            let ai_width = main_outer_size.width;
            let ai_height = 500u32; // Reset to default height of 500px
            
            // CRITICAL FIX: Position calculation for proper centering with DPI awareness
            // Calculate center X position of main window in logical coordinates first
            let main_center_x_logical = (main_outer_position.x as f64 / scale_factor) + (main_outer_size.width as f64 / scale_factor / 2.0);
            let ai_width_logical = ai_width as f64 / scale_factor;
            
            // Calculate AI window position to center it below main window (logical coordinates)
            let ai_x_logical = main_center_x_logical - (ai_width_logical / 2.0);
            // Adjust gap based on scale factor to compensate for DPI scaling issues
            let gap_adjustment = if scale_factor < 1.5 { -3.0 / scale_factor } else { -1.0 };
            let ai_y_logical = (main_outer_position.y as f64 / scale_factor) + (main_outer_size.height as f64 / scale_factor) + gap_adjustment;
            
            // Convert back to physical coordinates ONLY ONCE
            let ai_x_physical = (ai_x_logical * scale_factor) as i32;
            let ai_y_physical = (ai_y_logical * scale_factor) as i32;
            
            info!("🔄 DPI-FIXED Reset Positioning:");
            info!("  - Scale factor: {:.2} (gap adjustment: {:.1}px)", scale_factor, gap_adjustment);
            info!("  - Main center logical: {:.1}", main_center_x_logical);
            info!("  - AI reset physical: {}x{} at ({}, {})", ai_width, ai_height, ai_x_physical, ai_y_physical);
            info!("  - Gap calculation: scale < 1.5? {}, adjustment: {:.2}px logical", scale_factor < 1.5, gap_adjustment);
            
            // Apply size and position in one operation
            match ai_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: ai_width,
                height: ai_height,
            })) {
                Ok(_) => {
                    match ai_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: ai_x_physical,
                        y: ai_y_physical,
                    })) {
                        Ok(_) => {
                            info!("✅ AI response window reset below main with DPI-aware centering");
                            Ok(format!("AI window reset: {}x{} at ({}, {})", ai_width, ai_height, ai_x_physical, ai_y_physical))
                        }
                        Err(e) => {
                            error!("Failed to reset AI window position: {}", e);
                            Err(format!("Failed to reset position: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to reset AI window size: {}", e);
                    Err(format!("Failed to reset size: {}", e))
                }
            }
        } else {
            error!("Main window not found for AI window reset");
            Err("Main window not found".to_string())
        }
    } else {
        warn!("AI response window not found for reset - creating new one");
        create_ai_response_window_enhanced_below(app_handle)
    }
}

#[tauri::command]
fn reset_ai_response_window_size(app_handle: AppHandle) -> Result<String, String> {
    info!("🔄 Resetting AI response window to match main window width...");
    
    if let Some(ai_window) = app_handle.get_webview_window("ai-response") {
        if let Some(main_window) = app_handle.get_webview_window("main") {
            // Get main window size for width matching
            let main_size = main_window.outer_size().map_err(|e| {
                error!("❌ Failed to get main window size: {}", e);
                e.to_string()
            })?;
            
            // Get monitor info for proper sizing
            let monitor = ai_window.current_monitor().map_err(|e| e.to_string())?
                .ok_or_else(|| "No monitor found".to_string())?;
            let max_height = monitor.size();
            let default_height = (max_height.height as f64 * 0.4) as u32; // 40% of screen height
            
            // Calculate centered AI response window size (80% of main window width)
            let main_inner_size = main_window.inner_size().map_err(|e| e.to_string())?;
            let reset_width = (main_inner_size.width as f64 * 0.8) as u32; // Maintain 80% for centering
            let reset_height = default_height.min(500).max(200); // Between 200-500px
            
            info!("📐 Resetting AI response window: main_width={}px, reset_height={}px", reset_width, reset_height);
            
            match ai_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: reset_width,
                height: reset_height,
            })) {
                Ok(_) => {
                    info!("✅ AI response window reset to {}x{}", reset_width, reset_height);
                    Ok(format!("AI response window reset to {}x{}", reset_width, reset_height))
                }
                Err(e) => {
                    error!("❌ Failed to reset AI response window size: {}", e);
                    Err(format!("Failed to reset window size: {}", e))
                }
            }
        } else {
            error!("❌ Main window not found for width reference");
            Err("Main window not found".to_string())
        }
    } else {
        error!("❌ AI response window not found for reset");
        Err("AI response window not found".to_string())
    }
}

#[tauri::command]
fn set_window_capture_protection(window: &tauri::WebviewWindow, protect: bool) -> Result<(), String> {
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

// Diagnostic command for database connectivity
#[derive(Serialize, Deserialize)]
struct DatabaseDiagnostic {
    database_connected: bool,
    connection_error: Option<String>,
    tables_exist: bool,
    sample_data_count: Option<i64>,
    test_query_result: Option<String>,
}

#[tauri::command]
async fn diagnose_database() -> Result<DatabaseDiagnostic, String> {
    info!("🔍 Running database diagnostics");
    
    let mut diagnostic = DatabaseDiagnostic {
        database_connected: false,
        connection_error: None,
        tables_exist: false,
        sample_data_count: None,
        test_query_result: None,
    };
    
    // Test database initialization
    match crate::database::initialize_database().await {
        Ok(()) => {
            diagnostic.database_connected = true;
            
            // Test if tables exist by trying to query sessions table
            match crate::database::shared::DATABASE_POOL.get().await {
                Ok(client) => {
                    // Check if sessions table exists and get count
                    match client.query_one("SELECT COUNT(*) as count FROM sessions", &[]).await {
                        Ok(row) => {
                            diagnostic.tables_exist = true;
                            let count: i64 = row.get("count");
                            diagnostic.sample_data_count = Some(count);
                            diagnostic.test_query_result = Some(format!("Found {} sessions in database", count));
                            info!("✅ Database diagnostic passed: {} sessions found", count);
                        }
                        Err(e) => {
                            diagnostic.test_query_result = Some(format!("Table query failed: {}", e));
                            warn!("⚠️ Sessions table query failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    diagnostic.connection_error = Some(format!("Pool connection failed: {}", e));
                    warn!("⚠️ Database pool connection failed: {}", e);
                }
            }
        }
        Err(e) => {
            diagnostic.connection_error = Some(e.clone());
            warn!("⚠️ Database initialization failed: {}", e);
        }
    }
    
    Ok(diagnostic)
}

#[tauri::command]
async fn test_session_query(session_id: String) -> Result<String, String> {
    info!("🧪 Testing session query for ID: {}", session_id);
    
    // First run diagnostic
    let diagnostic = diagnose_database().await?;
    if !diagnostic.database_connected {
        return Err(format!("Database not connected: {}", diagnostic.connection_error.unwrap_or("Unknown error".to_string())));
    }
    
    // Try to get session info
    match crate::database::get_session_with_user_info(&session_id).await {
        Ok(session_info) => {
            Ok(format!("✅ Session found: {} (User: {}, Credits: {})", 
                session_info.session_name, 
                session_info.user_details.name,
                session_info.credits_available
            ))
        }
        Err(e) => {
            Err(format!("❌ Session query failed: {}", e))
        }
    }
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

// ===== NEW WINDOW MANAGEMENT COMMANDS FOR HOTKEYS =====

#[derive(Serialize, Deserialize)]
struct MoveWindowPayload {
    #[serde(rename = "deltaX")]
    delta_x: i32,
    #[serde(rename = "deltaY")]
    delta_y: i32,
}

#[derive(Serialize, Deserialize)]
struct ResizeWindowPayload {
    #[serde(rename = "scaleFactor")]
    scale_factor: f64,
}

#[tauri::command]
fn move_window_relative(app_handle: AppHandle, delta_x: i32, delta_y: i32) -> Result<String, String> {
    info!("📍 Moving main window by: ({}, {})", delta_x, delta_y);
    
    if let Some(window) = app_handle.get_webview_window("main") {
        // Get current position
        let current_position = window.outer_position().map_err(|e| {
            error!("❌ Failed to get current window position: {}", e);
            e.to_string()
        })?;
        
        // Calculate new position
        let new_x = current_position.x + delta_x;
        let new_y = current_position.y + delta_y;
        
        info!("📊 Window move: current=({}, {}), delta=({}, {}), new=({}, {})", 
              current_position.x, current_position.y, delta_x, delta_y, new_x, new_y);
        
        match window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x,
            y: new_y,
        })) {
            Ok(_) => {
                info!("✅ Main window successfully moved to: ({}, {})", new_x, new_y);
                
                // Verify the move worked
                match window.outer_position() {
                    Ok(new_position) => {
                        info!("🔍 Post-move verification: actual new position is ({}, {})", new_position.x, new_position.y);
                        Ok(format!("Window moved to ({}, {})", new_position.x, new_position.y))
                    }
                    Err(e) => {
                        warn!("❌ Failed to verify window move: {}", e);
                        Ok(format!("Window move attempted to ({}, {})", new_x, new_y))
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to move main window: {}", e);
                Err(format!("Failed to move main window: {}", e))
            }
        }
    } else {
        error!("❌ Main window not found for move");
        Err("Main window not found".to_string())
    }
}
#[tauri::command]
fn resize_window_scale(app_handle: AppHandle, width: u32, height: u32) -> Result<String, String> {
    info!("📏 Resizing main window to responsive size: {}x{}", width, height);
    
    if let Some(window) = app_handle.get_webview_window("main") {
        // Get current size for comparison
        let current_size = window.outer_size().map_err(|e| {
            error!("❌ Failed to get current window size: {}", e);
            e.to_string()
        })?;
        
        // Get monitor info for DPI-aware scaling
        let monitor = window.current_monitor().map_err(|e| e.to_string())?
            .ok_or_else(|| "No monitor found".to_string())?;
        let scale_factor = monitor.scale_factor();
        let monitor_size = monitor.size();
        
        // Apply DPI scaling to the requested dimensions
        let dpi_adjusted_width = ((width as f64) * scale_factor) as u32;
        let dpi_adjusted_height = ((height as f64) * scale_factor) as u32;
        
        // Ensure minimum and maximum constraints based on monitor size
        let min_width = 400;
        let min_height = 200;
        let max_width = ((monitor_size.width as f64) * 0.9) as u32;
        let max_height = ((monitor_size.height as f64) * 0.9) as u32;
        
        let final_width = dpi_adjusted_width.max(min_width).min(max_width);
        let final_height = dpi_adjusted_height.max(min_height).min(max_height);
        
        info!("📊 Responsive resize: current={}x{}, requested={}x{}, DPI_scale={:.2}, DPI_adjusted={}x{}, final={}x{}", 
              current_size.width, current_size.height, width, height, scale_factor,
              dpi_adjusted_width, dpi_adjusted_height, final_width, final_height);
        
        match window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: final_width,
            height: final_height,
        })) {
            Ok(_) => {
                info!("✅ Main window successfully resized to responsive size: {}x{}", final_width, final_height);
                
                // Verify the resize worked
                match window.outer_size() {
                    Ok(new_size) => {
                        info!("🔍 Post-resize verification: actual new size is {}x{}", new_size.width, new_size.height);
                        Ok(format!("Window resized to {}x{} (requested: {}x{})", new_size.width, new_size.height, width, height))
                    }
                    Err(e) => {
                        warn!("❌ Failed to verify window resize: {}", e);
                        Ok(format!("Window resize attempted: {}x{}", final_width, final_height))
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

#[tauri::command]
fn show_main_window(app_handle: AppHandle) -> Result<String, String> {
    info!("👁️ Showing main window...");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        match window.show() {
            Ok(_) => {
                info!("✅ Main window shown successfully");
                // Also bring to front
                if let Err(e) = window.set_focus() {
                    warn!("⚠️ Failed to focus main window: {}", e);
                }
                Ok("Main window shown".to_string())
            }
            Err(e) => {
                error!("❌ Failed to show main window: {}", e);
                Err(format!("Failed to show main window: {}", e))
            }
        }
    } else {
        error!("❌ Main window not found for show");
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
fn hide_main_window(app_handle: AppHandle) -> Result<String, String> {
    info!("🙈 Hiding main window...");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        match window.hide() {
            Ok(_) => {
                info!("✅ Main window hidden successfully");
                Ok("Main window hidden".to_string())
            }
            Err(e) => {
                error!("❌ Failed to hide main window: {}", e);
                Err(format!("Failed to hide main window: {}", e))
            }
        }
    } else {
        error!("❌ Main window not found for hide");
        Err("Main window not found".to_string())
    }
}

// Screenshot and AI Analysis Commands

#[derive(Serialize, Deserialize)]
struct ScreenshotResponse {
    screenshot: String, // Base64 encoded image
    width: u32,
    height: u32,
}

// Screenshot commands removed - using accessibility-based analysis instead

#[derive(Serialize, Deserialize)]
struct AnalyzeScreenWithAiPayload {
    model: String,
    provider: String, // "openai" or "pollinations"
    company: Option<String>,
    position: Option<String>,
    job_description: Option<String>,
    system_prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AiAnalysisResult {
    generated_question: String,
    analysis: String,
    confidence: f32,
}

#[tauri::command]
async fn analyze_screen_with_ai(
    payload: AnalyzeScreenWithAiPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    info!("[SCREEN_ANALYSIS] Using accessibility-based analysis as fallback for non-streaming analysis.");
    
    // Use the accessibility-based analysis for non-streaming version
    match analyze_applications_with_ai_streaming(payload, state, app_handle).await {
        Ok(result) => Ok(format!("Generated question: {}", result.generated_question)),
        Err(e) => Err(e),
    }
}

// New streaming version of screen analysis that shows progress in real-time
#[tauri::command]
async fn analyze_screen_with_ai_streaming(
    payload: AnalyzeScreenWithAiPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<AiAnalysisResult, String> {
    info!("[SCREEN_ANALYSIS_STREAMING] Starting screen capture and AI analysis with streaming...");
    
    // Show the AI response window before starting
    if let Err(e) = show_ai_response_window(app_handle.clone()) {
        warn!("Failed to show AI response window: {}", e);
    }
    
    // Use accessibility-based analysis instead of screenshot capture
    info!("[SCREEN_ANALYSIS_STREAMING] Delegating to accessibility-based analysis...");
    
    // Send initial status to UI
    let status_data = AiResponseData {
        message_type: "stream-token".to_string(),
        text: Some("[ANALYSIS] Starting accessibility-based screen analysis...".to_string()),
        error: None,
    };
    let _ = send_ai_response_data(app_handle.clone(), status_data).await;
    
    // Delegate to accessibility-based analysis
    analyze_applications_with_ai_streaming(payload, state, app_handle).await
}

// Accessibility-based AI Analysis Commands (Primary Solution)

/// Analyze all target applications using Windows Accessibility API with AI streaming
#[tauri::command]
async fn analyze_applications_with_ai_streaming(
    payload: AnalyzeScreenWithAiPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<AiAnalysisResult, String> {
    info!("[ACCESSIBILITY_STREAMING] Starting accessibility-based analysis of all target applications...");
    
    // Show the AI response window before starting
    if let Err(e) = show_ai_response_window(app_handle.clone()) {
        warn!("Failed to show AI response window: {}", e);
    }
    
    // Send initial status to UI
    let status_data = AiResponseData {
        message_type: "stream-token".to_string(),
        text: Some("[ACCESSIBILITY] Starting real-time text reading from applications...".to_string()),
        error: None,
    };
    let _ = send_ai_response_data(app_handle.clone(), status_data).await;
    
    // Read text from all target applications using Accessibility API
    let accessibility_results = match accessibility_reader::read_text_from_applications().await {
        Ok(results) => results,
        Err(e) => {
            error!("❌ [ACCESSIBILITY] Failed to read text from applications: {}", e);
            
            // Send error to UI
            let error_data = AiResponseData {
                message_type: "error".to_string(),
                text: None,
                error: Some(format!("Failed to read text from applications: {}", e)),
            };
            let _ = send_ai_response_data(app_handle, error_data).await;
            
            return Err(format!("Failed to read text from applications: {}", e));
        }
    };
    
    info!("📊 [ACCESSIBILITY] Found {} text blocks from target applications", accessibility_results.len());
    
    // Check if we found any meaningful text
    if accessibility_results.is_empty() {
        warn!("⚠️ [ACCESSIBILITY] No text found in target applications");
        
        let no_text_data = AiResponseData {
            message_type: "stream-token".to_string(),
            text: Some("\n[ACCESSIBILITY] No readable text found in target applications (Teams, Zoom, Chrome, etc.). Make sure the applications are open and contain visible text.".to_string()),
            error: None,
        };
        let _ = send_ai_response_data(app_handle.clone(), no_text_data).await;
        
        // Return a default result
        let default_result = AiAnalysisResult {
            generated_question: "Can you describe your experience with video conferencing tools and remote collaboration?".to_string(),
            analysis: "No text was found in target applications, so this is a general question about remote work experience.".to_string(),
            confidence: 0.3,
        };
        
        // Send completion
        let completion_data = AiResponseData {
            message_type: "complete".to_string(),
            text: Some(format!("\n[QUESTION] Generated Question (No accessibility text found):\n\n🎯 {}\n\n[ANALYSIS] {}\n\n[CONFIDENCE] {:.0}%", 
                default_result.generated_question, default_result.analysis, default_result.confidence * 100.0)),
            error: None,
        };
        let _ = send_ai_response_data(app_handle.clone(), completion_data).await;
        
        return Ok(default_result);
    }
    
    // Find the most relevant text (prioritize questions and meaningful content)
    let best_text = find_best_accessibility_text(&accessibility_results);
    
    info!("✅ [ACCESSIBILITY] Selected best text from {}: '{}'", 
          best_text.source_app, 
          best_text.text.chars().take(100).collect::<String>());
    
    // Send accessibility results to UI
    let accessibility_status_data = AiResponseData {
        message_type: "stream-token".to_string(),
        text: Some(format!(
            "\n[ACCESSIBILITY] ✅ Text reading complete!\n📊 Found {} text blocks from target applications\n🎯 Best source: {} ({}% confidence)\n📝 Selected text: {}...\n\n[AI] Now generating interview question based on extracted text...", 
            accessibility_results.len(),
            best_text.source_app,
            (best_text.confidence * 100.0) as u32,
            best_text.text.chars().take(80).collect::<String>()
        )),
        error: None,
    };
    let _ = send_ai_response_data(app_handle.clone(), accessibility_status_data).await;
    
    // Generate AI analysis using the extracted text
    generate_ai_analysis_from_text(
        &best_text.text,
        &format!("Windows Accessibility API from {}", best_text.source_app),
        payload,
        state,
        app_handle,
    ).await
}

/// Analyze focused window using Windows Accessibility API with AI streaming
#[tauri::command]
async fn analyze_focused_window_with_ai_streaming(
    payload: AnalyzeScreenWithAiPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<AiAnalysisResult, String> {
    info!("[ACCESSIBILITY_STREAMING] Starting accessibility-based analysis of focused window...");
    
    // Show the AI response window before starting
    if let Err(e) = show_ai_response_window(app_handle.clone()) {
        warn!("Failed to show AI response window: {}", e);
    }
    
    // Send initial status to UI
    let status_data = AiResponseData {
        message_type: "stream-token".to_string(),
        text: Some("[ACCESSIBILITY] Reading text from currently focused window...".to_string()),
        error: None,
    };
    let _ = send_ai_response_data(app_handle.clone(), status_data).await;
    
    // Read text from focused window using Accessibility API
    let accessibility_result = match accessibility_reader::read_text_from_focused_window().await {
        Ok(Some(result)) => result,
        Ok(None) => {
            warn!("⚠️ [ACCESSIBILITY] No text found in focused window");
            
            let no_text_data = AiResponseData {
                message_type: "stream-token".to_string(),
                text: Some("\n[ACCESSIBILITY] No readable text found in the currently focused window. Try focusing on an application window with visible text content.".to_string()),
                error: None,
            };
            let _ = send_ai_response_data(app_handle.clone(), no_text_data).await;
            
            // Return a default result
            let default_result = AiAnalysisResult {
                generated_question: "Can you describe what you're currently working on and the technologies involved?".to_string(),
                analysis: "No text was found in the focused window, so this is a general question about current work.".to_string(),
                confidence: 0.3,
            };
            
            // Send completion
            let completion_data = AiResponseData {
                message_type: "complete".to_string(),
                text: Some(format!("\n[QUESTION] Generated Question (No focused window text):\n\n🎯 {}\n\n[ANALYSIS] {}\n\n[CONFIDENCE] {:.0}%", 
                    default_result.generated_question, default_result.analysis, default_result.confidence * 100.0)),
                error: None,
            };
            let _ = send_ai_response_data(app_handle.clone(), completion_data).await;
            
            return Ok(default_result);
        },
        Err(e) => {
            error!("❌ [ACCESSIBILITY] Failed to read text from focused window: {}", e);
            
            // Send error to UI
            let error_data = AiResponseData {
                message_type: "error".to_string(),
                text: None,
                error: Some(format!("Failed to read text from focused window: {}", e)),
            };
            let _ = send_ai_response_data(app_handle, error_data).await;
            
            return Err(format!("Failed to read text from focused window: {}", e));
        }
    };
    
    info!("✅ [ACCESSIBILITY] Text extracted from focused window {}: '{}'", 
          accessibility_result.source_app, 
          accessibility_result.text.chars().take(100).collect::<String>());
    
    // Send accessibility results to UI
    let accessibility_status_data = AiResponseData {
        message_type: "stream-token".to_string(),
        text: Some(format!(
            "\n[ACCESSIBILITY] ✅ Text reading complete!\n🎯 Source: {} ({}% confidence)\n📝 Extracted text: {}...\n\n[AI] Now generating interview question based on extracted text...", 
            accessibility_result.source_app,
            (accessibility_result.confidence * 100.0) as u32,
            accessibility_result.text.chars().take(80).collect::<String>()
        )),
        error: None,
    };
    let _ = send_ai_response_data(app_handle.clone(), accessibility_status_data).await;
    
    // Generate AI analysis using the extracted text
    generate_ai_analysis_from_text(
        &accessibility_result.text,
        &format!("Windows Accessibility API from focused window: {}", accessibility_result.source_app),
        payload,
        state,
        app_handle,
    ).await
}

/// Helper function to find the best accessibility text result
fn find_best_accessibility_text(results: &[accessibility_reader::AccessibilityTextResult]) -> &accessibility_reader::AccessibilityTextResult {
    // Prioritize results that look like questions
    if let Some(question_result) = results.iter().find(|r| r.is_potential_question) {
        info!("📝 [ACCESSIBILITY] Found potential question text from {}", question_result.source_app);
        return question_result;
    }
    
    // Prioritize results with higher confidence
    if let Some(high_confidence_result) = results.iter().max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal)) {
        info!("📊 [ACCESSIBILITY] Using highest confidence text from {}", high_confidence_result.source_app);
        return high_confidence_result;
    }
    
    // Fallback to first result
    &results[0]
}

/// Helper function to generate AI analysis from extracted text
async fn generate_ai_analysis_from_text(
    extracted_text: &str,
    source_description: &str,
    payload: AnalyzeScreenWithAiPayload,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<AiAnalysisResult, String> {
    // Determine AI provider
    let provider = AIProvider::from_str(&payload.provider)
        .unwrap_or(AIProvider::OpenAI);
    
    let mut context = {
        let context_guard = state.interview_context.lock();
        context_guard.clone()
    };
    
    // Update context with payload data
    if let Some(company) = payload.company {
        context.company = Some(company);
    }
    if let Some(position) = payload.position {
        context.position = Some(position);
    }
    if let Some(job_description) = payload.job_description {
        context.job_description = Some(job_description);
    }
    
    // Build AI prompt using extracted text
    let system_prompt = format!(
        "You are an expert technical interviewer. I have extracted the following text from an application using {}:\n\n---\n{}\n---\n\nBased on this extracted text, generate a specific interview question that tests understanding of the visible content. The question should be relevant to the context and help assess the candidate's technical knowledge or experience.",
        source_description,
        extracted_text
    );
    
    let analysis_prompt = format!(
        "{}\n\nYour response MUST be valid JSON in exactly this format:\n{{\n  \"generated_question\": \"Your specific interview question based on the extracted text\",\n  \"analysis\": \"Brief explanation of why this question tests understanding of the extracted content\",\n  \"confidence\": 0.85\n}}\n\nOnly return the JSON object, nothing else.",
        system_prompt
    );
    
    // Initialize streaming state and timing
    let stream_start_time = std::time::Instant::now();
    info!("[AI_STREAMING] Starting AI analysis with extracted accessibility text...");
    let _ = app_handle.emit("ai-stream-start", ());
    
    // Stream AI analysis
    let app_handle_clone = app_handle.clone();
    let result = match provider {
        AIProvider::OpenAI => {
            info!("[AI_PROVIDER] Using OpenAI for accessibility-based analysis");
            state.ensure_openai_client()?;
            
            let client = {
                let client_guard = state.openai_client.lock();
                client_guard.as_ref().unwrap().clone()
            };
            
            let model = openai::OpenAIModel::from_string(&payload.model)
                .map_err(|e| format!("Invalid OpenAI model: {}", e))?;
            
            // For OpenAI, simulate streaming with status updates
            let status_update = AiResponseData {
                message_type: "stream-token".to_string(),
                text: Some("\n[AI] Sending extracted text to OpenAI for analysis...".to_string()),
                error: None,
            };
            let _ = send_ai_response_data(app_handle_clone.clone(), status_update).await;
            
            // Use text-based analysis since we have accessibility text
            client.generate_answer(&analysis_prompt, &context, model)
                .await
                .map_err(|e| e.to_string())
        },
        AIProvider::Pollinations => {
            info!("[AI_PROVIDER] Attempting Pollinations for streaming analysis with model: {}", payload.model);
            
            // Status update for Pollinations attempt
            let status_update = AiResponseData {
                message_type: "stream-token".to_string(),
                text: Some("\n[AI] Attempting Pollinations AI for streaming analysis...".to_string()),
                error: None,
            };
            let _ = send_ai_response_data(app_handle_clone.clone(), status_update).await;
            
            // Try Pollinations first, but fallback to OpenAI if it fails
            match state.ensure_pollinations_client() {
                Ok(()) => {
                    let client = {
                        let client_guard = state.pollinations_client.lock();
                        client_guard.as_ref().unwrap().clone()
                    };
                    
                    let model = pollinations::PollinationsModel::from_string(&payload.model)
                        .map_err(|e| format!("Invalid Pollinations model: {}", e))?;
                    
                    // Try Pollinations streaming
                    let app_handle_for_streaming = app_handle_clone.clone();
                    match client.generate_answer_streaming(
                        &analysis_prompt, 
                        &context, 
                        model,
                        move |token: &str| {
                            info!("[STREAM_TOKEN] Received Pollinations token: '{}'", token.chars().take(50).collect::<String>());
                            
                            let data = AiResponseData {
                                message_type: "stream-token".to_string(),
                                text: Some(token.to_string()),
                                error: None,
                            };
                            let app_handle_for_token = app_handle_for_streaming.clone();
                            tokio::spawn(async move {
                                if let Err(e) = send_ai_response_data(app_handle_for_token, data).await {
                                    error!("Failed to send streaming token to UI: {}", e);
                                }
                            });
                        }
                    ).await {
                        Ok(result) => {
                            info!("[SUCCESS] Pollinations streaming completed successfully");
                            Ok(result)
                        },
                        Err(pollinations_error) => {
                            warn!("[FALLBACK] Pollinations failed: {}, falling back to OpenAI", pollinations_error);
                            
                            // Send fallback status
                            let fallback_status = AiResponseData {
                                message_type: "stream-token".to_string(),
                                text: Some("\n[FALLBACK] Pollinations failed, switching to OpenAI...".to_string()),
                                error: None,
                            };
                            let _ = send_ai_response_data(app_handle_clone.clone(), fallback_status).await;
                            
                            // Fallback to OpenAI
                            state.ensure_openai_client()?;
                            let openai_client = {
                                let client_guard = state.openai_client.lock();
                                client_guard.as_ref().unwrap().clone()
                            };
                            
                            let openai_model = openai::OpenAIModel::GPT4Turbo; // Use a reliable model
                            openai_client.generate_answer(&analysis_prompt, &context, openai_model)
                                .await
                                .map_err(|e| format!("OpenAI fallback also failed: {}", e))
                        }
                    }
                },
                Err(client_error) => {
                    warn!("[FALLBACK] Pollinations client setup failed: {}, using OpenAI", client_error);
                    
                    // Send fallback status
                    let fallback_status = AiResponseData {
                        message_type: "stream-token".to_string(),
                        text: Some("\n[FALLBACK] Using OpenAI instead of Pollinations...".to_string()),
                        error: None,
                    };
                    let _ = send_ai_response_data(app_handle_clone.clone(), fallback_status).await;
                    
                    // Fallback to OpenAI
                    state.ensure_openai_client()?;
                    let openai_client = {
                        let client_guard = state.openai_client.lock();
                        client_guard.as_ref().unwrap().clone()
                    };
                    
                    let openai_model = openai::OpenAIModel::GPT4Turbo;
                    openai_client.generate_answer(&analysis_prompt, &context, openai_model)
                        .await
                        .map_err(|e| format!("OpenAI fallback failed: {}", e))
                }
            }
        }
    }?;
    
    let elapsed_time = stream_start_time.elapsed();
    info!("[SUCCESS] Accessibility-based AI analysis completed. Response length: {}, elapsed time: {:.2?}", result.len(), elapsed_time);
    
    // Try to parse the AI response as JSON
    let analysis_result = match serde_json::from_str::<AiAnalysisResult>(&result) {
        Ok(parsed) => {
            info!("[SUCCESS] Successfully parsed JSON response from AI");
            parsed
        },
        Err(parse_error) => {
            info!("[WARNING] Failed to parse AI response as JSON: {}", parse_error);
            info!("[DEBUG] Raw AI response: {}", result.chars().take(200).collect::<String>());
            
            // Try to extract question from the text response
            let extracted_question = extract_question_from_text(&result);
            let extracted_analysis = extract_analysis_from_text(&result);
            let extracted_confidence = extract_confidence_from_text(&result);
            
            AiAnalysisResult {
                generated_question: extracted_question,
                analysis: extracted_analysis,
                confidence: extracted_confidence,
            }
        }
    };
    
    // Send final formatted result to UI
    let formatted_response = format!(
        "\n[QUESTION] Generated Interview Question (Accessibility API):\n\n🎯 {}\n\n[ANALYSIS] Analysis:\n\n📋 {}\n\n[CONFIDENCE] Confidence: {:.0}%\n\n[SOURCE] Text extracted using {}\n\nGenerated using {} {} with Windows Accessibility API",
        analysis_result.generated_question,
        analysis_result.analysis,
        analysis_result.confidence * 100.0,
        source_description,
        payload.provider.to_uppercase(),
        payload.model
    );
    
    let completion_data = AiResponseData {
        message_type: "complete".to_string(),
        text: Some(formatted_response),
        error: None,
    };
    let _ = send_ai_response_data(app_handle.clone(), completion_data).await;
    
    // Emit completion event
    let _ = app_handle.emit("ai-stream-complete", &analysis_result);
    
    info!("[SUCCESS] Accessibility-based AI analysis completed: {}", analysis_result.generated_question);
    
    // Store the generated question
    let question_text = &analysis_result.generated_question;
    if !question_text.is_empty() {
        let storage_payload = serde_json::json!({
            "questionText": question_text,
            "questionNumber": 1,
            "category": "ai_generated",
            "difficultyLevel": "medium",
            "source": "accessibility_api_analysis",
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "aiProvider": payload.provider,
                "aiModel": payload.model,
                "confidence": analysis_result.confidence,
                "analysisType": "accessibility_api",
                "expectedDuration": 5,
                "sourceDescription": source_description,
                "extractedTextLength": extracted_text.len()
            }
        });
        
        if let Err(e) = app_handle.emit("store-ai-question", storage_payload) {
            error!("[ERROR] Failed to emit store-ai-question event: {}", e);
        } else {
            info!("[SUCCESS] Accessibility-based AI question storage event emitted");
        }
    }
    
    Ok(analysis_result)
}

// Helper function to extract a question from unstructured AI text
fn extract_question_from_text(text: &str) -> String {
    info!("[EXTRACT] Attempting to extract question from text: '{}'", text.chars().take(100).collect::<String>());
    
    if text.trim().is_empty() {
        warn!("[EXTRACT] Empty text provided for question extraction");
        return "Can you explain what you see in this technical context?".to_string();
    }
    
    // Look for question patterns in the text
    let lines: Vec<&str> = text.lines().collect();
    
    for line in &lines {
        let trimmed = line.trim();
        // Look for lines that end with a question mark or start with question indicators
        if trimmed.ends_with('?') || 
           trimmed.to_lowercase().starts_with("question:") ||
           trimmed.to_lowercase().starts_with("q:") {
            let question = trimmed
                .strip_prefix("Question:")
                .or_else(|| trimmed.strip_prefix("question:"))
                .or_else(|| trimmed.strip_prefix("Q:"))
                .or_else(|| trimmed.strip_prefix("q:"))
                .unwrap_or(trimmed)
                .trim();
            if !question.is_empty() {
                info!("[EXTRACT] Found question: '{}'", question);
                return question.to_string();
            }
        }
    }
    
    // If no specific question found, look for the first sentence that might be a question
    for line in &lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && (trimmed.contains("?") || trimmed.len() > 20) {
            info!("[EXTRACT] Using first meaningful line as question: '{}'", trimmed);
            return trimmed.to_string();
        }
    }
    
    // Try to find any meaningful content
    let first_meaningful = lines.into_iter()
        .find(|line| !line.trim().is_empty() && line.trim().len() > 5)
        .map(|line| line.trim().to_string());
        
    if let Some(content) = first_meaningful {
        info!("[EXTRACT] Using first meaningful content as fallback: '{}'", content);
        // Convert to question format
        if content.ends_with('.') {
            format!("Can you explain: {}?", content.trim_end_matches('.'))
        } else {
            format!("Can you explain {}?", content)
        }
    } else {
        warn!("[EXTRACT] No meaningful content found, using default question");
        "Can you explain what you see in this technical context?".to_string()
    }
}

// Helper function to extract analysis from unstructured AI text
fn extract_analysis_from_text(text: &str) -> String {
    info!("[EXTRACT] Attempting to extract analysis from text: '{}'", text.chars().take(100).collect::<String>());
    
    if text.trim().is_empty() {
        warn!("[EXTRACT] Empty text provided for analysis extraction");
        return "AI analysis based on screen content".to_string();
    }
    
    let lines: Vec<&str> = text.lines().collect();
    
    // Look for analysis indicators
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.to_lowercase().starts_with("analysis:") ||
           trimmed.to_lowercase().starts_with("explanation:") ||
           trimmed.to_lowercase().starts_with("rationale:") {
            let analysis = trimmed
                .strip_prefix("Analysis:")
                .or_else(|| trimmed.strip_prefix("analysis:"))
                .or_else(|| trimmed.strip_prefix("Explanation:"))
                .or_else(|| trimmed.strip_prefix("explanation:"))
                .or_else(|| trimmed.strip_prefix("Rationale:"))
                .or_else(|| trimmed.strip_prefix("rationale:"))
                .unwrap_or(trimmed)
                .trim();
            if !analysis.is_empty() {
                info!("[EXTRACT] Found analysis: '{}'", analysis);
                return analysis.to_string();
            }
        }
    }
    
    // If no specific analysis found, try to find meaningful content
    let meaningful_lines: Vec<&str> = lines.iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && line.len() > 10)
        .take(5)
        .collect();
    
    if !meaningful_lines.is_empty() {
        let combined_text = meaningful_lines.join(" ");
        // Limit to reasonable length
        let analysis_text = if combined_text.len() > 200 {
            format!("{}...", combined_text.chars().take(200).collect::<String>())
        } else {
            combined_text
        };
        
        info!("[EXTRACT] Using meaningful content as analysis: '{}'", analysis_text.chars().take(50).collect::<String>());
        return analysis_text;
    }
    
    // Final fallback using first 20 words
    let text_words: Vec<&str> = text.split_whitespace().take(20).collect();
    if !text_words.is_empty() {
        let fallback_analysis = text_words.join(" ");
        info!("[EXTRACT] Using first 20 words as fallback analysis: '{}'", fallback_analysis.chars().take(50).collect::<String>());
        fallback_analysis
    } else {
        warn!("[EXTRACT] No meaningful content found, using default analysis");
        "AI analysis based on screen content".to_string()
    }
}

// Helper function to extract confidence from unstructured AI text
fn extract_confidence_from_text(text: &str) -> f32 {
    info!("[EXTRACT] Attempting to extract confidence from text: '{}'", text.chars().take(100).collect::<String>());
    
    if text.trim().is_empty() {
        warn!("[EXTRACT] Empty text provided for confidence extraction");
        return 0.75;
    }
    
    // Look for confidence indicators in the text
    let lower_text = text.to_lowercase();
    
    // Try to find numerical confidence values
    if let Some(start) = lower_text.find("confidence:") {
        let after_confidence = &text[start + 11..];
        info!("[EXTRACT] Found confidence indicator, parsing: '{}'", after_confidence.chars().take(20).collect::<String>());
        
        if let Some(number_match) = regex::Regex::new(r"(\d+\.?\d*)")
            .unwrap()
            .find(after_confidence) {
            if let Ok(conf) = number_match.as_str().parse::<f32>() {
                let normalized_conf = if conf > 1.0 { conf / 100.0 } else { conf };
                info!("[EXTRACT] Found numeric confidence: {} -> {}", conf, normalized_conf);
                return normalized_conf;
            }
        }
    }
    
    // Check for qualitative confidence indicators
    if lower_text.contains("high confidence") || lower_text.contains("very confident") {
        info!("[EXTRACT] Found high confidence indicator");
        return 0.9;
    } else if lower_text.contains("confident") {
        info!("[EXTRACT] Found confident indicator");
        return 0.8;
    } else if lower_text.contains("moderate") {
        info!("[EXTRACT] Found moderate confidence indicator");
        return 0.7;
    } else if lower_text.contains("low confidence") {
        info!("[EXTRACT] Found low confidence indicator");
        return 0.5;
    }
    
    // Try to find any numerical value that might be confidence
    if let Some(number_match) = regex::Regex::new(r"0\.(\d+)|0,(\d+)|\b(\d{1,2})%")
        .unwrap()
        .find(&lower_text) {
        let match_str = number_match.as_str();
        info!("[EXTRACT] Found potential confidence value: '{}'", match_str);
        
        if match_str.ends_with('%') {
            if let Ok(conf) = match_str.trim_end_matches('%').parse::<f32>() {
                let normalized = conf / 100.0;
                info!("[EXTRACT] Parsed percentage confidence: {}% -> {}", conf, normalized);
                return normalized;
            }
        } else if let Ok(conf) = match_str.replace(',', ".").parse::<f32>() {
            let normalized = if conf > 1.0 { conf / 100.0 } else { conf };
            info!("[EXTRACT] Parsed decimal confidence: {} -> {}", conf, normalized);
            return normalized;
        }
    }
    
    // Default confidence
    warn!("[EXTRACT] No confidence indicators found, using default: 0.75");
    0.75
}

// DPI-aware window management command implementations

#[tauri::command]
fn setup_dpi_aware_positioning(app_handle: AppHandle) -> Result<String, String> {
    info!("🖥️ Setting up DPI-aware positioning for main window");
    
    match window_manager::setup_main_window_dpi_aware(&app_handle) {
        Ok(_) => Ok("DPI-aware positioning setup completed".to_string()),
        Err(e) => {
            error!("Failed to setup DPI-aware positioning: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
fn get_window_info(app_handle: AppHandle) -> Result<window_manager::WindowConfiguration, String> {
    info!("📊 Getting current window information");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        match window_manager::get_window_configuration(&window) {
            Ok(config) => {
                info!("✅ Window info: {}x{} at ({}, {}) on {}", 
                      config.width, config.height, config.x, config.y, config.monitor_name);
                Ok(config)
            }
            Err(e) => {
                error!("Failed to get window info: {}", e);
                Err(e)
            }
        }
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
fn get_monitors_info(app_handle: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    info!("🖥️ Getting monitors information");
    
    match window_manager::get_monitors_info(&app_handle) {
        Ok(monitors) => {
            info!("✅ Found {} monitors", monitors.len());
            Ok(monitors)
        }
        Err(e) => {
            error!("Failed to get monitors info: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
fn lock_window_size(app_handle: AppHandle, width: u32, height: u32) -> Result<String, String> {
    info!("🔒 Locking window size to {}x{}", width, height);
    
    if let Some(window) = app_handle.get_webview_window("main") {
        match window_manager::lock_window_size(&window, width, height) {
            Ok(_) => Ok(format!("Window size locked to {}x{}", width, height)),
            Err(e) => {
                error!("Failed to lock window size: {}", e);
                Err(e)
            }
        }
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
fn ensure_window_visible(app_handle: AppHandle) -> Result<String, String> {
    info!("👁️ Ensuring window is visible on current screen setup");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        match window_manager::ensure_window_visible(&window) {
            Ok(_) => Ok("Window visibility check completed".to_string()),
            Err(e) => {
                error!("Failed to ensure window visibility: {}", e);
                Err(e)
            }
        }
    } else {
        Err("Main window not found".to_string())
    }
}

// Helper function to get environment variables using runtime loading
fn get_env_var(key: &str) -> Option<String> {
    // Load .env file if it exists for development
    let _ = dotenvy::dotenv();
    
    // Try runtime environment variable first
    if let Ok(value) = std::env::var(key) {
        if !value.is_empty() {
            return Some(value);
        }
    }
    
    // Try compile-time embedded variables as fallback (only if they were set during build)
    // Use option_env!() instead of env!() to avoid compile-time errors
    let embedded_value = match key {
        "DEEPGRAM_API_KEY" => option_env!("DEEPGRAM_API_KEY"),
        "OPENAI_API_KEY" => option_env!("OPENAI_API_KEY"),
        "POLLINATIONS_API_KEY" => option_env!("POLLINATIONS_API_KEY"),
        "POLLINATIONS_REFERER" => option_env!("POLLINATIONS_REFERER"),
        "DEEPGRAM_MODEL" => option_env!("DEEPGRAM_MODEL"),
        "DEEPGRAM_LANGUAGE" => option_env!("DEEPGRAM_LANGUAGE"),
        "DEEPGRAM_ENDPOINTING" => option_env!("DEEPGRAM_ENDPOINTING"),
        "DEEPGRAM_INTERIM_RESULTS" => option_env!("DEEPGRAM_INTERIM_RESULTS"),
        "DEEPGRAM_SMART_FORMAT" => option_env!("DEEPGRAM_SMART_FORMAT"),
        "DEEPGRAM_KEEP_ALIVE" => option_env!("DEEPGRAM_KEEP_ALIVE"),
        "DEEPGRAM_PUNCTUATE" => option_env!("DEEPGRAM_PUNCTUATE"),
        "DEEPGRAM_PROFANITY_FILTER" => option_env!("DEEPGRAM_PROFANITY_FILTER"),
        "DEEPGRAM_DIARIZE" => option_env!("DEEPGRAM_DIARIZE"),
        "DEEPGRAM_MULTICHANNEL" => option_env!("DEEPGRAM_MULTICHANNEL"),
        "DEEPGRAM_NUMERALS" => option_env!("DEEPGRAM_NUMERALS"),
        "DB_HOST" => option_env!("DB_HOST"),
        "DB_PORT" => option_env!("DB_PORT"),
        "DB_NAME" => option_env!("DB_NAME"),
        "DB_USER" => option_env!("DB_USER"),
        "DB_PASSWORD" => option_env!("DB_PASSWORD"),
        _ => None,
    };
    
    // Return embedded value if it exists and is not empty
    if let Some(value) = embedded_value {
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    
    None
}

// Helper function to handle protocol launch with temporary tokens
async fn handle_protocol_launch_with_temp_token(
    session_id: String,
    _token: Option<String>, 
    temp_token: Option<String>,
    _user_id: Option<String>,
    auto_connect: Option<bool>,
    _auto_fill: Option<bool>,
    app_handle: AppHandle,
) -> Result<(), String> {
    info!("🚀 Protocol launch with temp token for session: {}", session_id);
    
    // Bring the main window to front and focus
    if let Some(main_window) = app_handle.get_webview_window("main") {
        if let Err(e) = main_window.show() {
            error!("Failed to show main window: {}", e);
        }
        if let Err(e) = main_window.set_focus() {
            error!("Failed to focus main window: {}", e);
        }
    }
    
    // If we have a temp token and auto_connect is enabled, authenticate automatically
    if let (Some(temp_token), Some(true)) = (temp_token.clone(), auto_connect) {
        info!("🔐 Auto-authenticating with temporary token...");
        
        let auth_payload = TempTokenAuthPayload {
            session_id: session_id.clone(),
            temp_token: temp_token.clone(),
        };
        
        match connect_with_temp_token(auth_payload).await {
            Ok(auth_response) => {
                if auth_response.success {
                    info!("✅ Auto-authentication successful!");
                    
                    // Emit successful authentication event to frontend
                    let auth_event_data = serde_json::json!({
                        "type": "temp-token-auth-success",
                        "session_id": session_id,
                        "user_id": auth_response.user_id,
                        "session": auth_response.session,
                        "remaining_credits": auth_response.remaining_credits,
                        "auto_authenticated": true,
                        "launched_at": chrono::Utc::now().to_rfc3339()
                    });
                    
                    if let Err(e) = app_handle.emit("temp-token-auth-result", auth_event_data) {
                        error!("Failed to emit temp-token-auth-result event: {}", e);
                    }
                } else {
                    warn!("❌ Auto-authentication failed: {}", auth_response.message);
                    
                    // Emit failed authentication event
                    let auth_event_data = serde_json::json!({
                        "type": "temp-token-auth-failed",
                        "session_id": session_id,
                        "error": auth_response.message,
                        "auto_authenticated": true,
                        "launched_at": chrono::Utc::now().to_rfc3339()
                    });
                    
                    if let Err(e) = app_handle.emit("temp-token-auth-result", auth_event_data) {
                        error!("Failed to emit temp-token-auth-result event: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ Auto-authentication error: {}", e);
                
                // Emit error authentication event
                let auth_event_data = serde_json::json!({
                    "type": "temp-token-auth-error",
                    "session_id": session_id,
                    "error": e,
                    "auto_authenticated": true,
                    "launched_at": chrono::Utc::now().to_rfc3339()
                });
                
                if let Err(emit_err) = app_handle.emit("temp-token-auth-result", auth_event_data) {
                    error!("Failed to emit temp-token-auth-result event: {}", emit_err);
                }
            }
        }
    } else {
        info!("📋 No auto-authentication - sending session launch data to frontend");
        
        // Send session launch data with temp token to frontend for manual handling
        let session_launch_data = serde_json::json!({
            "type": "session-launch",
            "session_id": session_id,
            "temp_token": temp_token,
            "auto_connect": auto_connect,
            "launched_at": chrono::Utc::now().to_rfc3339()
        });
        
        if let Err(e) = app_handle.emit("session-launch", session_launch_data) {
            error!("Failed to emit session-launch event: {}", e);
        }
    }
    
    info!("✅ Protocol launch with temp token handled successfully");
    Ok(())
}

// Helper function to log environment variable status
fn log_environment_status() {
    info!("🔧 Environment Configuration Status (using embedded + runtime fallback):");
    
    // Check build-time embedded variables vs runtime variables using our helper
    match get_env_var("DEEPGRAM_API_KEY") {
        Some(key) => {
            let key_preview = if key.len() > 8 { 
                format!("{}...{}", &key[..4], &key[key.len()-4..])
            } else { 
                "***".to_string()
            };
            info!("✅ DEEPGRAM_API_KEY: {} (length: {})", key_preview, key.len());
        }
        None => warn!("❌ DEEPGRAM_API_KEY: Not available (neither runtime nor embedded)")
    }
    
    match get_env_var("OPENAI_API_KEY") {
        Some(key) => {
            let key_preview = if key.len() > 8 { 
                format!("{}...{}", &key[..4], &key[key.len()-4..])
            } else { 
                "***".to_string()
            };
            info!("✅ OPENAI_API_KEY: {} (length: {})", key_preview, key.len());
        }
        None => warn!("❌ OPENAI_API_KEY: Not available (neither runtime nor embedded)")
    }
    
    match get_env_var("POLLINATIONS_API_KEY") {
        Some(key) => {
            let key_preview = if key.len() > 8 { 
                format!("{}...{}", &key[..4], &key[key.len()-4..])
            } else { 
                "***".to_string()
            };
            info!("✅ POLLINATIONS_API_KEY: {} (length: {})", key_preview, key.len());
        }
        None => warn!("❌ POLLINATIONS_API_KEY: Not available (neither runtime nor embedded)")
    }
    
    // Database configuration
    match get_env_var("DB_HOST") {
        Some(host) => info!("✅ DB_HOST: {}", host),
        None => warn!("❌ DB_HOST: Not available (neither runtime nor embedded)")
    }
    
    // Deepgram configuration
    match get_env_var("DEEPGRAM_MODEL") {
        Some(model) => info!("✅ DEEPGRAM_MODEL: {}", model),
        None => warn!("❌ DEEPGRAM_MODEL: Not available (will use default)")
    }
    
    info!("🔧 Environment configuration check complete (build-time embedded + runtime fallback)");
}

// Temporary token authentication command
#[tauri::command]
async fn connect_with_temp_token(payload: TempTokenAuthPayload) -> Result<TempTokenAuthResponse, String> {
    info!("🔐 Authenticating with temporary token for session: {}", payload.session_id);
    
    // Prepare the request
    let backend_url = get_env_var("BACKEND_URL")
        .unwrap_or_else(|| "http://localhost:3001".to_string());
    let endpoint = format!("{}/api/sessions/{}/connect-with-temp-token", backend_url, payload.session_id);
    
    info!("📡 Sending temp token auth request to: {}", endpoint);
    
    // Create the request payload
    let request_body = serde_json::json!({
        "temp_token": payload.temp_token
    });
    
    // Make the HTTP request
    let client = reqwest::Client::new();
    match client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            info!("📡 Received response with status: {}", status);
            
            if status.is_success() {
                match response.json::<TempTokenAuthResponse>().await {
                    Ok(auth_response) => {
                        if auth_response.success {
                            info!("✅ Temporary token authentication successful!");
                            info!("👤 Authenticated user: {:?}", auth_response.user_id);
                            info!("💳 Remaining credits: {:?}", auth_response.remaining_credits);
                        } else {
                            warn!("❌ Authentication failed: {}", auth_response.message);
                        }
                        Ok(auth_response)
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to parse authentication response: {}", e);
                        error!("❌ {}", error_msg);
                        Err(error_msg)
                    }
                }
            } else {
                // Handle HTTP error status
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                let error_msg = format!("Authentication failed with status {}: {}", status, error_text);
                error!("❌ {}", error_msg);
                
                // Return a failed response instead of an error for better UX
                Ok(TempTokenAuthResponse {
                    success: false,
                    message: error_msg,
                    user_id: None,
                    session: None,
                    remaining_credits: None,
                })
            }
        }
        Err(e) => {
            let error_msg = format!("Network error during authentication: {}", e);
            error!("❌ {}", error_msg);
            
            // Return a failed response instead of an error for better UX
            Ok(TempTokenAuthResponse {
                success: false,
                message: error_msg,
                user_id: None,
                session: None,
                remaining_credits: None,
            })
        }
    }
}


