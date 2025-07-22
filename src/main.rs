mod adb;
mod tcp_stream;
mod mux;

use adb::{AdbConfig, AdbSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AdbConfig::default();
    let _session = AdbSession::new(config.clone())?;
    println!("ADB session established.");

    let stream = tcp_stream::connect(config.local_port)?;

    let video_input_context = mux::bridge_stream(stream)?;

    println!("Stream bridge is active. Ready for video decoding.");
    println!("Next step: Initialize video decoder and SDL2 for rendering.");
    println!("Press Ctrl+C to exit.");
    
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
