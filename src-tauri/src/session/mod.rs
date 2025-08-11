pub mod manager;
pub mod activation;
pub mod sync;

pub use manager::*;
pub use activation::*;
pub use sync::*;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use lazy_static::lazy_static;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Global state for active sessions and timers
lazy_static! {
    static ref ACTIVE_SESSIONS: Arc<Mutex<HashMap<String, SessionConnection>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref TIMER_STORE: Arc<Mutex<HashMap<String, crate::interview::timer::InterviewTimer>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionConnection {
    pub session_id: String,
    pub status: SessionStatus,
    pub user_details: UserDetails,
    pub interview_config: InterviewConfig,
    pub credits_available: i32,
    pub session_duration_limit: u64, // in minutes
    pub desktop_connected_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SessionStatus {
    Created,
    Active,
    Completed,
    Expired,
    Disconnected,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDetails {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InterviewConfig {
    pub job_title: String,
    pub job_description: String,
    pub difficulty: String,
    pub session_type: String,
    pub resume_content: Option<String>,
}

// Helper functions to manage global state
pub async fn store_active_session(connection: SessionConnection) -> Result<(), String> {
    let mut sessions = ACTIVE_SESSIONS.lock();
    sessions.insert(connection.session_id.clone(), connection);
    Ok(())
}

pub async fn get_active_session(session_id: &str) -> Option<SessionConnection> {
    let sessions = ACTIVE_SESSIONS.lock();
    sessions.get(session_id).cloned()
}

pub async fn remove_active_session(session_id: &str) -> Option<SessionConnection> {
    let mut sessions = ACTIVE_SESSIONS.lock();
    sessions.remove(session_id)
}

pub async fn get_timer_store() -> Arc<Mutex<HashMap<String, crate::interview::timer::InterviewTimer>>> {
    TIMER_STORE.clone()
}
