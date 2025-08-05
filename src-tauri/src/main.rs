// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize logging with more verbose output
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    // Print device information before starting the app
    println!("\n=== MockMate Audio Devices ===");
    match mockmate_lib::audio::get_device_info() {
        Ok(info) => println!("{}", info),
        Err(e) => eprintln!("Failed to get device info: {}", e),
    }
    
    // List loopback capable devices specifically
    match mockmate_lib::audio::list_loopback_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("\n⚠️  No loopback-capable devices found.");
                println!("   System audio capture may not work properly.");
                println!("   Consider using a virtual audio cable for system audio capture.");
            } else {
                println!("\n✅ Found {} loopback-capable devices for system audio capture:", devices.len());
                for device in devices {
                    println!("   - {} {}", device.name, if device.is_default { "(Default)" } else { "" });
                }
            }
        }
        Err(e) => eprintln!("Failed to list loopback devices: {}", e),
    }
    
    println!("\n=== Starting MockMate Application ===");
    if let Err(e) = mockmate_lib::run() {
        eprintln!("Error running application: {}", e);
        std::process::exit(1);
    }
}
