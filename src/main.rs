// //! MirrOx: A Rust-based implementation of scrcp
mod utils;
mod adb;

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
                println!("Device: {} | Battery: {}% | Uptime: {}", device.model, device.battery_level, device.uptime); 
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
