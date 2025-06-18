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
