use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr;

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec,
    decoder::Video as VideoDecoder,
    format::{self, context::Input},
    frame::Video,
    media::Type,
    software::scaling::{context::Context as Scaler, flag::Flags},
};
use ffmpeg_sys_next::{
    avformat_alloc_context, avformat_find_stream_info, avformat_open_input,
    avio_alloc_context,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect};

use crate::mux::FifoIO;

const BUFFER_SIZE: usize = 4096;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init().unwrap();

    let mut fifo = Box::new(FifoIO::new(receiver));
    let fifo_ptr = &mut *fifo as *mut FifoIO as *mut c_void;

    let buffer = vec![0u8; BUFFER_SIZE];
    let buffer_ptr = buffer.as_ptr() as *mut c_uchar;
    std::mem::forget(buffer); // FFmpeg owns this memory

    let avio_ctx = unsafe {
        avio_alloc_context(
            buffer_ptr,
            BUFFER_SIZE as c_int,
            0,
            fifo_ptr,
            Some(FifoIO::read_packet),
            None,
            None,
        )
    };
    if avio_ctx.is_null() {
        return Err("Failed to allocate AVIOContext".into());
    }

    let fmt_ctx = unsafe { avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate AVFormatContext".into());
    }

    unsafe {
        (*fmt_ctx).pb = avio_ctx;
    }

    if unsafe {
        avformat_open_input(&mut (fmt_ctx as *mut _), ptr::null(), ptr::null_mut(), ptr::null_mut())
    } < 0
    {
        return Err("Failed to open input from AVIO".into());
    }

    if unsafe { avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    // Correct usage: wrap the raw pointer
    let mut context = unsafe { Input::wrap(fmt_ctx) };

    let input = context
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;

    let video_stream_index = input.index();
    let codec_params = input.parameters();
    let codec_id = codec_params.id();

    let codec = codec::decoder::find(codec_id).ok_or("Decoder not found")?;
    let mut decoder = codec.decoder().video()?;

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg_next::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("MirrOx", decoder.width() as u32, decoder.height() as u32)
        .position_centered()
        .opengl()
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator()use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr;

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec,
    decoder::Video as VideoDecoder,
    format::{self, context::Input},
    frame::Video,
    media::Type,
    software::scaling::{context::Context as Scaler, flag::Flags},
};
use ffmpeg_sys_next::{
    avformat_alloc_context, avformat_find_stream_info, avformat_open_input,
    avio_alloc_context,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect};

use crate::mux::FifoIO;

const BUFFER_SIZE: usize = 4096;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init().unwrap();

    let mut fifo = Box::new(FifoIO::new(receiver));
    let fifo_ptr = &mut *fifo as *mut FifoIO as *mut c_void;

    let buffer = vec![0u8; BUFFER_SIZE];
    let buffer_ptr = buffer.as_ptr() as *mut c_uchar;
    std::mem::forget(buffer); // FFmpeg owns this memory

    let avio_ctx = unsafe {
        avio_alloc_context(
            buffer_ptr,
            BUFFER_SIZE as c_int,
            0,
            fifo_ptr,
            Some(FifoIO::read_packet),
            None,
            None,
        )
    };
    if avio_ctx.is_null() {
        return Err("Failed to allocate AVIOContext".into());
    }

    let fmt_ctx = unsafe { avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate AVFormatContext".into());
    }

    unsafe {
        (*fmt_ctx).pb = avio_ctx;
    }

    if unsafe {
        avformat_open_input(&mut (fmt_ctx as *mut _), ptr::null(), ptr::null_mut(), ptr::null_mut())
    } < 0
    {
        return Err("Failed to open input from AVIO".into());
    }

    if unsafe { avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    // Correct usage: wrap the raw pointer
    let mut context = unsafe { Input::wrap(fmt_ctx) };

    let input = context
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;

    let video_stream_index = input.index();
    let codec_params = input.parameters();
    let codec_id = codec_params.id();

    let codec = codec::decoder::find(codec_id).ok_or("Decoder not found")?;
    let mut decoder = codec.decoder().video()?;

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg_next::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("MirrOx", decoder.width() as u32, decoder.height() as u32)
        .position_centered()
        .opengl()
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, decoder.width(), decoder.height())?;

    let mut event_pump = sdl.event_pump()?;
    let mut packet = ffmpeg_next::Packet::empty();

    while context.read_packet(&mut packet).is_ok() {
        if packet.stream() != video_stream_index {
            continue;
        }

        if decoder.send_packet(&packet).is_ok() {
            let mut frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&frame, &mut rgb_frame)?;

                texture.update(
                    None,
                    rgb_frame.data(0),
                    rgb_frame.stride(0),
                )?;

                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width(), decoder.height())))?;
                canvas.present();
            }
        }

        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Ok(());
            }
        }
    }

    Ok(())
}
;
    let mut texture = texture_creatoruse std::os::raw::{c_int, c_uchar, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::thread;

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec,
    codec::traits::Decoder,
    format::{self, context::Input},
    frame,
    media,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect};

use crate::mux::FifoIO;

