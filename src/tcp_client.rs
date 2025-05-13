use std::io::{Read};
use std::net::TcpStream;

pub fn connect_to_scrcpy() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    println!("[MirrOx] Connected to scrcpy server!");

    // Dummy example: send or receive basic handshake (scrcpy will send stream header)
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf)?;
    println!("[MirrOx] Received {} bytes", n);

    Ok(())
}
