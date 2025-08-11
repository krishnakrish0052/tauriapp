use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub job_title: String,
    pub job_description: String,
    pub difficulty: String,
    pub session_type: String,
    pub status: String,
    pub resume_content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub desktop_connected_at: Option<DateTime<Utc>>,
    pub interview_duration: Option<i32>,
    pub credits_used: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub credits: i32,
    pub created_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewQuestion {
    pub id: Uuid,
    pub session_id: Uuid,
    pub question_number: i32,
    pub question_text: String,
    pub category: String,
    pub difficulty_level: String,
    pub expected_duration: i32,
    pub asked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewAnswer {
    pub id: Uuid,
    pub question_id: Uuid,
    pub session_id: Uuid,
    pub answer_text: Option<String>,
    pub response_time: Option<i32>,
    pub ai_feedback: Option<String>,
    pub ai_score: Option<i32>,
    pub answered_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConnection {
    pub id: Uuid,
    pub session_id: Uuid,
    pub desktop_app_version: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub credits_deducted: i32,
    pub created_at: DateTime<Utc>,
}

// DTOs for API communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConnectionRequest {
    pub session_id: String,
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivationRequest {
    pub session_id: String,
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivationResult {
    pub success: bool,
    pub message: Option<String>,
    pub remaining_credits: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDetails {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewConfig {
    pub job_title: String,
    pub job_description: String,
    pub difficulty: String,
    pub session_type: String,
    pub resume_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub job_title: String,
    pub user_name: String,
    pub difficulty: String,
    pub credits_available: i32,
    pub status: String,
}