pub fn start_video_stream(rx: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init().unwrap();

    // Setup SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("MirrOx - Screen Stream", 1280, 720)
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    // Wrap the rx in a FifoIO (custom AVIOContext buffer source)
    let fifo = Arc::new(FifoIO::new(rx));
    let mut avio_ctx = fifo.create_avio_context()?;

    // Allocate AVFormatContext manually
    let fmt_ctx = unsafe { ffmpeg_next::ffi::avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate AVFormatContext".into());
    }

    // Hook the AVIOContext to it
    unsafe {
        (*fmt_ctx).pb = avio_ctx.as_mut_ptr();
    }

    // Open input using custom IO
    let mut input_ctx = unsafe { Input::wrap(fmt_ctx) };

    // Read stream info
    input_ctx.find_stream_info(None)?;

    // Find video stream index
    let stream_index = input_ctx
        .streams()
        .best(media::Type::Video)
        .ok_or("No video stream found")?
        .index();

    let stream = input_ctx.stream(stream_index).ok_or("Stream not found")?;
    let codec_params = stream.parameters();
    let codec_id = codec_params.id();
    let decoder_codec = codec::decoder::find(codec_id).ok_or("Decoder not found")?;
    let mut decoder = decoder_codec.decoder().video()?;

    decoder.set_parameters(codec_params)?;
    decoder.open()?;

    // Setup scaler and frame containers
    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let mut rgb_frame = frame::Video::empty();
    let mut decoded = frame::Video::empty();

    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        decoder.width(),
        decoder.height(),
    )?;

    // Main decoding and rendering loop
    for (i, packet) in input_ctx.packets().enumerate() {
        if packet.stream_index() != stream_index {
            continue;
        }

        decoder.send_packet(&packet)?;
        while decoder.receive_frame(&mut decoded).is_ok() {
            scaler.run(&decoded, &mut rgb_frame)?;

            let data = rgb_frame.data(0);
            let linesize = rgb_frame.stride(0);

            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..decoder.height() {
                    let src = &data[(y * linesize) as usize..((y + 1) * linesize) as usize];
                    let dst = &mut buffer[(y * pitch) as usize..(y + 1) * pitch as usize];
                    dst[..decoder.width() as usize * 3].copy_from_slice(&src[..decoder.width() as usize * 3]);
                }
            })?;

            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width(), decoder.height())))?;
            canvas.present();
        }

        // Exit if the window is closed
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Ok(());
            }
        }
    }

    Ok(())
}
use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr;
use std::slice;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec,
    codec::traits::Decoder,
    format::{self, context::Input},
    frame,
    media,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::{render::Canvas, video::Window, EventPump};

use crate::mux::FifoIO;

pub fn start_video_stream(receiver: Receiver<u8>, canvas: &mut Canvas<Window>, event_pump: &mut EventPump) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init().unwrap();

    // Create FifoIO for FFmpeg's custom AVIOContext
    let mut fifo_io = FifoIO::new(receiver);
    let mut fmt_ctx = fifo_io.open_format_context()?;

    fmt_ctx.find_stream_info(None)?;

    let input_stream = fmt_ctx
        .streams()
        .best(media::Type::Video)
        .ok_or("Could not find video stream")?;

    let codec_params = input_stream.parameters();
    let decoder_codec = codec::decoder::find(codec_params.id())
        .ok_or("Decoder not found")?;

    let mut decoder = decoder_codec.decoder().video()?;
    decoder.open_with(codec_params)?;

    let mut scaler = Scaler::get(use std::os::raw::{c_int, c_uchar, c_void};

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec,
    codec::traits::Decoder, // <-- THIS is the missing trait!
    format::{self, context::Input},
    frame,
    media,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::{render::Canvas, video::Window, EventPump, pixels::PixelFormatEnum};

use crate::mux::FifoIO;

pub fn start_video_stream(
    receiver: Receiver<u8>,
    canvas: &mut Canvas<Window>,
    event_pump: &mut EventPump,
) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init().unwrap();

    let mut fifo_io = FifoIO::new(receiver);
    let mut fmt_ctx = fifo_io.open_format_context()?;

    fmt_ctx.find_stream_info(None)?;

    let input_stream = fmt_ctx
        .streams()
        .best(media::Type::Video)
        .ok_or("Could not find video stream")?;

    let codec_params = input_stream.parameters();
    let codec = codec::decoder::find(codec_params.id()).ok_or("Codec not found")?;

    let mut decoder = codec.decoder().video()?; // <-- Now valid
    decoder.open_with(codec_params)?;

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGBA,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let texture_creator = canvas.texture_creator(); // ✅ this line is now correct
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA32,
        decoder.width(),
        decoder.height(),
    )?;

    for (stream, packet) in fmt_ctx.packets() {
        if stream.index() != input_stream.index() {
            continue;
        }

        decoder.send_packet(&packet)?;
        let mut frame = frame::Video::empty();

        while decoder.receive_frame(&mut frame).is_ok() {
            let mut rgb = frame::Video::empty();
            scaler.run(&frame, &mut rgb)?;

            let data = rgb.data(0);
            let stride = rgb.stride(0);

            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..decoder.height() {
                    let src = &data[(y * stride) as usize..(y * stride + decoder.width() * 4) as usize];
                    let dst = &mut buffer[(y * pitch) as usize..(y * pitch + decoder.width() * 4) as usize];
                    dst.copy_from_slice(src);
                }
            })?;

            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }
    }

    decoder.send_eof()?;
    Ok(())
}

        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGBA,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA32,
        decoder.width(),
        decoder.height(),
    )?;

    let mut frame_index = 0;
    for (stream, packet) in fmt_ctx.packets() {
        if stream.index() != input_stream.index() {
            continue;
        }

        decoder.send_packet(&packet)?;
        let mut decoded = frame::Video::empty();

        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = frame::Video::empty();
            scaler.run(&decoded, &mut rgb_frame)?;

            let data = rgb_frame.data(0);
            let linesize = rgb_frame.stride(0);

            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..decoder.height() {
                    let src = &data[(y * linesize) as usize..((y + 1) * linesize) as usize];
                    let dst = &mut buffer[(y * pitch) as usize..((y + 1) * pitch) as usize];
                    dst[..decoder.width() as usize * 4].copy_from_slice(&src[..decoder.width() as usize * 4]);
                }
            })?;

            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();

            frame_index += 1;

            // Handle basic SDL2 events (close on window quit)
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => return Ok(()),
                    _ => {}
                }
            }
        }
    }

    decoder.send_eof()?;
    Ok(())
}

        .create_texture_streaming(PixelFormatEnum::RGB24, decoder.width(), decoder.height())?;

    let mut event_pump = sdl.event_pump()?;
    let mut packet = ffmpeg_next::Packet::empty();

    while context.read_packet(&mut packet).is_ok() {
        if packet.stream() != video_stream_index {
            continue;use std::os::raw::{c_int, c_uchar, c_void};
use std::ptr;

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec::{self, traits::Decoder},
    decoder::Video as VideoDecoder,
    format::{self, context::Input},
    frame::Video,
    media::Type,
    software::scaling::{context::Context as Scaler, flag::Flags},
};
use ffmpeg_sys_next::{
    avformat_alloc_context, avformat_find_stream_info, avformat_open_input,
    avio_alloc_context,
};

use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect};

