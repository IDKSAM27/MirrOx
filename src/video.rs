use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use anyhow::{Context, Result};

use ffmpeg_next::{
    codec, format,
    format::io::IO,
    frame,
    media::Type,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect};

pub fn start_video_streaming() -> Result<()> {
    // Connect to scrcpy server
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    println!("[MirrOx] Connected to server!");

    // Read initial bytes to avoid 0-byte error
    let mut buffer = [0u8; 4096];
    let bytes_read = stream.read(&mut buffer)?;
    println!("[MirrOx] Received {bytes_read} bytes");

    if bytes_read == 0 {
        return Err(anyhow::anyhow!("No data received from server"));
    }

    let mut stream_data = Vec::from(&buffer[..bytes_read]);
    stream.read_to_end(&mut stream_data).ok(); // keep reading stream

    // Set up in-memory input with FFmpeg
    ffmpeg_next::init()?;
    let io = IO::from_seekable_read(stream_data.as_slice());

    let mut ictx = format::input_with_io(io)
        .context("FFmpeg failed to open stream")?;

    let input = ictx
        .streams()
        .best(Type::Video)
        .context("Couldn't find best video stream")?;
    let video_stream_index = input.index();

    let context_decoder =
        codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        pixel::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let (frame_tx, frame_rx): (Sender<frame::Video>, Receiver<frame::Video>) =
        mpsc::channel();

    // Decoding thread
    thread::spawn(move || {
        let mut decoded = frame::Video::empty();
        let mut rgb_frame = frame::Video::empty();

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                if decoder.decode(&packet, &mut decoded).is_ok() {
                    if scaler.run(&decoded, &mut rgb_frame).is_ok() {
                        let _ = frame_tx.send(rgb_frame.clone());
                    }
                }
            }
        }
    });

    // SDL2 init
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let window = video_subsystem
        .window("MirrOx", 800, 600)
        .position_centered()
        .resizable()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            }
        }

        if let Ok(frame) = frame_rx.try_recv() {
            let (width, height) = (frame.width(), frame.height());

            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB24, width, height)
                .unwrap();

            texture
                .update(None, frame.data(0), (3 * width) as usize)
                .unwrap();

            canvas.clear();
            canvas
                .copy(&texture, None, Some(Rect::new(0, 0, width, height)))
                .unwrap();
            canvas.present();
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
