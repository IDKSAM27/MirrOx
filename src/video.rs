use std::error::Error;
use ffmpeg_next as ffmpeg;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    render::TextureAccess,
};

pub struct VideoRenderer {
    ictx: ffmpeg::format::context::Input,
}

impl VideoRenderer {
    pub fn new(ictx: ffmpeg::format::context::Input) -> Result<Self, Box<dyn Error>> {
        Ok(Self { ictx })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        ffmpeg::init()?;

        let input = &mut self.ictx;

        // Find video stream index
        let stream_index = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or("No video stream found")?
            .index();

        let stream = input.stream(stream_index).ok_or("Stream index not found")?;

        // Using new API to obtain decoder
        let codec_params = stream.parameters();
        let mut decoder = ffmpeg::codec::context::Context::from_parameters(codec_params)?
            .decoder()
            .video()?;

        // Setup scaler for RGB24
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

        // SDL2 initialization
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("MirrOx - Screen Mirroring", width, height)
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().present_vsync().build()?;
        let texture_creator = canvas.texture_creator();

        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Streaming,
            width,
            height,
        )?;

        let mut event_pump = sdl_context.event_pump()?;

        let mut receive_frame = ffmpeg::util::frame::Video::empty();
        let mut scaled_frame = ffmpeg::util::frame::Video::empty();

        println!("Video decoding and rendering started!");

        'mainloop: for (stream, packet) in input.packets() {
            if stream.index() != stream_index {
                continue;
            }

            decoder.send_packet(&packet)?;

            while decoder.receive_frame(&mut receive_frame).is_ok() {
                scaler.run(&receive_frame, &mut scaled_frame)?;

                let rgb_data = scaled_frame.data(0);
                texture.update(None, rgb_data, (width * 3) as usize)?;

                canvas.clear();
                canvas.copy(&texture, None, None)?;
                canvas.present();

                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            println!("Exiting renderer.");
                            break 'mainloop;
                        }
                        _ => {}
                    }
                }
            }
        }

        println!("Video renderer loop ended.");
        Ok(())
    }
}
