use std::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use anyhow::Result;
use ffmpeg_next::{
    codec, format, frame,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel,
};

use sdl2::{pixels::PixelFormatEnum, rect::Rect};

pub fn start_video_streaming() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    stream.set_nonblocking(true)?;

    println!("[MirrOx] Connected to server!");

    let (frame_tx, frame_rx): (Sender<frame::Video>, Receiver<frame::Video>) = mpsc::channel();

    // Spawn thread for reading and decoding video
    thread::spawn(move || {
        if ffmpeg_next::init().is_err() {
            eprintln!("Failed to initialize FFmpeg");
            return;
        }

        let mut ictx = match format::input(&mut stream) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Could not open input: {}", e);
                return;
            }
        };

        let input = match ictx.streams().best(ffmpeg_next::media::Type::Video) {
            Some(s) => s,
            None => {
                eprintln!("No video stream found");
                return;
            }
        };

        let video_stream_index = input.index();
        let context_decoder =
            codec::context::Context::from_parameters(input.parameters()).unwrap();
        let mut decoder = context_decoder.decoder().video().unwrap();

        let mut scaler = Scaler::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            pixel::Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )
        .unwrap();

        let mut decoded = frame::Video::empty();

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                if let Err(e) = decoder.decode(&packet, &mut decoded) {
                    eprintln!("Decode error: {}", e);
                    continue;
                }

                let mut rgb_frame = frame::Video::empty();
                scaler.run(&decoded, &mut rgb_frame).unwrap();
                frame_tx.send(rgb_frame).unwrap();
            }
        }

        // Flush decoder
        if let Err(e) = decoder.send_eof() {
            eprintln!("Error sending EOF: {}", e);
        }
        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = frame::Video::empty();
            scaler.run(&decoded, &mut rgb_frame).unwrap();
            frame_tx.send(rgb_frame).unwrap();
        }
    });

    // SDL2 setup
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("MirrOx", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
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

        thread::sleep(Duration::from_millis(1));
    }

    // Ok(())
}
