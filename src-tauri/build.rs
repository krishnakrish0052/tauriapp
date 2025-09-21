use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:warning=BUILD.RS: Starting build process...");
    println!("cargo:warning=BUILD.RS: Current working directory: {:?}", env::current_dir().unwrap_or_default());
    
    // Load .env file during build for environment variables
    if let Err(e) = dotenvy::dotenv() {
        println!("cargo:warning=BUILD.RS: Failed to load .env file: {}. Using system environment variables.", e);
    } else {
        println!("cargo:warning=BUILD.RS: Successfully loaded .env file for build");
    }
    
    // Also try to load from parent directory .env (for desktop-app/.env)
    let parent_env = Path::new("../.env");
    if parent_env.exists() {
        if let Ok(contents) = fs::read_to_string(parent_env) {
            println!("cargo:warning=Found parent .env file, parsing manually");
            // Parse .env file manually since dotenvy might not find it
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    // Remove inline comments from the value
                    let clean_value = if let Some(comment_pos) = value.find('#') {
                        value[..comment_pos].trim()
                    } else {
                        value
                    };
                    
                    env::set_var(key, clean_value);
                    println!("cargo:warning=BUILD.RS: Set {}={}", key, clean_value);
                }
            }
        }
    }
    
    // Export environment variables to be available at runtime using cargo:rustc-env
    // These will be embedded in the binary at compile time
    if let Ok(deepgram_key) = env::var("DEEPGRAM_API_KEY") {
        println!("cargo:rustc-env=DEEPGRAM_API_KEY={}", deepgram_key);
        println!("cargo:warning=Embedded DEEPGRAM_API_KEY (length: {})", deepgram_key.len());
    } else {
        println!("cargo:warning=DEEPGRAM_API_KEY not found in environment during build");
    }
    
    if let Ok(openai_key) = env::var("OPENAI_API_KEY") {
        println!("cargo:rustc-env=OPENAI_API_KEY={}", openai_key);
        println!("cargo:warning=Embedded OPENAI_API_KEY (length: {})", openai_key.len());
    } else {
        println!("cargo:warning=OPENAI_API_KEY not found in environment during build");
    }
    
    if let Ok(pollinations_key) = env::var("POLLINATIONS_API_KEY") {
        println!("cargo:rustc-env=POLLINATIONS_API_KEY={}", pollinations_key);
        println!("cargo:warning=Embedded POLLINATIONS_API_KEY (length: {})", pollinations_key.len());
    } else {
        println!("cargo:warning=POLLINATIONS_API_KEY not found in environment during build");
    }
    
    if let Ok(pollinations_referer) = env::var("POLLINATIONS_REFERER") {
        println!("cargo:rustc-env=POLLINATIONS_REFERER={}", pollinations_referer);
        println!("cargo:warning=Embedded POLLINATIONS_REFERER (length: {})", pollinations_referer.len());
    } else {
        println!("cargo:warning=POLLINATIONS_REFERER not found in environment during build");
    }
    
    // Deepgram configuration
    if let Ok(model) = env::var("DEEPGRAM_MODEL") {
        println!("cargo:rustc-env=DEEPGRAM_MODEL={}", model);
        println!("cargo:warning=Embedded DEEPGRAM_MODEL ({})", model);
    } else {
        println!("cargo:warning=DEEPGRAM_MODEL not found in environment during build");
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
    
    // Only run tauri_build if we're building the main application, not test binaries
    let pkg_name = env::var("CARGO_PKG_NAME").unwrap_or_default();
    let bin_name = env::var("CARGO_BIN_NAME").unwrap_or_default();
    println!("cargo:warning=BUILD.RS: Package name: {}, Binary name: {}", pkg_name, bin_name);
    
    if !bin_name.contains("simple_env_test") {
        println!("cargo:warning=BUILD.RS: Running tauri_build for binary: {}", bin_name);
        tauri_build::build()
    } else {
        println!("cargo:warning=BUILD.RS: Skipping tauri_build for test binary: {}", bin_name);
    }
}
