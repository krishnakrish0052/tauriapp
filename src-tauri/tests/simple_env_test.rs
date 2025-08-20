use std::env;

fn main() {
    println!("===== Environment Variable Test =====");
    
    // Test compile-time embedded variables (from build.rs)
    println!("Compile-time embedded variables:");
    
    if let Some(deepgram_key) = option_env!("DEEPGRAM_API_KEY") {
        println!("DEEPGRAM_API_KEY (embedded): {}", if deepgram_key.is_empty() { "EMPTY" } else { &deepgram_key[..std::cmp::min(8, deepgram_key.len())] });
    } else {
        println!("DEEPGRAM_API_KEY (embedded): NOT FOUND");
    }
    
    if let Some(openai_key) = option_env!("OPENAI_API_KEY") {
        println!("OPENAI_API_KEY (embedded): {}", if openai_key.is_empty() { "EMPTY" } else { &openai_key[..std::cmp::min(8, openai_key.len())] });
    } else {
        println!("OPENAI_API_KEY (embedded): NOT FOUND");
    }
    
    if let Some(pollinations_key) = option_env!("POLLINATIONS_API_KEY") {
        println!("POLLINATIONS_API_KEY (embedded): {}", if pollinations_key.is_empty() { "EMPTY" } else { &pollinations_key[..std::cmp::min(8, pollinations_key.len())] });
    } else {
        println!("POLLINATIONS_API_KEY (embedded): NOT FOUND");
    }
    
    // Test runtime variables
    println!("\nRuntime environment variables:");
    
    match env::var("DEEPGRAM_API_KEY") {
        Ok(value) => println!("DEEPGRAM_API_KEY (runtime): {}", if value.is_empty() { "EMPTY" } else { &value[..std::cmp::min(8, value.len())] }),
        Err(_) => println!("DEEPGRAM_API_KEY (runtime): NOT FOUND"),
    }
    
    match env::var("OPENAI_API_KEY") {
        Ok(value) => println!("OPENAI_API_KEY (runtime): {}", if value.is_empty() { "EMPTY" } else { &value[..std::cmp::min(8, value.len())] }),
        Err(_) => println!("OPENAI_API_KEY (runtime): NOT FOUND"),
    }
    
    match env::var("POLLINATIONS_API_KEY") {
        Ok(value) => println!("POLLINATIONS_API_KEY (runtime): {}", if value.is_empty() { "EMPTY" } else { &value[..std::cmp::min(8, value.len())] }),
        Err(_) => println!("POLLINATIONS_API_KEY (runtime): NOT FOUND"),
    }
    
    println!("\n===== Test Complete =====");
}
