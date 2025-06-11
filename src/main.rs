mod adb;
mod tcp_client;
mod utils;
mod video;
mod mux;

use tcp_client::connect_to_server;
use adb::start_scrcpy_server;
use mux::spawn_mux_channel;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting MirrOx Server...");

    start_scrcpy_server()?;
    let tcp_stream = connect_to_server()?;
    println!("[*] Connected to server successfully.");

    // Start muxer thread to demux packets
    let receiver = spawn_mux_channel(tcp_stream)?;

    // Start the video decoder/renderer
    video::start_video_stream(receiver)?;

    Ok(())
}
