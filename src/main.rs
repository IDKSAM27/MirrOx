//! MirrOx: A Rust-based implementation of scrcp

use std::process::Command;

fn main() {
    println!("Starting scrcpy-rs...");
    
    // Example: Check if ADB is available
    match check_adb() {
        Ok(_) => println!("ADB is available!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

/// Checks if ADB is installed and accessible.
fn check_adb() -> Result<(), String> {
    let output = Command::new("adb")
        .arg("version")
        .output()
        .map_err(|e| format!("Failed to execute adb: {}", e))?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("ADB Version: {}", version);
        Ok(())
    } else {
        Err("ADB command failed".to_string())
    }
}

