use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};
use log::{info, error, warn};
use crate::database::DatabaseManager;
use crate::session::sync::sync_session_to_web_db;

#[derive(Serialize, Deserialize, Clone)]
pub struct TimerState {
    pub session_id: String,
    pub elapsed_seconds: u64,
    pub elapsed_minutes: u64,
    pub credits_used: u32,
    pub is_running: bool,
    pub started_at: u64,
    pub paused_duration: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CreditUsage {
    pub session_id: String,
    pub total_minutes: u64,
    pub credits_used: u32,
    pub cost_breakdown: Vec<CreditBlock>,
}

#[derive(Serialize, Deserialize)]
pub struct CreditBlock {
    pub start_minute: u64,
    pub end_minute: u64,
    pub credits: u32,
}

pub struct InterviewTimer {
    session_id: String,
    start_time: Instant,
    paused_time: Duration,
    is_running: bool,
    is_paused: bool,
    credit_rate: u64, // minutes per credit (60)
    last_credit_sync: Instant,
}

impl InterviewTimer {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            start_time: Instant::now(),
            paused_time: Duration::new(0, 0),
            is_running: false,
            is_paused: false,
            credit_rate: 60, // 60 minutes = 1 credit
            last_credit_sync: Instant::now(),
        }
    }
    
    pub fn start(&mut self) {
        if !self.is_running {
            self.start_time = Instant::now();
            self.is_running = true;
            self.is_paused = false;
            self.last_credit_sync = Instant::now();
            info!("⏱️ Timer started for session: {}", self.session_id);
        } else if self.is_paused {
            // Resume from pause
            let pause_duration = Instant::now() - self.last_credit_sync;
            self.paused_time += pause_duration;
            self.is_paused = false;
            info!("▶️ Timer resumed for session: {}", self.session_id);
        }
    }
    
    pub fn pause(&mut self) {
        if self.is_running && !self.is_paused {
            self.is_paused = true;
            self.last_credit_sync = Instant::now();
            info!("⏸️ Timer paused for session: {}", self.session_id);
        }
    }
    
    pub fn stop(&mut self) -> TimerState {
        self.is_running = false;
        self.is_paused = false;
        let state = self.get_current_state();
        info!("⏹️ Timer stopped for session: {} ({}min, {} credits)", 
              self.session_id, state.elapsed_minutes, state.credits_used);
        state
    }
    
    pub fn get_current_state(&self) -> TimerState {
        let total_elapsed = if self.is_running {
            let raw_elapsed = self.start_time.elapsed();
            raw_elapsed - self.paused_time
        } else {
            Duration::new(0, 0)
        };
        
        let elapsed_seconds = total_elapsed.as_secs();
        let elapsed_minutes = elapsed_seconds / 60;
        let credits_used = self.calculate_credits_used();
        
        TimerState {
            session_id: self.session_id.clone(),
            elapsed_seconds,
            elapsed_minutes,
            credits_used,
            is_running: self.is_running && !self.is_paused,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - elapsed_seconds,
            paused_duration: self.paused_time.as_secs(),
        }
    }
    
    pub fn calculate_credits_used(&self) -> u32 {
        if !self.is_running {
            return 1; // Minimum 1 credit when session starts
        }
        
        let elapsed_minutes = self.get_elapsed_minutes();
        
        // Always charge minimum 1 credit, then additional credits for every 60 minutes
        if elapsed_minutes == 0 {
            1 // Initial credit charged when session starts
        } else {
            1 + (elapsed_minutes / self.credit_rate) as u32
        }
    }
    
    pub fn get_elapsed_minutes(&self) -> u64 {
        if self.is_running {
            let total_elapsed = self.start_time.elapsed() - self.paused_time;
            total_elapsed.as_secs() / 60
        } else {
            0
        }
    }
    
    pub fn get_credit_usage_breakdown(&self) -> CreditUsage {
        let total_minutes = self.get_elapsed_minutes();
        let credits_used = self.calculate_credits_used();
        
        let mut blocks = vec![
            CreditBlock {
                start_minute: 0,
                end_minute: 1.min(total_minutes),
                credits: 1, // Initial credit
            }
        ];
        
        // Add additional credit blocks for every 60 minutes
        if total_minutes > 60 {
            let additional_blocks = (total_minutes / 60) as u32;
            for i in 1..=additional_blocks {
                blocks.push(CreditBlock {
                    start_minute: i as u64 * 60,
                    end_minute: ((i + 1) as u64 * 60).min(total_minutes),
                    credits: 1,
                });
            }
        }
        
        CreditUsage {
            session_id: self.session_id.clone(),
            total_minutes,
            credits_used,
            cost_breakdown: blocks,
        }
    }
    
    // Sync credits with database every 5 minutes or when significant usage occurs
    pub async fn sync_credits_if_needed(&mut self) -> Result<(), String> {
        let elapsed_since_sync = self.last_credit_sync.elapsed();
        
        if elapsed_since_sync >= Duration::from_secs(300) { // 5 minutes
            let credits_used = self.calculate_credits_used();
            let elapsed_minutes = self.get_elapsed_minutes() as u32;
            
            // Sync with database
            sync_session_to_web_db(&self.session_id, elapsed_minutes, credits_used as i32).await?;
            
            self.last_credit_sync = Instant::now();
            info!("💾 Credits synced: {} credits for {} minutes", credits_used, elapsed_minutes);
        }
        
        Ok(())
    }
}

