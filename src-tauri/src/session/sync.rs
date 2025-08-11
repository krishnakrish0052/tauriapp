use log::{info, error, warn};
use serde_json::Value;
use crate::database::DatabaseManager;

#[tauri::command]
pub async fn sync_session_progress(session_id: String, progress_data: Value) -> Result<(), String> {
    info!("🔄 Syncing session progress: {}", session_id);
    
    // For now, just log the progress data
    // In the future, we could store additional progress metrics in the database
    info!("Progress data: {}", progress_data);
    
    Ok(())
}

#[tauri::command]
pub async fn sync_session_metadata(session_id: String, metadata: Value) -> Result<(), String> {
    info!("📊 Syncing session metadata: {}", session_id);
    
    // For now, just log the metadata
    // In the future, we could store session metadata like interview quality, etc.
    info!("Metadata: {}", metadata);
    
    Ok(())
}

pub async fn sync_session_to_web_db(
    session_id: &str, 
    duration_minutes: u32, 
    credits_used: i32
) -> Result<(), String> {
    info!("💾 Syncing session to web database: {} ({}min, {} credits)", session_id, duration_minutes, credits_used);
    
    let db = DatabaseManager::new().await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    db.update_session_duration_and_credits(session_id, duration_minutes, credits_used).await
        .map_err(|e| format!("Failed to sync session data: {}", e))?;
    
    info!("✅ Session data synced successfully");
    Ok(())
}
