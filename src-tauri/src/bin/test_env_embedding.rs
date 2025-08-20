// Test tool to verify if environment variables are embedded in the binary
// This will help us debug the bundle issue

use std::env;

// Helper function to get environment variables with compile-time fallbacks
fn get_env_var(key: &str) -> Option<String> {
    // First try runtime environment variable (for development)
    if let Ok(value) = std::env::var(key) {
        return Some(value);
    }
    
    // Then try compile-time embedded variable (for production builds)
    match key {
        "DEEPGRAM_API_KEY" => option_env!("DEEPGRAM_API_KEY").map(|s| s.to_string()),
        "OPENAI_API_KEY" => option_env!("OPENAI_API_KEY").map(|s| s.to_string()),
        "POLLINATIONS_API_KEY" => option_env!("POLLINATIONS_API_KEY").map(|s| s.to_string()),
        "POLLINATIONS_REFERER" => option_env!("POLLINATIONS_REFERER").map(|s| s.to_string()),
        "DEEPGRAM_MODEL" => option_env!("DEEPGRAM_MODEL").map(|s| s.to_string()),
        "DEEPGRAM_LANGUAGE" => option_env!("DEEPGRAM_LANGUAGE").map(|s| s.to_string()),
        "DEEPGRAM_ENDPOINTING" => option_env!("DEEPGRAM_ENDPOINTING").map(|s| s.to_string()),
        "DEEPGRAM_INTERIM_RESULTS" => option_env!("DEEPGRAM_INTERIM_RESULTS").map(|s| s.to_string()),
        "DEEPGRAM_SMART_FORMAT" => option_env!("DEEPGRAM_SMART_FORMAT").map(|s| s.to_string()),
        "DEEPGRAM_KEEP_ALIVE" => option_env!("DEEPGRAM_KEEP_ALIVE").map(|s| s.to_string()),
        "DEEPGRAM_PUNCTUATE" => option_env!("DEEPGRAM_PUNCTUATE").map(|s| s.to_string()),
        "DEEPGRAM_PROFANITY_FILTER" => option_env!("DEEPGRAM_PROFANITY_FILTER").map(|s| s.to_string()),
        "DEEPGRAM_DIARIZE" => option_env!("DEEPGRAM_DIARIZE").map(|s| s.to_string()),
        "DEEPGRAM_MULTICHANNEL" => option_env!("DEEPGRAM_MULTICHANNEL").map(|s| s.to_string()),
        "DEEPGRAM_NUMERALS" => option_env!("DEEPGRAM_NUMERALS").map(|s| s.to_string()),
        "DB_HOST" => option_env!("DB_HOST").map(|s| s.to_string()),
        "DB_PORT" => option_env!("DB_PORT").map(|s| s.to_string()),
        "DB_NAME" => option_env!("DB_NAME").map(|s| s.to_string()),
        "DB_USER" => option_env!("DB_USER").map(|s| s.to_string()),
        "DB_PASSWORD" => option_env!("DB_PASSWORD").map(|s| s.to_string()),
        _ => None,
    }
}

fn main() {
    println!("🧪 MockMate Environment Variable Embedding Test");
    println!("================================================");
    println!();

    // Check if we're running from a bundle vs direct exe
    let exe_path = env::current_exe().unwrap_or_default();
    println!("📍 Executable path: {:?}", exe_path);
    
    let file_size = std::fs::metadata(&exe_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    println!("📊 Executable size: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
    println!();

    println!("🔍 Environment Variable Status:");
    println!("-------------------------------");

    // Test ALL environment variables from .env file
    let test_vars = [
        "DEEPGRAM_API_KEY",
        "OPENAI_API_KEY", 
        "POLLINATIONS_API_KEY",
        "POLLINATIONS_REFERER",
        "DEEPGRAM_MODEL",
        "DEEPGRAM_LANGUAGE",
        "DEEPGRAM_ENDPOINTING",
        "DEEPGRAM_INTERIM_RESULTS",
        "DEEPGRAM_SMART_FORMAT",
        "DEEPGRAM_KEEP_ALIVE",
        "DEEPGRAM_PUNCTUATE",
        "DEEPGRAM_PROFANITY_FILTER",
        "DEEPGRAM_DIARIZE",
        "DEEPGRAM_MULTICHANNEL",
        "DEEPGRAM_NUMERALS",
        "DB_HOST",
        "DB_PORT",
        "DB_NAME",
        "DB_USER",
        "DB_PASSWORD"
    ];

    let mut embedded_count = 0;
    let mut runtime_count = 0;

    for var_name in &test_vars {
        // Check runtime first
        let runtime_value = env::var(var_name).ok();
        
        // Check embedded
        let embedded_value = get_env_var(var_name);

        let status = match (runtime_value.as_ref(), embedded_value.as_ref()) {
            (Some(runtime), Some(embedded)) if runtime == embedded => {
                runtime_count += 1;
                "✅ Runtime (same as embedded)"
            },
            (Some(_), Some(_)) => {
                runtime_count += 1;
                embedded_count += 1;
                "🔄 Both (different values)"
            },
            (Some(_), None) => {
                runtime_count += 1;
                "🔧 Runtime only"
            },
            (None, Some(_)) => {
                embedded_count += 1;
                "📦 Embedded only"
            },
            (None, None) => "❌ Not available"
        };

        let preview = if let Some(value) = embedded_value.or(runtime_value) {
            if value.len() > 8 {
                format!("{}...{} (length: {})", &value[..4], &value[value.len()-4..], value.len())
            } else if !value.is_empty() {
                format!("*** (length: {})", value.len())
            } else {
                "empty".to_string()
            }
        } else {
            "none".to_string()
        };

        println!("  {}: {} - {}", var_name, status, preview);
    }

    println!();
    println!("📈 Summary:");
    println!("  Runtime variables: {}", runtime_count);
    println!("  Embedded variables: {}", embedded_count);
    
    if embedded_count > 0 {
        println!("✅ Environment variables are embedded in this binary!");
    } else if runtime_count > 0 {
        println!("⚠️ Only runtime environment variables found - embedding may have failed");
    } else {
        println!("❌ No environment variables found - check your .env file and build process");
    }

    println!();
    println!("🎯 Test complete. Use this tool to verify installers work correctly.");
    
    // Wait for user input in release builds
    println!();
    println!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
