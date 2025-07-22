use std::net::TcpStream;
use std::io::{self, Read};
use std::thread;
use ffmpeg_next as ffmpeg;
use thiserror::Error;

// --- Custom Error Type ---
#[derive(Debug, Error)]
pub enum MuxerError {
    #[error("Failed to create FFmpeg input from custom IO stream: {0}")]
    InputCreationError(#[from] ffmpeg::Error),

    #[error("I/O error during stream bridging: {0}")]
    IoError(#[from] io::Error),
}

/// Bridges a TcpStream to a readable FFmpeg input context using a manual pipe.
///
/// This function returns a fully prepared `ffmpeg_next::format::context::Input`.
pub fn bridge_stream(mut stream: TcpStream) -> Result<ffmpeg::format::tcontext::Input, MuxerError> {
    // As before, we must read the scrcpy-server header before doing anything else.
    let mut header_buffer = [0u8; 2]; // [stream_type, name_length]
    stream.read_exact(&mut header_buffer)?;
    
    let device_name_length = header_buffer[1] as usize;
    if device_name_length > 0 {
        let mut device_name_buffer = vec![0u8; device_name_length];
        stream.read_exact(&mut device_name_buffer)?;
        let device_name = String::from_utf8_lossy(&device_name_buffer);
        println!("Connected to device: {}", device_name);
    } else {
        println!("Connected to device (no name provided).");
    }

    // Create an in-memory pipe.
    // `pipe_writer` is the end we write data into.
    // `pipe_reader` is the end FFmpeg will read from.
    let (pipe_reader, mut pipe_writer) = pipe::pipe();

    // Spawn a dedicated thread to pump data from the TCP stream into the pipe.
    thread::Builder::new()
        .name("tcp-to-pipe-bridge".to_string())
        .spawn(move || {
            // `io::copy` efficiently transfers all bytes from the stream to the pipe writer
            // until the connection is closed.
            match io::copy(&mut stream, &mut pipe_writer) {
                Ok(bytes) => println!("Bridge thread finished: copied {} bytes.", bytes),
                Err(e) => eprintln!("Error in bridge thread: {}", e),
            }
        })?;

    // Use FFmpeg's `Input::from_stream` to create a context directly from our pipe reader.
    let ictx = ffmpeg::format::context::Input::from_stream(pipe_reader)?;
    
    println!("Stream bridge created. Video data is now being piped to the FFmpeg context.");
    
    Ok(ictx)
}
