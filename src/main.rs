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

    // let device_id = device.id;

    match adb_push(device.id, "D:/test.txt", "/sdcard/ADB/test.txt") {
        Ok(_) => println!("Push successful"),
        Err(e) => println!("Error: {}", e),
    }

    match adb_pull(device.id, "/sdcard/ADB/file.txt", "D:/file.txt") {
        Ok(_) => println!("Pull successful"), 
        Err(e) => println!("Error: {}", e),
    }
}
