use std::io::{Read, Write};
use std::net::TcpStream;

pub fn connect_to_server() -> std::io::Result<TcpStream> {
    println!("Connecting to server on localhost:27183...");
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    println!("[MirrOx] Connected to server!");

    // Send "client hello" header as expected by scrcpy-server
    let client_name = "MirrOx";
    let version = 1;
    let hello_message = build_hello_message(client_name, version);
    stream.write_all(&hello_message)?;
    stream.flush()?;
    println!("[MirrOx] Sent client hello");

    // Check response or read header bytes if needed (optional)

    Ok(stream)
}

// Build the initial "client hello" packet to trigger the video stream
fn build_hello_message(client_name: &str, version: u8) -> Vec<u8> {
    let name_bytes = client_name.as_bytes();
    let name_len = name_bytes.len() as u8;

    // Format: [version (1 byte)][name length (1 byte)][name (n bytes)]
    let mut buffer = Vec::with_capacity(2 + name_bytes.len());
    buffer.push(version);
    buffer.push(name_len);
    buffer.extend_from_slice(name_bytes);

    buffer
}
