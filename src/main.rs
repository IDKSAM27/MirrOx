mod adb;
mod utils;
mod tcp_client;
mod video;
mod mux;

use crossbeam_channel::unbounded;
use std::net::TcpStream;

fn main() {
    println!("Starting MirrOx Server...");

    // Get scrcpy-server version from local file
    let version = utils::get_scrcpy_server_version()
        .expect("[*] Could not read server version from server/version.txt");

    // Start the scrcpy-server on device
    if let Err(e) = adb::start_scrcpy_server(&version) {
        eprintln!("Failed to start server: {e}");
        return;
    }

    // Connect to the forwarded TCP port
    let tcp_stream = match tcp_client::connect_to_server() {
        Ok(stream) => {
            println!("[*] Connected to server successfully.");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {e}");
            return;
        }
    };

    // Set up channel and spawn demuxer
    let (video_tx, video_rx) = unbounded();
    mux::demux(tcp_stream, video_tx);

    // Start the SDL2 + FFmpeg video pipeline
    if let Err(e) = video::start_video_stream(video_rx) {
        eprintln!("Video stream error: {e}");
    }
}
