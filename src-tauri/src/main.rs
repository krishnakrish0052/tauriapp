// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize logging with info level output
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    
    println!("=== Starting MockMate Application ===");
    if let Err(e) = mockmate_lib::run() {
        eprintln!("Error running application: {}", e);
        std::process::exit(1);
    }
}
