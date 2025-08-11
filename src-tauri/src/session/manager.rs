use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use chrono::Utc;
use uuid::Uuid;
use crate::database::DatabaseManager;
use super::{SessionConnection, SessionStatus, UserDetails, InterviewConfig};

#[tauri::command]
pub async fn connect_session(session_id: String) -> Result<SessionConnection, String> {
    info!("🔌 Connecting to session: {}", session_id);
    
    // 1. Connect to main PostgreSQL database (web app DB)
    let db = DatabaseManager::new().await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    // 2. Validate session ID exists and is in 'created' status
    let session = db.get_session_by_id(&session_id).await
        .map_err(|e| format!("Session not found or invalid: {}", e))?;
    
    if session.status != "created" {
        return Err(format!("Session already started or completed. Current status: {}", session.status));
    }
    
    // 3. Check user credits
    let user = db.get_user_by_id(&session.user_id.to_string()).await
        .map_err(|e| format!("User not found: {}", e))?;
    
    if user.credits < 1 {
        return Err("Insufficient credits to start session".to_string());
    }
    
    info!("✅ Session validation successful. User {} has {} credits", user.first_name, user.credits);
    
    // 4. Create session connection data for desktop app
    let user_name = match user.last_name {
        Some(last_name) => format!("{} {}", user.first_name, last_name),
        None => user.first_name,
    };
    
    let connection = SessionConnection {
        session_id: session_id.clone(),
        status: SessionStatus::Created, // Still created until activated
        user_details: UserDetails {
            id: user.id.to_string(),
            name: user_name,
            email: user.email,
        },
        interview_config: InterviewConfig {
            job_title: session.job_title,
            job_description: session.job_description,
            difficulty: session.difficulty,
            session_type: session.session_type,
            resume_content: session.resume_content,
        },
        credits_available: user.credits,
        session_duration_limit: 60, // 60 minutes per credit
        desktop_connected_at: Utc::now(),
    };
    
    info!("🎯 Session connection prepared for: {}", connection.interview_config.job_title);
    
    Ok(connection)
}

#[tauri::command]
pub async fn get_session_status(session_id: String) -> Result<String, String> {
    info!("📊 Getting session status: {}", session_id);
    
    let db = DatabaseManager::new().await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    let session = db.get_session_by_id(&session_id).await
        .map_err(|e| format!("Session not found: {}", e))?;
    
    Ok(session.status)
}

#[tauri::command]
pub async fn validate_session_access(session_id: String, user_id: String) -> Result<bool, String> {
    info!("🔐 Validating session access for user: {}", user_id);
    
    let db = DatabaseManager::new().await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    let session = db.get_session_by_id(&session_id).await
        .map_err(|e| format!("Session not found: {}", e))?;
    
    if session.user_id.to_string() != user_id {
        warn!("❌ Access denied: Session {} does not belong to user {}", session_id, user_id);
        return Ok(false);
    }
    
    info!("✅ Access granted: User {} owns session {}", user_id, session_id);
    Ok(true)
}

#[derive(Serialize, Deserialize)]
pub struct SessionValidationResult {
    pub valid: bool,
    pub message: String,
    pub session_data: Option<SessionConnection>,
}

