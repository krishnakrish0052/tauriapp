use mockmate_lib::audio;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging
    env_logger::init();

    println!("=== MockMate Audio Capture Test ===");
    
    // Get device information
    match audio::get_device_info() {
        Ok(info) => println!("{}", info),
        Err(e) => eprintln!("Failed to get device info: {}", e),
    }
    
    // Test audio capture for 5 seconds
    println!("\n=== Testing Audio Capture ===");
    println!("Starting audio capture test for 5 seconds...");
    
    match audio::capture_audio() {
        Ok(_) => {
            println!("✅ Audio capture started successfully!");
            
            // Wait 5 seconds
            thread::sleep(Duration::from_secs(5));
            
            // Check if samples were captured
            let samples = audio::get_captured_samples();
            println!("📊 Captured {} audio samples", samples.len());
            
            if samples.len() > 0 {
                let avg_amplitude: f32 = samples.iter().map(|&s| s.abs()).sum::<f32>() / samples.len() as f32;
                println!("📈 Average amplitude: {:.4}", avg_amplitude);
            }
            
            // Stop capture
            match audio::stop_capture() {
                Ok(_) => println!("✅ Audio capture stopped successfully!"),
                Err(e) => eprintln!("❌ Failed to stop audio capture: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to start audio capture: {}", e);
            eprintln!("   This might be because:");
            eprintln!("   - No microphone is connected");
            eprintln!("   - Microphone permissions are not granted");
            eprintln!("   - Audio drivers are not working properly");
        }
    }
    
    println!("\n=== Test Complete ===");
}
