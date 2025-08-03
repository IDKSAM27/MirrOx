mod adb;
mod tcp_stream;
mod mux;
mod video; // ADD this

use adb::{AdbConfig, AdbSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AdbConfig::default();
    let _session = AdbSession::new(config.clone())?;
    println!("ADB session established.");

    let stream = tcp_stream::connect(config.local_port)?;
    let video_input_context = mux::bridge_stream(stream)?;

    // Updated: Start the modular video renderer!
    let mut renderer = video::VideoRenderer::new(video_input_context)?;
    renderer.run()?; // Runs until closed

    Ok(())
}