use crate::mux::FifoIO;

const BUFFER_SIZE: usize = 4096;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init().unwrap();

    let mut fifo = Box::new(FifoIO::new(receiver));
    let fifo_ptr = &mut *fifo as *mut FifoIO as *mut c_void;

    let buffer = vec![0u8; BUFFER_SIZE];
    let buffer_ptr = buffer.as_ptr() as *mut c_uchar;
    std::mem::forget(buffer); // FFmpeg owns this memory

    let avio_ctx = unsafe {
        avio_alloc_context(
            buffer_ptr,
            BUFFER_SIZE as c_int,
            0,
            fifo_ptr,
            Some(FifoIO::read_packet),
            None,
            None,
        )
    };
    if avio_ctx.is_null() {
        return Err("Failed to allocate AVIOContext".into());
    }

    let fmt_ctx = unsafe { avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate AVFormatContext".into());
    }

    unsafe {
        (*fmt_ctx).pb = avio_ctx;
    }

    if unsafe {
        avformat_open_input(&mut (fmt_ctx as *mut _), ptr::null(), ptr::null_mut(), ptr::null_mut())
    } < 0
    {
        return Err("Failed to open input from AVIO".into());
    }

    if unsafe { avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    let mut context = unsafe { Input::wrap(fmt_ctx) };

    let input = context
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;

    let video_stream_index = input.index();
    let codec_params = input.parameters();
    let codec_id = codec_params.id();

    let codec = codec::decoder::find(codec_id).ok_or("Decoder not found")?;
    let mut decoder = codec.decoder().video()?; // Needs Decoder trait in scope

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg_next::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("MirrOx", decoder.width() as u32, decoder.height() as u32)
        .position_centered()
        .opengl()
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, decoder.width(), decoder.height())?;

    let mut event_pump = sdl.event_pump()?;
    let mut packet = ffmpeg_next::Packet::empty();

    while context.read_packet(&mut packet).is_ok() {
        if packet.stream() != video_stream_index {
            continue;
        }

        if decoder.send_packet(&packet).is_ok() {
            let mut frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&frame, &mut rgb_frame)?;

                texture.update(
                    None,
                    rgb_frame.data(0),
                    rgb_frame.stride(0),
                )?;

                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width(), decoder.height())))?;
                canvas.present();
            }
        }

        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Ok(());
            }
        }
    }

    Ok(())
}

        }

        if decoder.send_packet(&packet).is_ok() {
            let mut frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                let mut rgb_frame = Video::empty();
                scaler.run(&frame, &mut rgb_frame)?;

                texture.update(
                    None,
                    rgb_frame.data(0),
                    rgb_frame.stride(0),
                )?;

                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width(), decoder.height())))?;
                canvas.present();
            }
        }

        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return Ok(());
            }
        }
    }

    Ok(())
}
