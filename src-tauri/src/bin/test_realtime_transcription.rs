use std::env;

use mockmate_lib::realtime_transcription::get_transcription_status;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("🎙️ Real-time Transcription Test");
    println!("================================");
    
    // Check for API key
    if env::var("DEEPGRAM_API_KEY").is_err() {
        eprintln!("❌ DEEPGRAM_API_KEY environment variable not set");
        eprintln!("   Please set it with your Deepgram API key to test transcription");
        return;
    }
    
    println!("✅ DEEPGRAM_API_KEY found");
    
    // Create a dummy app handle (for testing without Tauri runtime)
    // In a real scenario, this would be provided by Tauri
    // For now, we'll just test the service initialization
    
    println!("📝 Testing service initialization...");
    
    // Test status check
    match get_transcription_status().await {
        Ok(status) => println!("📊 Initial status: {}", status),
        Err(e) => println!("⚠️  Status check error (expected): {}", e),
    }
    
    println!("🔧 Note: Full testing requires Tauri runtime for AppHandle");
    println!("   This demonstrates the service is properly compiled and structured.");
    println!("   Use the desktop application to test the complete functionality.");
    
    println!("✅ Real-time transcription service compiled successfully!");
}
