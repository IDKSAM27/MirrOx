mod adb;
mod tcp_stream;

use adb::{AdbConfig, AdbSession};
use std::net::TcpStream; // We'll need this type soon

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Phase 1: Setup ADB Session ---
    // The responsibility of setting up the device is handled entirely by the adb module.
    let config = AdbConfig::default();
    let _session = AdbSession::new(config.clone())?; // RAII handles cleanup

    println!("ADB session established.");

    // --- Phase 2: Establish TCP Connection ---
    // The responsibility of connecting is handled entirely by the tcp_stream module.
    // We pass it the port from the config created in phase 1.
    let stream: TcpStream = tcp_stream::connect(config.local_port)?;

    println!("TCP connection successful. Stream is ready.");
    println!("Next steps: Bridge stream to FFmpeg and start decoding.");
    println!("Press Ctrl+C to exit.");

    // The application loop will go here. For now, we just keep it alive.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
