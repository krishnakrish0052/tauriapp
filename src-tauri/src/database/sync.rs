use super::{DatabaseManager, DatabaseError, Result};
use super::models::*;
use log::{info, error, warn};
use tokio::time::{Duration, interval};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DatabaseSync {
    db: Arc<DatabaseManager>,
    session_id: Option<String>,
    sync_interval: Duration,
    is_running: Arc<Mutex<bool>>,
}

impl DatabaseSync {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            session_id: None,
            sync_interval: Duration::from_secs(30), // Sync every 30 seconds
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start_sync(&mut self, session_id: String) -> Result<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            warn!("Database sync is already running");
            return Ok(());
        }

        info!("Starting database sync for session: {}", session_id);
        self.session_id = Some(session_id.clone());
        *is_running = true;

        let db = self.db.clone();
        let sync_session_id = session_id.clone();
        let sync_interval = self.sync_interval;
        let is_running_clone = self.is_running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(sync_interval);

            loop {
                ticker.tick().await;

                let running = {
                    let guard = is_running_clone.lock().await;
                    *guard
                };

                if !running {
                    info!("Database sync stopped for session: {}", sync_session_id);
                    break;
                }

                if let Err(e) = Self::sync_session_data(&db, &sync_session_id).await {
                    error!("Failed to sync session data: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn stop_sync(&mut self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        self.session_id = None;
        info!("Database sync stopped");
    }

    async fn sync_session_data(db: &DatabaseManager, session_id: &str) -> Result<()> {
        // Update session heartbeat
        db.update_session_heartbeat(session_id).await?;
        
        // Sync any pending local changes
        info!("Synced session data for: {}", session_id);
        Ok(())
    }

    pub async fn sync_question_to_db(
        db: &DatabaseManager,
        session_id: &str,
        question: &InterviewQuestion
    ) -> Result<()> {
        info!("Syncing question to database: {}", question.id);
        
        db.insert_interview_question(
            session_id,
            question.question_number,
            &question.question_text,
            &question.category,
            &question.difficulty_level,
            question.expected_duration
        ).await?;

        Ok(())
    }

    pub async fn sync_answer_to_db(
        db: &DatabaseManager,
        session_id: &str,
        answer: &InterviewAnswer
    ) -> Result<()> {
        info!("Syncing answer to database: {}", answer.id);
        
        db.insert_interview_answer(
            &answer.question_id,
            session_id,
            answer.answer_text.as_deref(),
            answer.response_time,
            answer.ai_feedback.as_deref(),
            answer.ai_score
        ).await?;

        Ok(())
    }
}
