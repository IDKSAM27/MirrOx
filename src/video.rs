use ffmpeg_next::{
    codec, decoder, format,
    frame::Video,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use ffmpeg_sys_next as ffmpeg_sys;
use sdl2::{
    event::Event,
    pixels::PixelFormatEnum,
    rect::Rect,
    render::TextureAccess,
};
use std::collections::VecDeque;
use std::ffi::CString;
use std::ptr;
use std::time::Duration;

use crossbeam_channel::Receiver;
use crate::mux::FifoIO;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init()?;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("MirrOx", 1280, 720)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::YV12,
        1280,
        720,
    )?;

    let mut event_pump = sdl_context.event_pump()?;

    // Setup AVIOContext with custom reader
    let mut fifo = FifoIO::new(receiver);
    let buffer_size = 4096;
    let avio_ctx_buffer = unsafe { ffmpeg_sys::av_malloc(buffer_size) as *mut u8 };

    let avio = unsafe {
        ffmpeg_sys::avio_alloc_context(
            avio_ctx_buffer,
            buffer_size as i32,
            0,
            &mut fifo as *mut _ as *mut _,
            Some(FifoIO::read_packet),
            None,
            None,
        )
    };

    if avio.is_null() {
        return Err("Failed to create AVIOContext".into());
    }

    let mut fmt_ctx = unsafe { ffmpeg_sys::avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate AVFormatContext".into());
    }

    unsafe { (*fmt_ctx).pb = avio };

    if unsafe {
        ffmpeg_sys::avformat_open_input(&mut fmt_ctx, ptr::null(), ptr::null_mut(), ptr::null_mut())
    } < 0
    {
        return Err("Failed to open input".into());
    }

    if unsafe { ffmpeg_sys::avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    let mut video_stream_index = -1;
    for i in 0..unsafe { (*fmt_ctx).nb_streams } {
        let stream = unsafe { *(*fmt_ctx).streams.add(i as usize) };
        let codecpar = unsafe { *stream.codecpar };
        if codecpar.codec_type == ffmpeg_sys::AVMediaType::AVMEDIA_TYPE_VIDEO {
            video_stream_index = i as i32;
            break;
        }
    }

    if video_stream_index == -1 {
        return Err("No video stream found".into());
    }

    let stream = unsafe { *(*fmt_ctx).streams.add(video_stream_index as usize) };
    let codec_id = unsafe { (*stream.codecpar).codec_id };
    let decoder = codec::decoder::find(codec_id)
        .ok_or("Decoder not found")?
        .open()?;
    let mut context = decoder::Video::from_codec(decoder);

    let mut scaler = Scaler::get(
        context.format(),
        context.width(),
        context.height(),
        Pixel::YUV420P,
        context.width(),
        context.height(),
        Flags::BILINEAR,
    )?;

    let mut decoded = Video::empty();
    let mut rgb_frame = Video::empty();

    'main: loop {
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        let mut packet = ffmpeg_next::Packet::empty();
        if context.receive_frame(&mut decoded).is_ok() {
            scaler.run(&decoded, &mut rgb_frame)?;
            let data = rgb_frame.data(0);

            texture.update(None, data, rgb_frame.stride(0) as usize)?;
            canvas.clear();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, 1280, 720)))?;
            canvas.present();
        } else {
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(())
}
