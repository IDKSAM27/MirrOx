use std::net::TcpStream;
use std::io::{Read, BufReader};
use ffmpeg_next as ffmpeg;

pub fn start_video_stream() -> anyhow::Result<()> {
    ffmpeg::init()?; // Initialize FFmpeg

    let stream = TcpStream::connect("127.0.0.1:27183")?;
    let mut reader = BufReader::new(stream);

    // TODO: Setup decoder
    let codec = ffmpeg::codec::decoder::find(ffmpeg::codec::Id::H264)
        .ok_or_else(|| anyhow::anyhow!("H264 decoder not found"))?;

    let mut context = codec
        .open_as()
        .map_err(|e| anyhow::anyhow!("Failed to open codec: {}", e))?;

    let mut packet = ffmpeg::codec::packet::Packet::empty();
    let mut buf = vec![0u8; 1024 * 1024]; // 1MB buffer

    loop {
        let len = reader.read(&mut buf)?;
        if len == 0 {
            break;
        }

        packet.data_mut().extend_from_slice(&buf[..len]);

        if let Ok(_) = context.send_packet(&packet) {
            let mut frame = ffmpeg::util::frame::Video::empty();
            if context.receive_frame(&mut frame).is_ok() {
                // Frame decoded: render it here
                println!("[Video] Decoded frame: {}x{}", frame.width(), frame.height());
            }
        }

        packet.data_mut().clear();
    }

    Ok(())
}
