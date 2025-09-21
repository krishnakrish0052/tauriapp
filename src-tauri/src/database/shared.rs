use once_cell::sync::Lazy;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};
use crate::get_env_var;

// Database connection pool - shared globally
pub static DATABASE_POOL: Lazy<Pool> = Lazy::new(|| {
    let mut cfg = Config::new();
    
    // Read from embedded environment variables (with runtime fallbacks)
    cfg.host = Some(get_env_var("DB_HOST").unwrap_or_else(|| "localhost".to_string()));
    cfg.port = Some(get_env_var("DB_PORT").unwrap_or_else(|| "5432".to_string()).parse().unwrap_or(5432));
    cfg.dbname = Some(get_env_var("DB_NAME").unwrap_or_else(|| "mockmate_db".to_string()));
    cfg.user = Some(get_env_var("DB_USER").unwrap_or_else(|| "mockmate_user".to_string()));
    cfg.password = Some(get_env_var("DB_PASSWORD").unwrap_or_else(|| "".to_string()));

    // Log the database configuration for debugging
    log::info!("📊 Database Configuration:");
    log::info!("  Host: {}", cfg.host.as_ref().unwrap_or(&"<none>".to_string()));
    log::info!("  Port: {}", cfg.port.unwrap_or(0));
    log::info!("  Database: {}", cfg.dbname.as_ref().unwrap_or(&"<none>".to_string()));
    log::info!("  User: {}", cfg.user.as_ref().unwrap_or(&"<none>".to_string()));
    log::info!("  Password: {}", if cfg.password.as_ref().map(|p| !p.is_empty()).unwrap_or(false) { "***set***" } else { "<empty>" });

    cfg.create_pool(Some(Runtime::Tokio1), NoTls).expect("Failed to create database pool")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_name: String,
    pub company_name: Option<String>,
    pub job_title: String,
    pub job_description: Option<String>,
    pub status: String,
    pub desktop_connected: bool,
    pub websocket_connection_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_duration_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub google_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub name: String, // Computed field: first_name + " " + last_name
    pub avatar_url: Option<String>,
    pub credits: i32,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWithUser {
    // Session fields
    pub session_id: Uuid,
    pub session_name: String,
    pub company_name: Option<String>,
    pub job_title: String,
    pub job_description: Option<String>,
    pub status: String,
    pub desktop_connected: bool,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    
    // User fields
    pub user_details: UserInfo,
    
    // Interview configuration
    pub interview_config: InterviewConfig,
    
    // Credits available
    pub credits_available: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewConfig {
    pub job_title: String,
    pub company_name: Option<String>,
    pub difficulty: String, // Default to "Medium"
}

pub async fn get_session_with_user_info(session_id: &str) -> Result<SessionWithUser, String> {
    let pool = &*DATABASE_POOL;
    let client = pool.get().await.map_err(|e| format!("Database connection error: {}", e))?;
    
    // Parse session ID as UUID
    let session_uuid = Uuid::parse_str(session_id)
        .map_err(|_| "Invalid session ID format".to_string())?;
    
    // Query to get session with user information
    let query = r#"
        SELECT 
            s.id as session_id,
            s.session_name,
            s.company_name,
            s.job_title,
            s.job_description,
            s.status,
            s.desktop_connected,
            s.created_at,
            s.started_at,
            u.id as user_id,
            u.first_name,
            u.last_name,
            u.email,
            u.avatar_url,
            u.credits
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.id = $1
    "#;
    
    let rows = client.query(query, &[&session_uuid]).await.map_err(|e| {
        log::error!("Database query failed: {}", e);
        format!("Database query error: {}", e)
    })?;
    
    if rows.is_empty() {
        log::warn!("Session not found: {}", session_id);
        return Err("Session not found".to_string());
    }
    
    if rows.len() > 1 {
        log::error!("Multiple sessions found for ID {}: {} rows returned", session_id, rows.len());
        return Err(format!("Multiple sessions found for ID {} (database consistency error)", session_id));
    }
    
    let row = &rows[0];
    
    let first_name: String = row.get("first_name");
    let last_name: String = row.get("last_name");
    let name = format!("{} {}", first_name, last_name);
    
    // Convert NaiveDateTime to DateTime<Utc> for timestamps
    let created_at_naive: NaiveDateTime = row.get("created_at");
    let started_at_naive: Option<NaiveDateTime> = row.get("started_at");
    
    let session_with_user = SessionWithUser {
        session_id: row.get("session_id"),
        session_name: row.get("session_name"),
        company_name: row.get("company_name"),
        job_title: row.get("job_title"),
        job_description: row.get("job_description"),
        status: row.get("status"),
        desktop_connected: row.get("desktop_connected"),
        created_at: Utc.from_utc_datetime(&created_at_naive),
        started_at: started_at_naive.map(|dt| Utc.from_utc_datetime(&dt)),
        
        user_details: UserInfo {
            name: name.clone(),
            email: row.get("email"),
            avatar_url: row.get("avatar_url"),
        },
        
        interview_config: InterviewConfig {
            job_title: row.get("job_title"),
            company_name: row.get("company_name"),
            difficulty: "Medium".to_string(), // Default difficulty
        },
        
        credits_available: row.get("credits"),
    };
    
    Ok(session_with_user)
}

pub async fn activate_session(session_id: &str) -> Result<(), String> {
    let pool = &*DATABASE_POOL;
    let mut client = pool.get().await.map_err(|e| format!("Database connection error: {}", e))?;
    
    let session_uuid = Uuid::parse_str(session_id)
        .map_err(|_| "Invalid session ID format".to_string())?;
    
    // Start a transaction
    let transaction = client.transaction().await.map_err(|e| format!("Transaction error: {}", e))?;
    
    // First, check if the session exists and is in the correct state
    let check_query = "SELECT status, user_id FROM sessions WHERE id = $1";
    let session_row = transaction.query_one(check_query, &[&session_uuid]).await.map_err(|e| {
        match e.code() {
            Some(&tokio_postgres::error::SqlState::NO_DATA_FOUND) => "Session not found".to_string(),
            _ => format!("Database query error: {}", e)
        }
    })?;
    
    let current_status: String = session_row.get("status");
    let user_id: Uuid = session_row.get("user_id");
    
    if current_status != "created" && current_status != "active" {
        return Err("Session cannot be activated".to_string());
    }
    
    // Check if user has sufficient credits
    let credits_query = "SELECT credits FROM users WHERE id = $1";
    let user_row = transaction.query_one(credits_query, &[&user_id]).await
        .map_err(|e| format!("Failed to check user credits: {}", e))?;
    
    let current_credits: i32 = user_row.get("credits");
    
    if current_credits < 1 {
        return Err("Insufficient credits to activate session".to_string());
    }
    
    // Update session status to active and set started_at
    let update_session_query = r#"
        UPDATE sessions 
        SET status = 'active', 
            started_at = NOW(), 
            desktop_connected = true
        WHERE id = $1
    "#;
    
    transaction.execute(update_session_query, &[&session_uuid]).await
        .map_err(|e| format!("Failed to activate session: {}", e))?;
    
    // Deduct 1 credit from user
    let deduct_credits_query = "UPDATE users SET credits = credits - 1 WHERE id = $1";
    transaction.execute(deduct_credits_query, &[&user_id]).await
        .map_err(|e| format!("Failed to deduct credits: {}", e))?;
    
    // Record the credit transaction
    let transaction_query = r#"
        INSERT INTO credit_transactions (user_id, session_id, transaction_type, credits_amount, description)
        VALUES ($1, $2, 'usage', -1, 'Session activation')
    "#;
    
    transaction.execute(transaction_query, &[&user_id, &session_uuid]).await
        .map_err(|e| format!("Failed to record credit transaction: {}", e))?;
    
    // Commit the transaction
    transaction.commit().await.map_err(|e| format!("Transaction commit error: {}", e))?;
    
    Ok(())
}

pub async fn disconnect_session(session_id: &str) -> Result<(), String> {
    let pool = &*DATABASE_POOL;
    let client = pool.get().await.map_err(|e| format!("Database connection error: {}", e))?;
    
    let session_uuid = Uuid::parse_str(session_id)
        .map_err(|_| "Invalid session ID format".to_string())?;
    
    let query = r#"
        UPDATE sessions 
        SET desktop_connected = false,
            websocket_connection_id = NULL
        WHERE id = $1
    "#;
    
    client.execute(query, &[&session_uuid]).await
        .map_err(|e| format!("Failed to disconnect session: {}", e))?;
    
    Ok(())
}

pub async fn get_session_info(session_id: &str) -> Result<Session, String> {
    let pool = &*DATABASE_POOL;
    let client = pool.get().await.map_err(|e| format!("Database connection error: {}", e))?;
    
    let session_uuid = Uuid::parse_str(session_id)
        .map_err(|_| "Invalid session ID format".to_string())?;
    
    let query = "SELECT * FROM sessions WHERE id = $1";
    let row = client.query_one(query, &[&session_uuid]).await.map_err(|e| {
        match e.code() {
            Some(&tokio_postgres::error::SqlState::NO_DATA_FOUND) => "Session not found".to_string(),
            _ => format!("Database query error: {}", e)
        }
    })?;
    
    // Convert NaiveDateTime to DateTime<Utc> for timestamps
    let created_at_naive: NaiveDateTime = row.get("created_at");
    let started_at_naive: Option<NaiveDateTime> = row.get("started_at");
    let ended_at_naive: Option<NaiveDateTime> = row.get("ended_at");
    
    let session = Session {
        id: row.get("id"),
        user_id: row.get("user_id"),
        session_name: row.get("session_name"),
        company_name: row.get("company_name"),
        job_title: row.get("job_title"),
        job_description: row.get("job_description"),
        status: row.get("status"),
        desktop_connected: row.get("desktop_connected"),
        websocket_connection_id: row.get("websocket_connection_id"),
        created_at: Utc.from_utc_datetime(&created_at_naive),
        started_at: started_at_naive.map(|dt| Utc.from_utc_datetime(&dt)),
        ended_at: ended_at_naive.map(|dt| Utc.from_utc_datetime(&dt)),
        total_duration_minutes: row.get("total_duration_minutes"),
    };
    
    Ok(session)
}

// Initialize database on app startup (optional - gracefully handles failures)
pub async fn initialize_database() -> Result<(), String> {
    // Load environment variables
    dotenvy::dotenv().ok(); // Don't fail if .env doesn't exist
    
    // Test the connection - but make it optional for development
    let pool = &*DATABASE_POOL;
    match pool.get().await {
        Ok(client) => {
            // Try a simple ping query
            match client.query_one("SELECT 1 as ping", &[]).await {
                Ok(_) => {
                    log::info!("✅ Successfully connected to PostgreSQL database");
                    Ok(())
                }
                Err(e) => {
                    log::warn!("⚠️ Database ping failed: {}", e);
                    log::warn!("💡 Database features will be disabled. App will continue without database.");
                    Ok(()) // Don't fail the app if database is not available
                }
            }
        }
        Err(e) => {
            log::warn!("⚠️ Failed to connect to database: {}", e);
            log::warn!("💡 Database features will be disabled. App will continue without database.");
            log::info!("🔧 To enable database features, ensure PostgreSQL is running with:");
            log::info!("   Host: {}", get_env_var("DB_HOST").unwrap_or_else(|| "localhost".to_string()));
            log::info!("   Port: {}", get_env_var("DB_PORT").unwrap_or_else(|| "5432".to_string()));
            log::info!("   Database: {}", get_env_var("DB_NAME").unwrap_or_else(|| "mockmate_db".to_string()));
            log::info!("   User: {}", get_env_var("DB_USER").unwrap_or_else(|| "mockmate_user".to_string()));
            Ok(()) // Don't fail the app if database is not available
        }
    }
}
