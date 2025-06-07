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

    let mut stream = match tcp_client::connect_to_server() {
        Ok(s) => {
            println!("[*] Connected to server successfully.");
            s
        }
        Err(e) => {
            eprintln!("Failed to connect to server: {e}");
            return Ok(());
        }
    };

    // Scrcpy sends a 1-byte channel indicator: 0x00 (video)
    let mut first_byte = [0u8; 1];
    stream.read_exact(&mut first_byte)?;

    if first_byte[0] != 0x00 {
        eprintln!("[!] Expected video stream (channel 0x00), got 0x{:02X}", first_byte[0]);
        return Ok(());
    }

    // Feed remaining stream directly to video decoder
    video::start_video_stream(stream)?;

    Ok(())
}
