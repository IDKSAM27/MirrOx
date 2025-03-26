//! MirrOx: A Rust-based implementation of scrcp

mod adb;

fn main() {
    println!("Starting scrcpy-rs...");
    
    // Example: Check if ADB is available
    match adb::check_adb() {
        Ok(_) => println!("ADB is available!"),
        Err(e) => eprintln!("Error: {}", e),
    }

    match adb::list_devices() {
        Ok(devices) => {
            println!("Connected devices:");
            for device in devices {
                println!("- {}", device);   // Add the device number if possible
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
