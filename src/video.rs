// src/video.rs

use std::error::Error;
use std::sync::mpsc::{self, TryRecvError};
use ffmpeg_next as ffmpeg;
use sdl2::{pixels::PixelFormatEnum, render::TextureAccess, event::Event, keyboard::Keycode};

pub struct VideoRenderer {
    ictx: ffmpeg::format::context::Input,
}

impl VideoRenderer {
    pub fn new(ictx: ffmpeg::format::context::Input) -> Result<Self, Box<dyn Error>> {
        Ok(Self { ictx })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        ffmpeg::init()?;

        // Find the video stream
        let input = &self.ictx;
        let stream_index = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or("No video stream found")?
            .index();

        // Open the decoder
        let stream = input.stream(stream_index).ok_or("Stream index not found")?;
        let mut decoder = stream.codec().decoder().video()?;

        // Setup RGB sws scaler
        let width = decoder.width();
        let height = decoder.height();
        let mut scaler = ffmpeg::software::scaling::Context::get(
            decoder.format(),
            width,
            height,
            ffmpeg::format::Pixel::RGB24,
            width,
            height,
            ffmpeg::software::scaling::Flags::BILINEAR,
        )?;

        // SDL2 Setup
        let sdl = sdl2::init()?;
        let video_sub = sdl.video()?;
        let window = video_sub.window("MirrOx - Screen Mirroring", width, height).position_centered().build()?;
        let mut canvas = window.into_canvas().present_vsync().build()?;
        let texture_creator = canvas.texture_creator();

        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Streaming,
            width,
            height,
        )?;

        // Packet/frame containers
        let mut receive_frame = ffmpeg::util::frame::Video::empty();
        let mut scaled_frame = ffmpeg::util::frame::Video::empty();

        // Setup SDL2 event handling for quit via ESC or window close
        let mut event_pump = sdl.event_pump()?;

        println!("Video decoding and rendering started!");
        'mainloop: for (stream, mut packet) in input.packets() {
            if stream.index() != stream_index { continue; }

            decoder.send_packet(&packet)?;

            while decoder.receive_frame(&mut receive_frame).is_ok() {
                scaler.run(&receive_frame, &mut scaled_frame)?;
                // Access the RGB bytes
                let rgb = scaled_frame.data(0);

                // Feed to SDL2 texture and render
                texture.update(None, rgb, (width * 3) as usize)?;
                canvas.clear();
                canvas.copy(&texture, None, None)?;
                canvas.present();

                // Handle SDL2 events
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => {
                            println!("Exiting renderer loop.");
                            break 'mainloop;
                        }
                        _ => {}
                    }
                }
            }
        }

        println!("Video loop finished.");
        Ok(())
    }
}
