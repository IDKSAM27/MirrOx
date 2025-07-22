// src/mux.rs

use std::net::TcpStream;
use std::io::{self, Read};
use std::thread;
// --- THE STABLE SOLUTION ---
// Use the dedicated crate for creating a custom FFmpeg input.
use ffmpeg_next_io::demuxer::input;
use thiserror::Error;

// --- Custom Error Type ---
#[derive(Debug, Error)]
pub enum MuxerError {
    #[error("Failed to create I/O demuxer for FFmpeg: {0}")]
    DemuxerCreationError(#[from] ffmpeg_next::Error),

    #[error("I/O error during stream bridging: {0}")]
    IoError(#[from] io::Error),
}

/// Bridges a TcpStream to a readable FFmpeg input context using `ffmpeg-next-io`.
///
/// This function returns a fully prepared `ffmpeg_next::format::context::Input`,
/// ready for the video decoder.
pub fn bridge_stream(mut stream: TcpStream) -> Result<ffmpeg_next::format::context::Input, MuxerError> {
    // We must still read the scrcpy-server header before passing the stream along.
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

    // `input()` from ffmpeg-next-io takes our TcpStream and handles all the
    // complex background threading and piping for us.
    let ictx = input(stream)?;

    println!("Stream bridge created. Video data is now being piped to the FFmpeg context.");

    // The `input` function returns the FFmpeg Input context directly.
    Ok(ictx)
}
