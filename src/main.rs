mod adb;
mod tcp_stream;
mod mux;
mod video; // Add this

use adb::{AdbConfig, AdbSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ADB session setup
    let config = AdbConfig::default();
    let _session = AdbSession::new(config.clone())?;
    println!("ADB session established.");

    // TCP connect
    let stream = tcp_stream::connect(config.local_port)?;

    // Create FFmpeg input context (mux bridge)
    let video_input_context = mux::bridge_stream(stream)?;

    // New: Start video renderer!
    let mut renderer = video::VideoRenderer::new(video_input_context)?;
    renderer.run()?;

    Ok(())
}
