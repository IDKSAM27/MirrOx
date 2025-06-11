mod adb;
mod mux;
mod video;

use adb::start_scrcpy_server;
use mux::start_muxed_stream;
use video::start_video_stream;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Start scrcpy-server
    start_scrcpy_server("v3.2")?;

    // Connect to server and receive video byte stream
    let receiver = start_muxed_stream()?;

    // Decode and render video
    start_video_stream(receiver)?;

    Ok(())
}
