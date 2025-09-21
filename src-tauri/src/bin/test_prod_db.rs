use std::env;

#[tokio::main]
async fn main() {
    println!("🔧 Testing production database connection...");
    
    // Production database configuration
    let db_host = "199.192.27.155";
    let db_port = "5432";
    let db_name = "mockmate_db";
    let db_user = "mockmate_user";
    let db_password = "mockmate_2024!";
    
    println!("📊 Production Database Configuration:");
    println!("  DB_HOST: {}", db_host);
    println!("  DB_PORT: {}", db_port);
    println!("  DB_NAME: {}", db_name);
    println!("  DB_USER: {}", db_user);
    println!("  DB_PASSWORD: ***set***");
    
    // Test basic TCP connection
    println!("\n🔌 Testing TCP connection to production database...");
    let connection_target = format!("{}:{}", db_host, db_port);
    match tokio::net::TcpStream::connect(&connection_target).await {
        Ok(_) => println!("✅ TCP connection to {} successful", connection_target),
        Err(e) => {
            println!("❌ TCP connection failed: {}", e);
            println!("💡 This could be due to:");
            println!("   - Firewall blocking the connection");
            println!("   - Database server not accepting external connections");
            println!("   - Network connectivity issues");
            return;
        }
    }
    
    // Test database connection using tokio-postgres directly
    println!("\n🔗 Testing PostgreSQL connection to production...");
    
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        db_host, db_port, db_name, db_user, db_password
    );
    
    match tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await {
        Ok((client, connection)) => {
            println!("✅ PostgreSQL connection to production successful");
            
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
            
            // Test if sessions table exists and count records
            println!("🔍 Checking sessions table in production...");
            match client.query("SELECT COUNT(*) FROM sessions", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let count: i64 = row.get(0);
                        println!("✅ Sessions table exists with {} records in production", count);
                    }
                }
                Err(e) => println!("❌ Sessions table query failed: {}", e)
            }
            
            // Test if users table exists
            println!("🔍 Checking users table in production...");
            match client.query("SELECT COUNT(*) FROM users", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let count: i64 = row.get(0);
                        println!("✅ Users table exists with {} records in production", count);
                    }
                }
                Err(e) => println!("❌ Users table query failed: {}", e)
            }
            
            // Test specific session lookup
            println!("🔍 Testing lookup for session: 5633cc79-66e5-4a1e-b0d8-c23690a4a6ef");
            let session_query = r#"
                SELECT 
                    s.id,
                    s.session_name,
                    s.job_title,
                    s.status,
                    u.first_name,
                    u.last_name
                FROM sessions s
                JOIN users u ON s.user_id = u.id
                WHERE s.id = $1
            "#;
            
            let session_uuid = uuid::Uuid::parse_str("5633cc79-66e5-4a1e-b0d8-c23690a4a6ef")
                .expect("Invalid UUID format");
                
            match client.query(session_query, &[&session_uuid]).await {
                Ok(rows) => {
                    if rows.is_empty() {
                        println!("❌ Session 5633cc79-66e5-4a1e-b0d8-c23690a4a6ef not found in production");
                    } else {
                        let row = &rows[0];
                        let session_name: String = row.get("session_name");
                        let job_title: String = row.get("job_title");
                        let status: String = row.get("status");
                        let first_name: String = row.get("first_name");
                        let last_name: String = row.get("last_name");
                        
                        println!("✅ Session found in production:");
                        println!("   Name: {}", session_name);
                        println!("   Job: {}", job_title);
                        println!("   Status: {}", status);
                        println!("   User: {} {}", first_name, last_name);
                    }
                }
                Err(e) => println!("❌ Session query failed: {}", e)
            }
        }
        Err(e) => {
            println!("❌ PostgreSQL connection to production failed: {}", e);
            println!("\n💡 Possible solutions:");
            println!("  1. Check if the production database accepts external connections");
            println!("  2. Verify firewall/security group settings allow connections on port 5432");
            println!("  3. Confirm the database server is running");
            println!("  4. Check if the credentials are correct");
            return;
        }
    }
    
    println!("\n✅ Production database connection test completed!");
}
