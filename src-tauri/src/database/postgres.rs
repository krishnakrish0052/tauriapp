use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use uuid::Uuid;
use chrono::Utc;
use log::{info, error};
use std::str::FromStr;
use serde::{Serialize, Deserialize};

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
        
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)
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
                       desktop_connected_at, session_started_at, interview_duration, credits_used
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
            session_started_at: row.get(10),
            interview_duration: row.get(11),
            credits_used: row.get(12),
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
                SET status = $1, 
                    desktop_connected_at = CASE 
                        WHEN $1 = 'active' AND desktop_connected_at IS NULL THEN $2
                        ELSE desktop_connected_at 
                    END,
                    session_started_at = CASE 
                        WHEN $1 = 'active' AND session_started_at IS NULL THEN $2
                        ELSE session_started_at 
                    END
                WHERE id = $3
                "#,
                &[&status, &now.naive_utc(), &session_uuid]
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
        
        let message_id = Uuid::new_v4();
        let now = Utc::now();

        // Create metadata JSON with question details
        let metadata = serde_json::json!({
            "questionNumber": question_number,
            "category": category,
            "difficulty": difficulty_level,
            "expectedDuration": expected_duration,
            "source": "desktop_app",
            "timestamp": now.to_rfc3339()
        });

        client
            .execute(
                r#"
                INSERT INTO interview_messages 
                (id, session_id, message_type, content, metadata, timestamp)
                VALUES ($1, $2, 'question', $3, $4, $5)
                "#,
                &[
                    &message_id,
                    &session_uuid,
                    &question_text,
                    &metadata,
                    &now.naive_utc(),
                ]
            )
            .await
            .map_err(|e| {
                error!("Failed to insert interview question: {}", e);
                DatabaseError::QueryFailed(format!("Failed to insert question: {}", e))
            })?;

        info!("Inserted interview question {} for session {}", message_id, session_id);
        Ok(message_id)
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
        
        let message_id = Uuid::new_v4();
        let now = Utc::now();

        // Create metadata JSON with answer details
        let metadata = serde_json::json!({
            "questionId": question_id,
            "responseTime": response_time,
            "aiFeedback": ai_feedback,
            "aiScore": ai_score,
            "source": "desktop_app",
            "timestamp": now.to_rfc3339()
        });

        // Use answer_text or default to empty string if None
        let content = answer_text.unwrap_or("");

        client
            .execute(
                r#"
                INSERT INTO interview_messages 
                (id, session_id, message_type, content, metadata, timestamp, parent_message_id)
                VALUES ($1, $2, 'answer', $3, $4, $5, $6)
                "#,
                &[
                    &message_id,
                    &session_uuid,
                    &content,
                    &metadata,
                    &now.naive_utc(),
                    question_id, // Link the answer to its question using parent_message_id
                ]
            )
            .await
            .map_err(|e| {
                error!("Failed to insert interview answer: {}", e);
                DatabaseError::QueryFailed(format!("Failed to insert answer: {}", e))
            })?;

        info!("Inserted interview answer {} for session {}", message_id, session_id);
        Ok(message_id)
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
                    &now.naive_utc(),
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
                &[&now.naive_utc(), &session_uuid]
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

    pub async fn mark_session_started(&self, session_id: &str) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let now = Utc::now();
        
        let rows_affected = client
            .execute(
                r#"
                UPDATE sessions 
                SET session_started_at = $1
                WHERE id = $2 AND session_started_at IS NULL
                "#,
                &[&now.naive_utc(), &session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to mark session as started: {}", e);
                DatabaseError::QueryFailed(format!("Failed to mark session as started: {}", e))
            })?;

        if rows_affected == 0 {
            info!("Session {} was already marked as started or not found", session_id);
        } else {
            info!("✅ Session {} marked as started at {}", session_id, now);
        }
        
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

    // Session data access methods - now reading from interview_messages table
    pub async fn get_session_questions(&self, session_id: &str) -> Result<Vec<InterviewQuestion>> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let rows = client
            .query(
                r#"
                SELECT id, session_id, content, metadata, timestamp
                FROM interview_messages
                WHERE session_id = $1 AND message_type = 'question'
                ORDER BY timestamp ASC
                "#,
                &[&session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to fetch questions for session {}: {}", session_id, e);
                DatabaseError::QueryFailed(format!("Failed to fetch questions: {}", e))
            })?;

        let mut questions = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let metadata: Option<serde_json::Value> = row.get(3);
            let question_number = metadata
                .as_ref()
                .and_then(|m| m.get("questionNumber"))
                .and_then(|n| n.as_i64())
                .unwrap_or((index + 1) as i64) as i32;
            
            let category = metadata
                .as_ref()
                .and_then(|m| m.get("category"))
                .and_then(|c| c.as_str())
                .unwrap_or("general");
                
            let difficulty = metadata
                .as_ref()
                .and_then(|m| m.get("difficulty"))
                .and_then(|d| d.as_str())
                .unwrap_or("medium");
                
            let expected_duration = metadata
                .as_ref()
                .and_then(|m| m.get("expectedDuration"))
                .and_then(|d| d.as_i64())
                .unwrap_or(30) as i32;

            questions.push(InterviewQuestion {
                id: row.get(0),
                session_id: row.get(1),
                question_number,
                question_text: row.get(2),
                category: category.to_string(),
                difficulty_level: difficulty.to_string(),
                expected_duration,
                asked_at: row.get(4),
                created_at: row.get(4),
            });
        }

        Ok(questions)
    }

    pub async fn get_session_answers(&self, session_id: &str) -> Result<Vec<InterviewAnswer>> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;

        let rows = client
            .query(
                r#"
                SELECT id, parent_message_id, session_id, content, metadata, timestamp
                FROM interview_messages
                WHERE session_id = $1 AND message_type = 'answer'
                ORDER BY timestamp ASC
                "#,
                &[&session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to fetch answers for session {}: {}", session_id, e);
                DatabaseError::QueryFailed(format!("Failed to fetch answers: {}", e))
            })?;

        let mut answers = Vec::new();
        for row in rows {
            let metadata: Option<serde_json::Value> = row.get(4);
            let response_time = metadata
                .as_ref()
                .and_then(|m| m.get("responseTime"))
                .and_then(|r| r.as_i64())
                .map(|r| r as i32);
                
            let ai_feedback = metadata
                .as_ref()
                .and_then(|m| m.get("aiFeedback"))
                .and_then(|f| f.as_str())
                .map(|s| s.to_string());
                
            let ai_score = metadata
                .as_ref()
                .and_then(|m| m.get("aiScore"))
                .and_then(|s| s.as_i64())
                .map(|s| s as i32);

            answers.push(InterviewAnswer {
                id: row.get(0),
                question_id: row.get(1), // parent_message_id is the question ID
                session_id: row.get(2),
                answer_text: Some(row.get(3)),
                response_time,
                ai_feedback,
                ai_score,
                answered_at: row.get(5),
                created_at: row.get(5),
            });
        }

        Ok(answers)
    }

    pub async fn get_session_report(&self, session_id: &str) -> Result<SessionReport> {
        let session = self.get_session_by_id(session_id).await?;
        let user = self.get_user_by_id(&session.user_id.to_string()).await?;
        let questions = self.get_session_questions(session_id).await?;
        let answers = self.get_session_answers(session_id).await?;

        let total_questions = questions.len() as i32;
        let total_answers = answers.len() as i32;
        
        // Calculate averages before moving the vectors
        let average_response_time = if answers.is_empty() {
            0.0
        } else {
            let (total_time, count) = answers.iter()
                .filter_map(|a| a.response_time)
                .map(|t| t as f64)
                .fold((0.0, 0), |(sum, count), time| (sum + time, count + 1));
            if count > 0 { total_time / count as f64 } else { 0.0 }
        };
        
        let average_score = if answers.is_empty() {
            0.0
        } else {
            let (total_score, count) = answers.iter()
                .filter_map(|a| a.ai_score)
                .map(|s| s as f64)
                .fold((0.0, 0), |(sum, count), score| (sum + score, count + 1));
            if count > 0 { total_score / count as f64 } else { 0.0 }
        };

        Ok(SessionReport {
            session,
            user,
            questions,
            answers,
            total_questions,
            total_answers,
            average_response_time,
            average_score,
        })
    }

    pub async fn update_session_final_duration(&self, session_id: &str, total_minutes: i32) -> Result<()> {
        let client = self.pool.get().await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
        
        let session_uuid = Uuid::from_str(session_id)
            .map_err(|_| DatabaseError::SessionNotFound("Invalid session ID format".to_string()))?;
        
        let rows_affected = client
            .execute(
                r#"
                UPDATE sessions 
                SET interview_duration = $1,
                    status = CASE 
                        WHEN status = 'active' THEN 'completed'
                        ELSE status 
                    END
                WHERE id = $2
                "#,
                &[&total_minutes, &session_uuid]
            )
            .await
            .map_err(|e| {
                error!("Failed to update session final duration: {}", e);
                DatabaseError::QueryFailed(format!("Failed to update session final duration: {}", e))
            })?;

        if rows_affected == 0 {
            return Err(DatabaseError::SessionNotFound("Session not found for final duration update".to_string()));
        }

        info!("Updated session {} final duration: {}min and status to completed", session_id, total_minutes);
        Ok(())
    }
}

