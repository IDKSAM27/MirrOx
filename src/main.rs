mod adb;
mod tcp_stream;
mod mux;

use adb::{AdbConfig, AdbSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Phase 1: Setup ADB Session ---
    let config = AdbConfig::default();
    let _session = AdbSession::new(config.clone())?;
    println!("ADB session established.");

    // --- Phase 2: Establish TCP Connection ---
    let stream = tcp_stream::connect(config.local_port)?;

    // --- Phase 3: Bridge the Stream for Decoding ---
    // The variable now holds an FFmpeg Input context directly.
    let video_input_context = mux::bridge_stream(stream)?;

    println!("Stream bridge is active. Ready for video decoding.");
    println!("Next step: Initialize video decoder and SDL2 for rendering.");
    println!("Press Ctrl+C to exit.");

    // The application loop will go here.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
