// //! MirrOx: A Rust-based implementation of scrcp

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
                println!("- {} ({}) [{}] - Model: {}", device.id, device.state, device.connection_type, device.model);
            }

            // Example: Run a shell command on the first device
            if let Some(device) = devices.first() {
                match adb::run_shell_command(&device.id, "uptime") {
                    Ok(output) => println!("Device {} Uptime: {}", device.id, output),
                    Err(e) => eprintln!("Error running shell command: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
