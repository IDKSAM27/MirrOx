// //! MirrOx: A Rust-based implementation of scrcp

// mod adb;

// fn main() {
//     println!("Starting MirrOx...");

//     // Check if ADB is available
//     if let Err(e) = adb::check_adb() {
//         eprintln!("Error: {}", e);
//         return;
//     }

//     // List connected devices with their state
//     match adb::list_devices() {
//         Ok(devices) => {
//             println!("Connected devices:");
//             for device in devices {
//                 println!("- {} ({})", device.id, device.state); // State would be: device, unauthorized, offline
//             }
//         }
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }

mod adb; // Import the adb module

fn main() {
    println!("Starting MirrOx...");

    // Check if ADB is available
    if let Err(e) = adb::check_adb() {
        eprintln!("Error: {}", e);
        return;
    }

    // List connected devices with their state and connection type
    match adb::list_devices() {
        Ok(devices) => {
            println!("Connected devices:");
            for device in devices {
                println!("- {} ({}) [{}]", device.id, device.state, device.connection_type);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
