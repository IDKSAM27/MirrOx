use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use ffmpeg_next::{
    codec,
    decoder::Video as VideoDecoder,
    format,
    format::Pixel,
    frame,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::rational::Rational,
};
use sdl2::{event::Event, pixels::PixelFormatEnum};

pub fn start_video_stream() -> Result<()> {
    println!("[*] Connecting to server on localhost:27183...");
    let mut stream = TcpStream::connect("127.0.0.1:27183")
        .context("Failed to connect to scrcpy server")?;
    stream.set_nonblocking(true)?;

    println!("[MirrOx] Connected to server!");

    // Create a temp file for FFmpeg to read
    let mut temp_file = tempfile::NamedTempFile::new().context("Failed to create temp file")?;

    // Spawn a thread to read bytes from the socket and write to temp file
    let mut cloned_stream = stream.try_clone().context("Failed to clone TCP stream")?;
    let mut temp_path = temp_file.path().to_path_buf();
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(n) = cloned_stream.read(&mut buffer) {
            if n == 0 {
                break;
            }
            if let Err(e) = temp_file.write_all(&buffer[..n]) {
                eprintln!("[MirrOx] Write error: {e:?}");
                break;
            }
        }
    });

    // Wait a bit to fill some buffer (you can tune this)
    thread::sleep(Duration::from_millis(1000));

    // Now initialize FFmpeg format context from temp file
    ffmpeg_next::init().context("Failed to initialize FFmpeg")?;
    let mut ictx = format::input(&temp_path).context("FFmpeg input error")?;

    let input_stream = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .context("Couldn't find best video stream")?;
    let video_stream_index = input_stream.index();
    let codec_params = input_stream.parameters();
    let decoder = codec::Context::from_parameters(codec_params)?
        .decoder()
        .video()
        .context("Couldn't get video decoder")?;

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )
    .context("Couldn't initialize scaler")?;

    // SDL2 setup
    let sdl = sdl2::init().map_err(|e| anyhow::anyhow!(e))?;
    let video_subsystem = sdl.video().map_err(|e| anyhow::anyhow!(e))?;
    let window = video_subsystem
        .window("MirrOx", decoder.width() as u32, decoder.height() as u32)
        .position_centered()
        .build()
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| anyhow::anyhow!(e))?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            decoder.width(),
            decoder.height(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut event_pump = sdl.event_pump().map_err(|e| anyhow::anyhow!(e))?;
    let mut decoded = frame::Video::empty();
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = frame::Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;

                texture.with_lock(None, |buffer, pitch| {
                    let data = rgb_frame.data(0);
                    let linesize = rgb_frame.linesize(0);
                    for y in 0..decoder.height() {
                        let src = &data[(y * linesize) as usize..(y * linesize + decoder.width() * 3) as usize];
                        let dst = &mut buffer[(y * pitch) as usize..(y * pitch + decoder.width() * 3) as usize];
                        dst.copy_from_slice(src);
                    }
                })?;

                canvas.clear();
                canvas.copy(&texture, None, None)?;
                canvas.present();

                if let Some(Event::Quit { .. }) = event_pump.poll_event() {
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}
