use std::io::{Read};
use std::net::TcpStream;

pub fn connect_to_server() -> std::io::Result<()> {
    println!("Connecting to scrcpy server on localhost:27183...");
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;

    let mut buffer = [0u8; 4096];
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        println!("[stream] Read {} bytes", n);
        // TODO: Decode and render video (H.264 stream)
    }

    Ok(())
}