// Tauri commands for frontend

#[tauri::command]
pub async fn start_interview_timer(session_id: String) -> Result<TimerState, String> {
    info!("⏱️ Starting interview timer for session: {}", session_id);
    
    let timer_store = crate::session::get_timer_store().await;
    let mut timers = timer_store.lock();
    
    if let Some(timer) = timers.get_mut(&session_id) {
        timer.start();
        Ok(timer.get_current_state())
    } else {
        let mut new_timer = InterviewTimer::new(session_id.clone());
        new_timer.start();
        let state = new_timer.get_current_state();
        timers.insert(session_id, new_timer);
        Ok(state)
    }
}

#[tauri::command]
pub async fn pause_interview_timer(session_id: String) -> Result<TimerState, String> {
    info!("⏸️ Pausing interview timer for session: {}", session_id);
    
    let timer_store = crate::session::get_timer_store().await;
    let mut timers = timer_store.lock();
    
    if let Some(timer) = timers.get_mut(&session_id) {
        timer.pause();
        Ok(timer.get_current_state())
    } else {
        Err("Timer not found for session".to_string())
    }
}

#[tauri::command]
pub async fn stop_interview_timer(session_id: String) -> Result<TimerState, String> {
    info!("⏹️ Stopping interview timer for session: {}", session_id);
    
    let timer_store = crate::session::get_timer_store().await;
    
    // Extract and remove the timer from the store
    let mut timer = {
        let mut timers = timer_store.lock();
        timers.remove(&session_id)
    };
    
    if let Some(ref mut timer) = timer {
        // Stop the timer and get final state
        let final_state = timer.stop();
        
        // Final sync with database (no lock held)
        let _ = timer.sync_credits_if_needed().await;
        
        // Update session status to completed and finalize duration
        let db = DatabaseManager::new().await
            .map_err(|e| format!("Database connection failed: {}", e))?;
        
        // Update session with final duration and status
        db.update_session_final_duration(&session_id, final_state.elapsed_minutes as i32).await
            .map_err(|e| format!("Failed to update session final duration: {}", e))?;
        
        info!("✅ Session {} finalized with {} minutes total duration", session_id, final_state.elapsed_minutes);
        
        Ok(final_state)
    } else {
        Err("Timer not found for session".to_string())
    }
}

#[tauri::command]
pub async fn get_timer_state(session_id: String) -> Result<TimerState, String> {
    let timer_store = crate::session::get_timer_store().await;
    let timers = timer_store.lock();
    
    if let Some(timer) = timers.get(&session_id) {
        Ok(timer.get_current_state())
    } else {
        Err("Timer not found for session".to_string())
    }
}

#[tauri::command]
pub async fn get_credit_usage(session_id: String) -> Result<CreditUsage, String> {
    let timer_store = crate::session::get_timer_store().await;
    let timers = timer_store.lock();
    
    if let Some(timer) = timers.get(&session_id) {
        Ok(timer.get_credit_usage_breakdown())
    } else {
        Err("Timer not found for session".to_string())
    }
}
