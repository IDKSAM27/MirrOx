mod adb;

use adb::{AdbConfig, AdbSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a configuration.
    // We can use the default or customize it here.
    let config = AdbConfig::default();

    // 2. Initialize the session.
    // The `_session` variable's lifetime controls the entire connection.
    // When it's dropped at the end of `main`, cleanup is automatic.
    let _session = AdbSession::new(config.clone())?; // Pass config

    println!("ADB session established. Port {} is ready.", config.local_port);
    println!("Press Ctrl+C to exit.");
    
    // The next step will be to create and use the TCP stream and video decoder here.
    // For now, we just keep the application alive.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // `_session` is dropped here, and `impl Drop for AdbSession` is called.
}
