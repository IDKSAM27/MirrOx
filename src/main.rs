mod adb;
mod utils;
mod tcp_client;
mod video;
mod mux;

use crossbeam_channel::unbounded;
use std::net::TcpStream;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting MirrOx Server...");

    let version = utils::get_scrcpy_server_version()
        .expect("[*] Could not read server version from server/version.txt");

    if let Err(e) = adb::start_scrcpy_server(&version) {
        eprintln!("Failed to start server: {e}");
        return Ok(());
    }

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

    let (video_tx, video_rx) = unbounded();
    let (control_tx, control_rx) = unbounded();
    let (clipboard_tx, clipboard_rx) = unbounded();
    let (device_tx, device_rx) = unbounded();

    mux::demux(tcp_stream, video_tx, control_tx, clipboard_tx, device_tx);

    video::start_video_stream(video_rx)?; 

    Ok(())
}
