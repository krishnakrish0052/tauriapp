use std::env;

fn main() {
    println!("🔧 Testing environment variable access in built binary...");
    
    // Test embedded environment variables (via env! macro)
    println!("📊 Embedded Environment Variables (from build.rs via cargo:rustc-env):");
    
    println!("✅ DEEPGRAM_API_KEY: {} (length: {})", 
        if env!("DEEPGRAM_API_KEY").is_empty() { "EMPTY" } else { "SET" },
        env!("DEEPGRAM_API_KEY").len()
    );
    
    println!("✅ OPENAI_API_KEY: {} (length: {})", 
        if env!("OPENAI_API_KEY").is_empty() { "EMPTY" } else { "SET" },
        env!("OPENAI_API_KEY").len()
    );
    
    println!("✅ POLLINATIONS_API_KEY: {} (length: {})", 
        if env!("POLLINATIONS_API_KEY").is_empty() { "EMPTY" } else { "SET" },
        env!("POLLINATIONS_API_KEY").len()
    );
    
    println!("✅ DEEPGRAM_MODEL: {}", env!("DEEPGRAM_MODEL"));
    println!("✅ DB_HOST: {}", env!("DB_HOST"));
    
    println!("🔧 Environment variable test complete!");
    
    // Test runtime environment variables (should be empty unless set explicitly)
    println!("📊 Runtime Environment Variables (from system):");
    match env::var("DEEPGRAM_API_KEY") {
        Ok(val) => println!("✅ Runtime DEEPGRAM_API_KEY: SET (length: {})", val.len()),
        Err(_) => println!("❌ Runtime DEEPGRAM_API_KEY: NOT SET")
    }
    
    match env::var("OPENAI_API_KEY") {
        Ok(val) => println!("✅ Runtime OPENAI_API_KEY: SET (length: {})", val.len()),
        Err(_) => println!("❌ Runtime OPENAI_API_KEY: NOT SET")
    }
    
    println!("✅ Environment variable access test successful!");
}
