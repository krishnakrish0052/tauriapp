use mockmate::database::DatabaseManager;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    println!("🔧 Testing database insertion...");
    
    // Test session ID
    let session_id = "e302a575-1e13-4466-8ae7-7aea024df3ec";
    
    println!("📡 Connecting to database...");
    let db = DatabaseManager::new().await?;
    println!("✅ Connected to database");
    
    println!("💾 Testing question insertion...");
    let question_id = db.insert_interview_question(
        session_id,
        1,
        "What is your experience with Rust programming?",
        "technical",
        "medium",
        300
    ).await?;
    println!("✅ Question inserted with ID: {}", question_id);
    
    println!("💾 Testing answer insertion...");
    let answer_id = db.insert_interview_answer(
        &question_id,
        session_id,
        Some("I have 2 years of experience with Rust, primarily building backend services and CLI tools."),
        Some(45),
        Some("Good answer with specific examples"),
        Some(8)
    ).await?;
    println!("✅ Answer inserted with ID: {}", answer_id);
    
    println!("🎉 All database operations successful!");
    Ok(())
}
