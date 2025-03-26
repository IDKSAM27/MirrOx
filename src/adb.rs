use std::process::Command;

// Represents a connected ADB device with its state.
#[derive(Debug)]
pub struct AdbDevice {
    pub id: String,
    pub state: String,
    pub connection_type: String, // USB or TCP
}

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

/// Lists connected ADB devices with state and connection type (USB or TCP).
pub fn list_devices() -> Result<Vec<AdbDevice>, String> {   // fn function() -> Result<T, E>    where, Ok(T) if success
                                                            //                                         Err(E) if fails
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .map_err(|e| format!("Failed to execute adb: {}", e))?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // Parse ADB output
        for line in output_str.lines().skip(1) { // Skip "List of devices attached"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let device_id = parts[0].to_string();
                let state = parts[1].to_string();

                // Determine connection type
                let connection_type = if device_id.contains(':') {
                    "TCP" // Device ID contains ":" → It's a TCP (wireless) device
                } else {
                    "USB" // Otherwise, it's a USB device
                };

                devices.push(AdbDevice {
                    id: device_id,
                    state,
                    connection_type: connection_type.to_string(),
                });
            }
        }

        if devices.is_empty() {
            Err("No devices found.".to_string())
        } else {
            Ok(devices)
        }
    } else {
        Err("Failed to retrieve devices.".to_string())
    }
}


