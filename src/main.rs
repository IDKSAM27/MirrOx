// //! MirrOx: A Rust-based implementation of scrcp
mod utils;
mod adb;
use crate::adb::*;

fn main() {
    println!("Starting MirrOx...");

    // Check if ADB is available
    if let Err(e) = adb::check_adb() {
        eprintln!("Error: {}", e);
        return;
    }

    // List connected devices
    match adb::list_devices() {
        Ok(devices) => {
            println!("Connected devices:");
            for device in &devices {
                println!("- {} ({}) [{}] | Manufacturer: {} | Model: {}", device.id, device.state, device.connection_type, device.manufacture, device.model);
                println!("Device: {} | Battery: {}% | Uptime: {}", device.model, device.battery_level, device.uptime); //device.battery_level
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    match get_connected_devices() {
        Ok(devices) => {
            if let Some(device_id) = devices.first() {
                println!("Using device: {}", device_id);

                // Push a file from PC to Android
                match adb_push(device_id, "D:/test.txt", "/sdcard/ADB/test.txt") {
                    Ok(_) => println!("Push successful"),
                    Err(e) => println!("Error: {}", e),
                }

                // Pull a file from Android to PC
                match adb_pull(device_id, "/sdcard/ADB/test.txt", "C:/Users/Sampreet/Downloads/file.txt") {
                    Ok(_) => println!("Pull successful"),
                    Err(e) => println!("Error: {}", e),
                }
            } else {
                println!("No devices found.");
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    match select_device() {
        Ok(device_id) => {
            println!("Selected device: {}", device_id);
            // Now you can use devide_id in your ADB commands
            let _ = run_shell_command(&device_id, "echo 'Hello from Rust'");
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
