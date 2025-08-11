use log::{info, error, warn};
use crate::database::DatabaseManager;
use super::{SessionConnection, SessionStatus, store_active_session};

// This function is now handled by session::manager::activate_session
// to avoid duplicate command names

#[tauri::command]
pub async fn deactivate_session(session_id: String) -> Result<bool, String> {
    info!("🛑 Deactivating session: {}", session_id);
    
    // 1. Update session status to completed
    let db = DatabaseManager::new().await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    db.update_session_status(&session_id, "completed").await
        .map_err(|e| format!("Failed to update session status: {}", e))?;
    
    // 2. Remove from active sessions
    super::remove_active_session(&session_id).await;
    
    // 3. Clean up any running timers
    let timer_store = super::get_timer_store().await;
    let mut timers = timer_store.lock();
    timers.remove(&session_id);
    
    info!("✅ Session {} deactivated successfully!", session_id);
    
    Ok(true)
}

// This function is now handled by session::manager::get_active_session_info
// to avoid duplicate command names
