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

pub fn list_devices() -> Result<Vec<String>, String> {  // fn function() -> Result<T, E>    where, Ok(T) if success
                                                        //                                         Err(E) if fails
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .map_err(|e| format!("Failed to execute adb: {}", e))?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // Parse ADB output
        for line in output_str.lines().skip(1) { // Skip the first line "List of devices attached"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                devices.push(parts[0].to_string()); // Device ID
            }
        }
        if devices.is_empty() {
            Err("No devices found.".to_string())
        } else {
            Ok(devices)
        }
    }
    else {
        Err("Failed to retrieve devices.".to_string())
    }
}