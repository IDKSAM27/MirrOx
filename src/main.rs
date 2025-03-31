// //! MirrOx: A Rust-based implementation of scrcp
mod utils;
mod adb;
mod video;
use crate::adb::*;
// use crate::video::*;

fn main() {
    env_logger::init();
    println!("Starting MirrOx...");

    // Check if ADB is available
    if let Err(e) = adb::check_adb() {
        log::error!("ADB check failed: {}", e);
        return;
    }

    // List connected devices
    match adb::list_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                log::error!("No devices found.");
                return; // Exit early if no devices are found
            }
            println!("Connected devices:");
            for device in &devices {
                println!(
                    "- {} ({}) [{}] | Manufacturer: {} | Model: {}",
                    device.id, device.state, device.connection_type, device.manufacture, device.model
                );
                println!(
                    "Device: {} | Battery: {}% | Uptime: {}\n",
                    device.model, device.battery_level, device.uptime
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            return; // Exit early if list_devices() fails
        }
    }

    match adb::say_hello_from_device() {
        Ok(_) => println!("Message sent successfully.\n"),
        Err(e) => eprintln!("2Error: {}", e),
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

    // Capture the screenshot, also saves it as screenshot.png
    // if let Ok(selected_device) = adb::select_device() {
    //     if let Err(e) = adb::capture_screen(&selected_device.id, "screenshot.png") {
    //         log::error!("Error capturing screen: {}", e);
    //     }
    // } else {
    //     log::error!("Failed to select device.");
    // }

    match adb::select_device() {
        Ok(selected_device) => {
            println!("Selected device: {} ({})", selected_device.id, selected_device.model);
            
            match adb::capture_screen(&selected_device.id) {
                Ok(raw_data) => {
                    if let Err(e) = video::parse_screenshot(raw_data, "screenshot.png") {
                        log::error!("Error processing screenshot: {}", e);
                    }
                }
                Err(e) => log::error!("Error capturing screen: {}", e),
            }
        }
        Err(e) => log::error!("Device selection failed: {}", e),
    }
}
