use tokio_postgres::{Client, NoTls, Error as PgError};
use deadpool_postgres::{Config, Pool, Runtime};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::{info, error, warn};
use std::str::FromStr;

use super::{DatabaseError, Result};
use super::models::*;
use crate::database::models::SessionInfo;

#[derive(Debug)]
pub struct DatabaseManager {
    pool: Pool,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self> {
        // Read individual database configuration variables (same as shared.rs)
        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()).parse().unwrap_or(5432);
        let dbname = std::env::var("DB_NAME").unwrap_or_else(|_| "mockmate_db".to_string());
        let user = std::env::var("DB_USER").unwrap_or_else(|_| "mockmate_user".to_string());
        let password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "".to_string());

        // Construct database URL from individual components
        let database_url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, dbname);

        info!("Connecting to database: {}@{}:{}/{}", user, host, port, dbname);

        let mut cfg = Config::new();
        cfg.url = Some(database_url);
        cfg.manager = Some(deadpool_postgres::ManagerConfig { 
            recycling_method: deadpool_postgres::RecyclingMethod::Fast 
        });
        
        let pool = cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
            .map_err(|e| DatabaseError::ConnectionFailed(format!("Pool creation failed: {}", e)))?;

        // Test connection
        let _client = pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(format!("Connection test failed: {}", e)))?;
        
        info!("Database connection established successfully");

        Ok(DatabaseManager { pool })
    }

    pub async fn get_session_by_id(&self, session_id: &str) -> Result<Session> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let row = client
            .query_one(
                r#"
                SELECT id, user_id, job_title, job_description, difficulty, 
                       session_type, status, resume_content, created_at, 
                       desktop_connected_at, interview_duration, credits_used
                FROM sessions 
                WHERE id = $1
                "#,
                &[&session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to fetch session {}: {}", session_id, e);
                DatabaseError::SessionNotFound(format!("Session not found: {}", e))
            })?;

        Ok(Session {
            id: row.get(0),
            user_id: row.get(1),
            job_title: row.get(2),
            job_description: row.get(3),
            difficulty: row.get(4),
            session_type: row.get(5),
            status: row.get(6),
            resume_content: row.get(7),
            created_at: row.get(8),
            desktop_connected_at: row.get(9),
            interview_duration: row.get(10),
            credits_used: row.get(11),
        })
    }

    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let user_uuid = Uuid::from_str(user_id)
            .map_err(|_| DatabaseError::UserNotFound("Invalid user ID format".to_string()))?;

        let row = client
            .query_one(
                r#"
                SELECT id, email, first_name, last_name, credits, created_at, last_active
                FROM users 
                WHERE id = $1
                "#,
                &[&user_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to fetch user {}: {}", user_id, e);
                DatabaseError::UserNotFound(format!("User not found: {}", e))
            })?;

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            first_name: row.get(2),
            last_name: row.get(3),
            credits: row.get(4),
            created_at: row.get(5),
            last_active: row.get(6),
        })
    }

    pub async fn update_session_status(&self, session_id: &str, status: &str) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let now = Utc::now();
        
        let rows_affected = client
            .execute(
                r#"
                UPDATE sessions 
                SET status = $1, desktop_connected_at = CASE 
                    WHEN $1 = 'active' AND desktop_connected_at IS NULL THEN $2
                    ELSE desktop_connected_at 
                END
                WHERE id = $3
                "#,
                &[&status, &now, &session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to update session status: {}", e);
                DatabaseError::QueryFailed(format!("Failed to update session status: {}", e))
            })?;

        if rows_affected == 0 {
            return Err(DatabaseError::SessionNotFound("Session not found for status update".to_string()));
        }

        info!("Session {} status updated to: {}", session_id, status);
        Ok(())
    }

    pub async fn deduct_user_credits(&self, user_id: &str, credits: i32) -> Result<i32> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let user_uuid = Uuid::from_str(user_id)
            .map_err(|_| DatabaseError::UserNotFound("Invalid user ID format".to_string()))?;

        let row = client
            .query_one(
                r#"
                UPDATE users 
                SET credits = credits - $1 
                WHERE id = $2 AND credits >= $1
                RETURNING credits
                "#,
                &[&credits, &user_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to deduct credits for user {}: {}", user_id, e);
                if e.to_string().contains("no rows") {
                    DatabaseError::InsufficientCredits
                } else {
                    DatabaseError::QueryFailed(format!("Failed to deduct credits: {}", e))
                }
            })?;

        let remaining_credits: i32 = row.get(0);
        info!("Deducted {} credits from user {}. Remaining: {}", credits, user_id, remaining_credits);
        
        Ok(remaining_credits)
    }

    pub async fn insert_interview_question(
        &self, 
        session_id: &str, 
        question_number: i32,
        question_text: &str,
        category: &str,
        difficulty_level: &str,
        expected_duration: i32
    ) -> Result<Uuid> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;
        
        let question_id = Uuid::new_v4();
        let now = Utc::now();

        client
            .execute(
                r#"
                INSERT INTO interview_questions 
                (id, session_id, question_number, question_text, category, 
                 difficulty_level, expected_duration, asked_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
                "#,
                &[
                    &question_id,
                    &session_uuid,
                    &question_number,
                    &question_text,
                    &category,
                    &difficulty_level,
                    &expected_duration,
                    &now,
                ]
            )
            .await
            .map_err(|e| {
                error!("Failed to insert interview question: {}", e);
                DatabaseError::QueryFailed(format!("Failed to insert question: {}", e))
            })?;

        info!("Inserted interview question {} for session {}", question_id, session_id);
        Ok(question_id)
    }

    pub async fn insert_interview_answer(
        &self,
        question_id: &Uuid,
        session_id: &str,
        answer_text: Option<&str>,
        response_time: Option<i32>,
        ai_feedback: Option<&str>,
        ai_score: Option<i32>
    ) -> Result<Uuid> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;
        
        let answer_id = Uuid::new_v4();
        let now = Utc::now();

        client
            .execute(
                r#"
                INSERT INTO interview_answers 
                (id, question_id, session_id, answer_text, response_time, 
                 ai_feedback, ai_score, answered_at, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
                "#,
                &[
                    &answer_id,
                    question_id,
                    &session_uuid,
                    &answer_text,
                    &response_time,
                    &ai_feedback,
                    &ai_score,
                    &now,
                ]
            )
            .await
            .map_err(|e| {
                error!("Failed to insert interview answer: {}", e);
                DatabaseError::QueryFailed(format!("Failed to insert answer: {}", e))
            })?;

        info!("Inserted interview answer {} for session {}", answer_id, session_id);
        Ok(answer_id)
    }

    pub async fn update_session_duration_and_credits(
        &self, 
        session_id: &str, 
        duration_minutes: u32, 
        credits_used: i32
    ) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let rows_affected = client
            .execute(
                r#"
                UPDATE sessions 
                SET interview_duration = $1, credits_used = $2 
                WHERE id = $3
                "#,
                &[&(duration_minutes as i32), &credits_used, &session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to update session duration: {}", e);
                DatabaseError::QueryFailed(format!("Failed to update session duration: {}", e))
            })?;

        if rows_affected == 0 {
            return Err(DatabaseError::SessionNotFound("Session not found for duration update".to_string()));
        }

        info!("Updated session {} duration: {}min, credits: {}", session_id, duration_minutes, credits_used);
        Ok(())
    }

    pub async fn create_session_connection(
        &self,
        session_id: &str,
        desktop_app_version: Option<&str>,
        credits_deducted: i32
    ) -> Result<Uuid> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;
        
        let connection_id = Uuid::new_v4();
        let now = Utc::now();

        client
            .execute(
                r#"
                INSERT INTO session_connections 
                (id, session_id, desktop_app_version, connected_at, 
                 credits_deducted, created_at)
                VALUES ($1, $2, $3, $4, $5, $4)
                "#,
                &[
                    &connection_id,
                    &session_uuid,
                    &desktop_app_version,
                    &now,
                    &credits_deducted,
                ]
            )
            .await
            .map_err(|e| {
                error!("Failed to create session connection: {}", e);
                DatabaseError::QueryFailed(format!("Failed to create session connection: {}", e))
            })?;

        info!("Created session connection {} for session {}", connection_id, session_id);
        Ok(connection_id)
    }

    pub async fn update_session_heartbeat(&self, session_id: &str) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let now = Utc::now();
        
        let rows_affected = client
            .execute(
                r#"
                UPDATE sessions 
                SET desktop_connected_at = $1 
                WHERE id = $2
                "#,
                &[&now, &session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to update session heartbeat: {}", e);
                DatabaseError::QueryFailed(format!("Failed to update session heartbeat: {}", e))
            })?;

        if rows_affected == 0 {
            return Err(DatabaseError::SessionNotFound("Session not found for heartbeat update".to_string()));
        }

        info!("Session {} heartbeat updated", session_id);
        Ok(())
    }

    pub async fn test_connection(&self) -> Result<String> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        let row = client
            .query_one("SELECT version()", &[])
            .await
            .map_err(|e| DatabaseError::QueryFailed(format!("Test query failed: {}", e)))?;

        let version: String = row.get(0);
        info!("Database connection test successful: {}", version);
        Ok(format!("Database connection successful. {}", version))
    }
}

// Tauri commands for database operations
#[tauri::command]
pub async fn test_database_connection() -> std::result::Result<String, String> {
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    db.test_connection().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_db_session_info(session_id: String) -> std::result::Result<SessionInfo, String> {
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    let session = db.get_session_by_id(&session_id).await
        .map_err(|e| e.to_string())?;
    
    let user = db.get_user_by_id(&session.user_id.to_string()).await
        .map_err(|e| e.to_string())?;

    let user_name = match user.last_name {
        Some(last_name) => format!("{} {}", user.first_name, last_name),
        None => user.first_name,
    };

    Ok(SessionInfo {
        id: session.id.to_string(),
        job_title: session.job_title,
        user_name,
        difficulty: session.difficulty,
        credits_available: user.credits,
        status: session.status,
    })
}
