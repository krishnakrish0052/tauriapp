use std::env;

#[tokio::main]
async fn main() {
    println!("🔧 Testing database connection...");
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Print database configuration
    println!("📊 Database Configuration:");
    println!("  DB_HOST: {}", env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()));
    println!("  DB_PORT: {}", env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()));
    println!("  DB_NAME: {}", env::var("DB_NAME").unwrap_or_else(|_| "mockmate_db".to_string()));
    println!("  DB_USER: {}", env::var("DB_USER").unwrap_or_else(|_| "mockmate_user".to_string()));
    println!("  DB_PASSWORD: {}", if env::var("DB_PASSWORD").unwrap_or_default().is_empty() { "<empty>" } else { "***set***" });
    
    // Test basic TCP connection
    println!("\n🔌 Testing TCP connection to database...");
    match tokio::net::TcpStream::connect("localhost:5432").await {
        Ok(_) => println!("✅ TCP connection to localhost:5432 successful"),
        Err(e) => {
            println!("❌ TCP connection failed: {}", e);
            return;
        }
    }
    
    // Test database connection using tokio-postgres directly
    println!("\n🔗 Testing PostgreSQL connection...");
    
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
            println!("✅ PostgreSQL connection successful");
            
            // Spawn the connection task
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });
            
            // Test a simple query
            println!("🧪 Testing simple query...");
            match client.query("SELECT 1 as test_value", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let value: i32 = row.get(0);
                        println!("✅ Simple query successful, got value: {}", value);
                    }
                }
                Err(e) => println!("❌ Simple query failed: {}", e)
            }
            
            // Test if sessions table exists
            println!("🔍 Checking if sessions table exists...");
            match client.query("SELECT COUNT(*) FROM sessions", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let count: i64 = row.get(0);
                        println!("✅ Sessions table exists with {} records", count);
                    }
                }
                Err(e) => println!("❌ Sessions table query failed: {}", e)
            }
            
            // Test if users table exists
            println!("🔍 Checking if users table exists...");
            match client.query("SELECT COUNT(*) FROM users", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let count: i64 = row.get(0);
                        println!("✅ Users table exists with {} records", count);
                    }
                }
                Err(e) => println!("❌ Users table query failed: {}", e)
            }
        }
        Err(e) => {
            println!("❌ PostgreSQL connection failed: {}", e);
            println!("\n💡 Possible solutions:");
            println!("  1. Check if PostgreSQL is running");
            println!("  2. Verify database '{}' exists", db_name);
            println!("  3. Verify user '{}' exists with correct password", db_user);
            println!("  4. Check if user has access to the database");
            return;
        }
    }
    
    println!("\n✅ Database connection test completed!");
}