#[tauri::command]
pub async fn activate_session(session_id: String) -> Result<bool, String> {
    info!("🚀 Activating session: {}", session_id);
    
    // 1. Connect to database and validate session
    let db = DatabaseManager::new().await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    let session = db.get_session_by_id(&session_id).await
        .map_err(|e| format!("Session not found: {}", e))?;
    
    if session.status != "created" {
        return Err(format!("Session cannot be activated. Current status: {}", session.status));
    }
    
    // 2. Check user credits again before activation
    let user = db.get_user_by_id(&session.user_id.to_string()).await
        .map_err(|e| format!("User not found: {}", e))?;
    
    if user.credits < 1 {
        return Err("Insufficient credits to activate session".to_string());
    }
    
    // 3. Update session status to 'active' and deduct initial credit
    db.update_session_status(&session_id, "active").await
        .map_err(|e| format!("Failed to update session status: {}", e))?;
    
    db.deduct_user_credits(&session.user_id.to_string(), 1).await
        .map_err(|e| {
            // Try to revert session status if credit deduction fails
            let _ = futures::executor::block_on(async {
                db.update_session_status(&session_id, "created").await
            });
            format!("Failed to deduct credits: {}", e)
        })?;
    
    // 4. Create and store active session
    let user_name = match user.last_name {
        Some(last_name) => format!("{} {}", user.first_name, last_name),
        None => user.first_name,
    };
    
    let connection = SessionConnection {
        session_id: session_id.clone(),
        status: SessionStatus::Active,
        user_details: UserDetails {
            id: user.id.to_string(),
            name: user_name,
            email: user.email,
        },
        interview_config: InterviewConfig {
            job_title: session.job_title,
            job_description: session.job_description,
            difficulty: session.difficulty,
            session_type: session.session_type,
            resume_content: session.resume_content,
        },
        credits_available: user.credits - 1,
        session_duration_limit: 60,
        desktop_connected_at: Utc::now(),
    };
    
    // Store in active sessions
    crate::session::store_active_session(connection).await
        .map_err(|e| format!("Failed to store active session: {}", e))?;
    
    info!("✅ Session {} successfully activated. 1 credit deducted from user {}", session_id, user.id);
    
    Ok(true)
}

#[tauri::command]
pub async fn validate_session_id(session_id: String) -> Result<SessionValidationResult, String> {
    info!("🔍 Validating session ID: {}", session_id);
    
    if session_id.trim().is_empty() {
        return Ok(SessionValidationResult {
            valid: false,
            message: "Session ID cannot be empty".to_string(),
            session_data: None,
        });
    }
    
    // Validate UUID format
    if Uuid::parse_str(&session_id).is_err() {
        return Ok(SessionValidationResult {
            valid: false,
            message: "Invalid session ID format. Please check your session ID.".to_string(),
            session_data: None,
        });
    }
    
    // Try to connect to get session details
    match connect_session(session_id).await {
        Ok(connection) => Ok(SessionValidationResult {
            valid: true,
            message: "Session is valid and ready for activation".to_string(),
            session_data: Some(connection),
        }),
        Err(e) => Ok(SessionValidationResult {
            valid: false,
            message: e,
            session_data: None,
        }),
    }
}

#[tauri::command]
pub async fn disconnect_session(session_id: String) -> Result<bool, String> {
    info!("🔌 Disconnecting from session: {}", session_id);
    
    // Remove from active sessions
    let removed_session = crate::session::remove_active_session(&session_id).await;
    
    if removed_session.is_some() {
        // Update database status
        let db = DatabaseManager::new().await
            .map_err(|e| format!("Database connection failed: {}", e))?;
        
        db.update_session_status(&session_id, "disconnected").await
            .map_err(|e| format!("Failed to update session status: {}", e))?;
        
        info!("✅ Session {} disconnected successfully", session_id);
        Ok(true)
    } else {
        warn!("⚠️ Session {} was not found in active sessions", session_id);
        Ok(false)
    }
}

#[tauri::command]
pub async fn get_active_session_info(session_id: String) -> Result<SessionConnection, String> {
    info!("📋 Getting active session info: {}", session_id);
    
    if let Some(session) = crate::session::get_active_session(&session_id).await {
        Ok(session)
    } else {
        Err("Session not found in active sessions".to_string())
    }
}

#[tauri::command]
pub async fn update_session_heartbeat(session_id: String) -> Result<(), String> {
    info!("💓 Session heartbeat: {}", session_id);
    
    // Update last active timestamp for the session
    if let Some(mut session) = crate::session::get_active_session(&session_id).await {
        session.desktop_connected_at = Utc::now();
        let _ = crate::session::store_active_session(session).await;
    }
    
    Ok(())
}
