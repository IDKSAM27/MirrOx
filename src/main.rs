mod adb;
mod utils;
mod tcp_client;
mod video;
// mod mux;

use anyhow::Result;
use std::io::Read;
// use std::net::TcpStream;

fn main() -> Result<()> {
    println!("Starting MirrOx Server...");

    let version = utils::get_scrcpy_server_version()
        .expect("[*] Could not read server version from server/version.txt");

    if let Err(e) = adb::start_scrcpy_server(&version) {
        eprintln!("Failed to start server: {e}");
        return Ok(());
    }

    let mut tcp_stream = match tcp_client::connect_to_server() {
        Ok(stream) => {
            println!("[*] Connected to server successfully.");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {e}");
            return Ok(());
        }
    };

    // Discard the first byte: frame type header `0x00` for video stream
    let mut frame_type = [0u8; 1];
    tcp_stream.read_exact(&mut frame_type)?;
    if frame_type[0] != 0x00 {
        return Err(anyhow::anyhow!(
            "Expected video frame type (0x00), got: {:02x}",
            frame_type[0]
        ));
    }

    // Now safely pass the stream to FFmpeg
    video::start_video_stream(tcp_stream)?;

    Ok(())
}

