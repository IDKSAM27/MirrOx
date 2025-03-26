use std::process::Command;

// Checks if ADB is installed and accessible.
pub fn check_adb() -> Result<(), String> {
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
