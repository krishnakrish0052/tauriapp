use std::env;

#[tokio::main]
async fn main() {
    println!("🔧 Listing all sessions in database...");
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_name = env::var("DB_NAME").unwrap_or_else(|_| "mockmate_db".to_string());
    let db_user = env::var("DB_USER").unwrap_or_else(|_| "mockmate_user".to_string());
    let db_password = env::var("DB_PASSWORD").unwrap_or_default();
    
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        db_host, db_port, db_name, db_user, db_password
    );
    
    match tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await {
        Ok((client, connection)) => {
            println!("✅ Connected to database");
            
            // Spawn the connection task
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });
            
            // List all sessions with user information
            let query = r#"
                SELECT 
                    s.id,
                    s.session_name,
                    s.job_title,
                    s.status,
                    s.desktop_connected,
                    s.created_at,
                    u.email,
                    u.first_name,
                    u.last_name
                FROM sessions s
                JOIN users u ON s.user_id = u.id
                ORDER BY s.created_at DESC
            "#;
            
            match client.query(query, &[]).await {
                Ok(rows) => {
                    println!("\n📋 Found {} sessions:", rows.len());
                    println!("{:-<120}", "");
                    println!("{:<40} {:<30} {:<20} {:<10} {:<15}", "Session ID", "Session Name", "Job Title", "Status", "User");
                    println!("{:-<120}", "");
                    
                    let mut first_session_id: Option<uuid::Uuid> = None;
                    
                    for row in &rows {
                        let session_id: uuid::Uuid = row.get("id");
                        let session_name: String = row.get("session_name");
                        let job_title: String = row.get("job_title");
                        let status: String = row.get("status");
                        let desktop_connected: bool = row.get("desktop_connected");
                        let first_name: String = row.get("first_name");
                        let last_name: String = row.get("last_name");
                        let _email: String = row.get("email");
                        
                        if first_session_id.is_none() {
                            first_session_id = Some(session_id);
                        }
                        
                        let connection_status = if desktop_connected { "🔗" } else { "❌" };
                        let user_name = format!("{} {}", first_name, last_name);
                        
                        println!("{} {:<38} {:<30} {:<20} {:<10} {:<15}", 
                                connection_status,
                                session_id.to_string(), 
                                session_name.chars().take(28).collect::<String>(), 
                                job_title.chars().take(18).collect::<String>(),
                                status,
                                user_name.chars().take(13).collect::<String>());
                    }
                    println!("{:-<120}", "");
                    
                    if !rows.is_empty() {
                        println!("\n💡 You can test the connect_session function with any of the Session IDs above.");
                        if let Some(session_id) = first_session_id {
                            println!("   For example: connect_session(\"{}\")", session_id);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to query sessions: {}", e)
            }
        }
        Err(e) => println!("❌ Database connection failed: {}", e)
    }
}
