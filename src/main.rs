//! MirrOx: A Rust-based implementation of scrcp

mod adb;

fn main() {
    println!("Starting scrcpy-rs...");
    
    // Example: Check if ADB is available
    match adb::check_adb() {
        Ok(_) => println!("ADB is available!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
