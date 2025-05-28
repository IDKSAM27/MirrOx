use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use anyhow::Result;

use ffmpeg_next::{
    codec, format, frame,
    software::scaling::{self, context::Context as Scaler},
    util::format::pixel,
};

use sdl2::{pixels::PixelFormatEnum, rect::Rect, event::Event};

pub fn start_video_streaming() -> Result<()> {
    // Initialize FFmpeg
    ffmpeg_next::init().unwrap();

    // Open input from TCP stream

    let mut ictx = match format::input("tcp://127.0.0.1:27183") {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("[MirrOx] FFmpeg failed to open stream: {e}");
            return Err(anyhow::anyhow!("FFmpeg input error"));
    }
    };

    
    let input = ictx.streams().best(ffmpeg_next::media::Type::Video).unwrap();
    let video_stream_index = input.index();

    let context_decoder = codec::context::Context::from_parameters(input.parameters()).unwrap();
    let decoder = context_decoder.decoder().video().unwrap();
    let decoder_params = (
        decoder.format(),
        decoder.width(),
        decoder.height(),
    );

    let (frame_tx, frame_rx): (Sender<frame::Video>, Receiver<frame::Video>) = mpsc::channel();

    // Move decoder into thread
    thread::spawn(move || {
        let mut decoder = decoder;

        let mut scaler = Scaler::get(
            decoder_params.0,
            decoder_params.1,
            decoder_params.2,
            pixel::Pixel::RGB24,
            decoder_params.1,
            decoder_params.2,
            scaling::Flags::BILINEAR,
        ).unwrap();

        let mut decoded = frame::Video::empty();

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                if decoder.send_packet(&packet).is_ok() {
                    while decoder.receive_frame(&mut decoded).is_ok() {
                        let mut rgb_frame = frame::Video::empty();
                        scaler.run(&decoded, &mut rgb_frame).unwrap();
                        frame_tx.send(rgb_frame).unwrap();
                    }
                }
            }
        }

        // Flush decoder
        decoder.send_eof().ok();
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
            match event {
                Event::Quit { .. } => break 'running Ok(()),
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
            canvas.copy(&texture, None, Some(Rect::new(0, 0, width, height))).unwrap();
            canvas.present();
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
