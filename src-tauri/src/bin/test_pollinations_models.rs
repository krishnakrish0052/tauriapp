use mockmate_lib::pollinations::{PollinationsClient, PollinationsModel};
use mockmate_lib::openai::InterviewContext;
use anyhow::Result;
use log::{info, error, warn};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    info!("🧪 Testing all Pollinations models to identify working ones...");
    
    // Initialize client
    let api_key = std::env::var("POLLINATIONS_API_KEY").unwrap_or_else(|_| "test".to_string());
    let referrer = std::env::var("POLLINATIONS_REFERER").unwrap_or_else(|_| "mockmate".to_string());
    let client = PollinationsClient::new(api_key, referrer);
    
    // Test models list - limited to most reliable models for quick testing
    let test_models = vec![
        "roblox-rp",        // Llama 3.1 8B Instruct (our new default)
        "gemini",           // Gemini 2.5 Flash Lite
        "mistral",          // Mistral Small 3.1 24B
        "nova-fast",        // Amazon Nova Micro
        "openai",           // OpenAI GPT-5 Nano
        "qwen-coder",       // Qwen 2.5 Coder 32B
    ];
    
    let context = InterviewContext {
        company: Some("Test Company".to_string()),
        position: Some("Software Engineer".to_string()),
        job_description: Some("Test role".to_string()),
        user_name: None,
        difficulty_level: Some("medium".to_string()),
        session_type: Some("test".to_string()),
        resume_content: None,
        user_experience_level: Some("mid-level".to_string()),
        interview_style: Some("technical".to_string()),
    };
    
    let test_question = "What is your greatest strength?";
    let mut working_models = Vec::new();
    let mut failed_models = Vec::new();
    
    info!("🚀 Starting model tests with question: '{}'", test_question);
    println!("\n=== POLLINATIONS MODEL TESTING RESULTS ===\n");
    
    let total_models = test_models.len();
    for model_id in &test_models {
        print!("Testing {:<20} ... ", model_id);
        
        match PollinationsModel::from_string(model_id) {
            Ok(model) => {
                // Test with timeout to avoid hanging
                match timeout(Duration::from_secs(15), test_model(&client, &model, test_question, &context)).await {
                    Ok(Ok(response)) => {
                        if response.len() > 10 && !response.to_lowercase().contains("error") {
                            println!("✅ WORKING - Response: {}", response.chars().take(50).collect::<String>() + "...");
                            working_models.push((model_id.to_string(), model.display_name().to_string()));
                        } else {
                            println!("❌ FAILED - Invalid response: {}", response);
                            failed_models.push((model_id.to_string(), format!("Invalid response: {}", response)));
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ FAILED - Error: {}", e);
                        failed_models.push((model_id.to_string(), e.to_string()));
                    }
                    Err(_) => {
                        println!("❌ FAILED - Timeout");
                        failed_models.push((model_id.to_string(), "Timeout".to_string()));
                    }
                }
            }
            Err(e) => {
                println!("❌ FAILED - Model creation error: {}", e);
                failed_models.push((model_id.to_string(), e.to_string()));
            }
        }
        
        // Small delay between tests to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Print summary
    println!("\n=== TESTING SUMMARY ===");
    println!("✅ Working Models ({}):", working_models.len());
    for (id, name) in &working_models {
        println!("  - {} ({})", id, name);
    }
    
    println!("\n❌ Failed Models ({}):", failed_models.len());
    for (id, error) in &failed_models {
        println!("  - {}: {}", id, error);
    }
    
    // Generate Rust code for working models filter
    println!("\n=== RECOMMENDED WORKING MODELS FILTER ===");
    println!("const WORKING_MODELS: &[&str] = &[");
    for (id, _) in &working_models {
        println!("    \"{}\",", id);
    }
    println!("];");
    
    info!("🏁 Model testing completed. {}/{} models working.", working_models.len(), total_models);
    
    Ok(())
}

async fn test_model(
    client: &PollinationsClient,
    model: &PollinationsModel, 
    question: &str,
    context: &InterviewContext
) -> Result<String> {
    client.generate_answer(question, context, model.clone()).await
}
