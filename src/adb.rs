use crate::utils::*;

use std::process::Command;

// Represents a connected ADB device with its state.
#[derive(Debug)]
pub struct AdbDevice {
    pub id: String, // ADB device id
    pub state: String, // device, unauthorized, offline states
    pub connection_type: String, // USB or TCP
    pub manufacture: String, // Manufacturing company of the device
    pub model: String, // Device model name
    pub uptime: String, // Device uptime
    pub battery_level: String, // Device battery %
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
                let connection_type = if device_id.contains(':') { "TCP" } else { "USB" };
                let manufacture = run_shell_command(&device_id, "getprop ro.product.manufacturer").unwrap_or("Unknown".to_string());
                let model = run_shell_command(&device_id, "getprop ro.product.model").unwrap_or("Unknown".to_string());
                let uptime = run_shell_command(&device_id, "uptime").unwrap_or("Unknown".to_string());
                let battery_info = run_shell_command(&device_id, "dumpsys battery").unwrap_or("Unknown".to_string());
                let battery_level = extract_battery_level(&battery_info); // extract_battery_level() is in utils.rs

                devices.push(AdbDevice {
                    id: device_id,
                    state,
                    connection_type: connection_type.to_string(),
                    manufacture,
                    model,
                    uptime,
                    battery_level,
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

// Runs an ADB shell command on a specific device.
pub fn run_shell_command(device_id: &str, command: &str) -> Result<String, String> {
    let output = Command::new("adb")
        .arg("-s")
        .arg(device_id)
        .arg("shell")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute adb shell command: {}", e))?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result.trim().to_string()) // Trim whitespaces
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("ADB shell command failed: {}", error_msg))
    }
}
