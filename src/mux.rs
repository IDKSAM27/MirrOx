use std::net::TcpStream;
use std::io::{self, Read};
use std::thread;
// --- CORRECTED PATH ---
// The `channel` function is now located in the input context module.
use ffmpeg_next::format::context::input::channel;
use thiserror::Error;

// --- Custom Error Type ---
// Updated to handle potential errors from ffmpeg_next directly.
#[derive(Debug, Error)]
pub enum MuxerError {
    #[error("Failed to create I/O channel for FFmpeg: {0}")]
    ChannelCreationError(#[from] ffmpeg_next::Error),

    #[error("I/O error during stream bridging: {0}")]
    IoError(#[from] io::Error),
}

/// Bridges a TcpStream to a readable FFmpeg input context.
///
/// This function now returns a fully prepared `ffmpeg_next::format::context::Input`,
/// which is the object the decoder will use to find and read the video stream.
pub fn bridge_stream(mut stream: TcpStream) -> Result<ffmpeg_next::format::context::Input, MuxerError> {
    // The scrcpy-server sends a small header before the video stream.
    // We must read this first, otherwise FFmpeg will receive invalid data and fail.
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

    // `channel()` creates a connected `Input` context and a `Writer` object.
    let (mut ictx, mut writer) = channel()?;

    // Spawn a dedicated thread to pump data from the TCP stream into the writer.
    // This decouples network I/O from the video decoding loop.
    thread::Builder::new()
        .name("tcp-to-ffmpeg-bridge".to_string())
        .spawn(move || {
            // `io::copy` efficiently transfers all bytes from the stream to the writer
            // until the connection is closed.
            match io::copy(&mut stream, &mut writer) {
                Ok(bytes) => println!("Bridge thread finished: copied {} bytes.", bytes),
                Err(e) => eprintln!("Error in bridge thread: {}", e),
            }
        })?;

    println!("Stream bridge created. Video data is now being piped to the FFmpeg context.");

    // Return the `Input` context. It is now ready to be used by the decoder.
    Ok(ictx)
}