// Additional data structures for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReport {
    pub session: Session,
    pub user: User,
    pub questions: Vec<InterviewQuestion>,
    pub answers: Vec<InterviewAnswer>,
    pub total_questions: i32,
    pub total_answers: i32,
    pub average_response_time: f64,
    pub average_score: f64,
}

// Tauri commands for database operations
#[tauri::command]
pub async fn test_database_connection() -> std::result::Result<String, String> {
    match DatabaseManager::new().await {
        Ok(db) => {
            db.test_connection().await
                .map_err(|e| e.to_string())
        }
        Err(e) => {
            log::warn!("Database connection test failed: {}", e);
            Ok(format!("Database unavailable: {}", e))
        }
    }
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

#[tauri::command]
pub async fn save_interview_question(
    session_id: String,
    question_number: i32,
    question_text: String,
    category: String,
    difficulty_level: String,
    expected_duration: i32
) -> std::result::Result<String, String> {
    info!("💾 Attempting to save interview question {} for session {}", question_number, session_id);
    
    match DatabaseManager::new().await {
        Ok(db) => {
            match db.insert_interview_question(
                &session_id,
                question_number,
                &question_text,
                &category,
                &difficulty_level,
                expected_duration
            ).await {
                Ok(question_id) => {
                    info!("✅ Question saved with ID: {}", question_id);
                    Ok(question_id.to_string())
                }
                Err(e) => {
                    log::warn!("❌ Failed to save question to database: {}", e);
                    // Generate a fallback UUID for the question
                    let fallback_id = uuid::Uuid::new_v4();
                    log::info!("💡 Using fallback question ID: {}", fallback_id);
                    Ok(fallback_id.to_string())
                }
            }
        }
        Err(e) => {
            log::warn!("❌ Database unavailable for saving question: {}", e);
            log::info!("💡 Database features disabled - generating fallback question ID");
            // Generate a fallback UUID for the question
            let fallback_id = uuid::Uuid::new_v4();
            log::info!("💡 Using fallback question ID: {}", fallback_id);
            Ok(fallback_id.to_string())
        }
    }
}

#[tauri::command]
pub async fn save_interview_answer(
    session_id: String,
    question_id: String,
    answer_text: String,
    response_time: i32,
    ai_feedback: Option<String>,
    ai_score: Option<i32>
) -> std::result::Result<String, String> {
    info!("🔥🔥🔥 BACKEND: save_interview_answer called with params:");
    info!("  📋 session_id: {}", session_id);
    info!("  🆔 question_id: {}", question_id);
    info!("  📝 answer_text length: {}", answer_text.len());
    info!("  📝 answer_text preview (first 200 chars): {}", answer_text.chars().take(200).collect::<String>());
    info!("  ⏱️ response_time: {}", response_time);
    
    match DatabaseManager::new().await {
        Ok(db) => {
            info!("✅ Database connection established successfully");
            
            match Uuid::from_str(&question_id) {
                Ok(question_uuid) => {
                    info!("✅ Question UUID parsed successfully: {}", question_uuid);
                    
                    match db.insert_interview_answer(
                        &question_uuid,
                        &session_id,
                        Some(&answer_text),
                        Some(response_time),
                        ai_feedback.as_deref(),
                        ai_score
                    ).await {
                        Ok(answer_id) => {
                            info!("✅✅✅ SUCCESS! Answer saved with ID: {}", answer_id);
                            info!("✅ Saved answer length: {} characters", answer_text.len());
                            Ok(answer_id.to_string())
                        },
                        Err(e) => {
                            log::error!("❌❌❌ FAILED to insert answer into database: {}", e);
                            log::error!("❌ Failed answer details: session_id={}, question_id={}, answer_length={}", session_id, question_id, answer_text.len());
                            Err(format!("Database insert failed: {}", e))
                        }
                    }
                },
                Err(_) => {
                    log::error!("❌ Invalid question ID format: {}", question_id);
                    Err("Invalid question ID format".to_string())
                }
            }
        },
        Err(e) => {
            log::error!("❌❌❌ FAILED to connect to database: {}", e);
            Err(format!("Database connection failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_session_questions(session_id: String) -> std::result::Result<Vec<InterviewQuestion>, String> {
    info!("📋 Retrieving questions for session: {}", session_id);
    
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    let questions = db.get_session_questions(&session_id).await
        .map_err(|e| e.to_string())?;
    
    info!("✅ Retrieved {} questions", questions.len());
    Ok(questions)
}

#[tauri::command]
pub async fn get_session_answers(session_id: String) -> std::result::Result<Vec<InterviewAnswer>, String> {
    info!("📝 Retrieving answers for session: {}", session_id);
    
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    let answers = db.get_session_answers(&session_id).await
        .map_err(|e| e.to_string())?;
    
    info!("✅ Retrieved {} answers", answers.len());
    Ok(answers)
}

#[tauri::command]
pub async fn get_interview_report(session_id: String) -> std::result::Result<SessionReport, String> {
    info!("📊 Generating interview report for session: {}", session_id);
    
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    let report = db.get_session_report(&session_id).await
        .map_err(|e| e.to_string())?;
    
    info!("✅ Generated report with {} questions, {} answers, avg score: {:.1}", 
          report.total_questions, report.total_answers, report.average_score);
    Ok(report)
}

#[tauri::command]
pub async fn finalize_session_duration(
    session_id: String, 
    total_minutes: i32
) -> std::result::Result<String, String> {
    info!("🏁 Finalizing session {} with duration: {} minutes", session_id, total_minutes);
    
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    db.update_session_final_duration(&session_id, total_minutes).await
        .map_err(|e| e.to_string())?;
    
    info!("✅ Session duration finalized");
    Ok("Session duration finalized successfully".to_string())
}

#[tauri::command]
pub async fn mark_session_started(session_id: String) -> std::result::Result<String, String> {
    info!("🚀 Marking session {} as started", session_id);
    
    let db = DatabaseManager::new().await
        .map_err(|e| e.to_string())?;
    
    db.mark_session_started(&session_id).await
        .map_err(|e| e.to_string())?;
    
    info!("✅ Session marked as started");
    Ok("Session marked as started successfully".to_string())
}
