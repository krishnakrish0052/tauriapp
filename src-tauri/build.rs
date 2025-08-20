use std::env;

fn main() {
    // Load .env file during build for environment variables
    if let Err(e) = dotenvy::dotenv() {
        println!("cargo:warning=Failed to load .env file: {}. Using system environment variables.", e);
    } else {
        println!("cargo:warning=Successfully loaded .env file for build");
    }
    
    // Export environment variables to be available at runtime
    // These will be embedded in the binary
    if let Ok(deepgram_key) = env::var("DEEPGRAM_API_KEY") {
        println!("cargo:rustc-env=DEEPGRAM_API_KEY={}", deepgram_key);
    }
    
    if let Ok(openai_key) = env::var("OPENAI_API_KEY") {
        println!("cargo:rustc-env=OPENAI_API_KEY={}", openai_key);
    }
    
    if let Ok(pollinations_key) = env::var("POLLINATIONS_API_KEY") {
        println!("cargo:rustc-env=POLLINATIONS_API_KEY={}", pollinations_key);
    }
    
    if let Ok(pollinations_referer) = env::var("POLLINATIONS_REFERER") {
        println!("cargo:rustc-env=POLLINATIONS_REFERER={}", pollinations_referer);
    }
    
    // Deepgram configuration
    if let Ok(model) = env::var("DEEPGRAM_MODEL") {
        println!("cargo:rustc-env=DEEPGRAM_MODEL={}", model);
    }
    
    if let Ok(language) = env::var("DEEPGRAM_LANGUAGE") {
        println!("cargo:rustc-env=DEEPGRAM_LANGUAGE={}", language);
    }
    
    if let Ok(endpointing) = env::var("DEEPGRAM_ENDPOINTING") {
        println!("cargo:rustc-env=DEEPGRAM_ENDPOINTING={}", endpointing);
    }
    
    if let Ok(interim) = env::var("DEEPGRAM_INTERIM_RESULTS") {
        println!("cargo:rustc-env=DEEPGRAM_INTERIM_RESULTS={}", interim);
    }
    
    if let Ok(smart_format) = env::var("DEEPGRAM_SMART_FORMAT") {
        println!("cargo:rustc-env=DEEPGRAM_SMART_FORMAT={}", smart_format);
    }
    
    if let Ok(keep_alive) = env::var("DEEPGRAM_KEEP_ALIVE") {
        println!("cargo:rustc-env=DEEPGRAM_KEEP_ALIVE={}", keep_alive);
    }
    
    if let Ok(punctuate) = env::var("DEEPGRAM_PUNCTUATE") {
        println!("cargo:rustc-env=DEEPGRAM_PUNCTUATE={}", punctuate);
    }
    
    if let Ok(profanity_filter) = env::var("DEEPGRAM_PROFANITY_FILTER") {
        println!("cargo:rustc-env=DEEPGRAM_PROFANITY_FILTER={}", profanity_filter);
    }
    
    if let Ok(diarize) = env::var("DEEPGRAM_DIARIZE") {
        println!("cargo:rustc-env=DEEPGRAM_DIARIZE={}", diarize);
    }
    
    if let Ok(multichannel) = env::var("DEEPGRAM_MULTICHANNEL") {
        println!("cargo:rustc-env=DEEPGRAM_MULTICHANNEL={}", multichannel);
    }
    
    if let Ok(numerals) = env::var("DEEPGRAM_NUMERALS") {
        println!("cargo:rustc-env=DEEPGRAM_NUMERALS={}", numerals);
    }
    
    // Database configuration
    if let Ok(db_host) = env::var("DB_HOST") {
        println!("cargo:rustc-env=DB_HOST={}", db_host);
    }
    
    if let Ok(db_port) = env::var("DB_PORT") {
        println!("cargo:rustc-env=DB_PORT={}", db_port);
    }
    
    if let Ok(db_name) = env::var("DB_NAME") {
        println!("cargo:rustc-env=DB_NAME={}", db_name);
    }
    
    if let Ok(db_user) = env::var("DB_USER") {
        println!("cargo:rustc-env=DB_USER={}", db_user);
    }
    
    if let Ok(db_password) = env::var("DB_PASSWORD") {
        println!("cargo:rustc-env=DB_PASSWORD={}", db_password);
    }
    
    tauri_build::build()
}
