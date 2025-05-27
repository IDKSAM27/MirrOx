use std::net::TcpStream;
use std::io::{Read};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use anyhow::Result;

use ffmpeg_next::{decoder::Video, format, frame, software::scaling, util::format::pixel};

use sdl2::{pixels::PixelFormatEnum, rect::Rect};

pub fn start_video_streaming() -> Result<()> {
    // Connect to the scrcpy server TCP stream
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    stream.set_nonblocking(true)?;

    println!("[MirrOx] Connected to server!");

    let (frame_tx, frame_rx): (Sender<frame::Video>, Receiver<frame::Video>) = mpsc::channel();

    // Spawn thread for reading and decoding video
    thread::spawn(move || {
        let _ = ffmpeg_next::init();

        // Guess format from raw input (scrcpy sends an MPEG-TS stream)
        let mut ictx = format::input(&mut stream).expect("Failed to open input format"); // TcpStream issue, required some kind of bound. NOT SURE.
        let input = ictx.streams().best(ffmpeg_next::media::Type::Video).unwrap();
        let video_stream_index = input.index();

        let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters()).unwrap();
        let mut decoder = context_decoder.decoder().video().unwrap();

        let mut scaler = scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            pixel::Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            scaling::Flags::BILINEAR,
        ).unwrap();

        let mut decoded = frame::Video::empty();

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.decode(&packet, &mut decoded).unwrap(); // 7.1.0 doesn't have 'decode' ig
                let mut rgb_frame = frame::Video::empty();
                scaler.run(&decoded, &mut rgb_frame).unwrap();
                frame_tx.send(rgb_frame).unwrap();
            }
        }
    });

    // SDL2 window + renderer
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

    'running: loop {
        if let Ok(frame) = frame_rx.try_recv() {
            let (width, height) = (frame.width(), frame.height());

            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB24, width, height)
                .unwrap();

            texture
                .update(None, frame.data(0), (3 * width) as usize)
                .unwrap();

            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, width, height))).unwrap();
            canvas.present();
        }

        // TODO: Add SDL2 event pump and exit condition
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
