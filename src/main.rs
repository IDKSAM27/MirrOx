mod adb;
mod utils;
mod tcp_client;
mod video;
mod mux;

use crossbeam_channel::unbounded;
use std::net::TcpStream;
use std::io::Result;

fn main() -> Result<()> {
    println!("Starting MirrOx Server...");

    // Get scrcpy-server version string from utils
    let version = utils::get_scrcpy_server_version()
        .expect("[*] Could not read server version from server/version.txt");

    // Start the scrcpy server on the device
    if let Err(e) = adb::start_scrcpy_server(&version) {
        eprintln!("Failed to start server: {e}");
        return Ok(());
    }

    // Connect to the scrcpy server via TCP (127.0.0.1:27183 forwarded by adb)
    let tcp_stream = match tcp_client::connect_to_server() {
        Ok(stream) => {
            println!("[*] Connected to server successfully.");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {e}");
            return Ok(());
        }
    };

    // Create crossbeam channels for multiplexed data
    let (video_tx, video_rx) = unbounded();
    let (control_tx, control_rx) = unbounded();
    let (clipboard_tx, clipboard_rx) = unbounded();
    let (device_tx, device_rx) = unbounded();

    // Start demuxing the TCP stream into channels
    mux::demux(tcp_stream, video_tx, control_tx, clipboard_tx, device_tx);

    // Launch video streaming using the video_rx channel
    video::start_video_stream(video_rx)?;

    // TODO: Launch handlers for control_rx, clipboard_rx, device_rx in separate threads

    Ok(())
}
