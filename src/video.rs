use std::ffi::CString;
use std::ptr;

use crossbeam_channel::Receiver;
use ffmpeg_next as ffmpeg;
use ffmpeg_sys_next::*;
use sdl2::{pixels::PixelFormatEnum, rect::Rect};

use crate::mux::FifoIO;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ffmpeg::init()?;

    // Wrap FifoIO from the channel
    let mut fifo = FifoIO::new(receiver);

    // Allocate AVFormatContext
    let fmt_ctx = unsafe { avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate format context".into());
    }

    // Allocate buffer and AVIOContext
    let buffer_size = 8192;
    let buffer = unsafe { av_malloc(buffer_size) as *mut u8 };
    if buffer.is_null() {
        return Err("Failed to allocate AVIO buffer".into());
    }

    // Setup AVIOContext with custom reader
    let read_cb: Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut u8, i32) -> i32> =
        Some(FifoIO::read_packet);
    let avio_ctx = unsafe {
        avio_alloc_context(
            buffer,
            buffer_size as i32,
            0,
            &mut fifo as *mut _ as *mut _,
            read_cb,
            None,
            None,
        )
    };
    unsafe {
        (*fmt_ctx).pb = avio_ctx;
    }

    // Open input using dummy format
    let input_format = unsafe {
        let name = CString::new("h264")?;
        av_find_input_format(name.as_ptr())
    };
    if input_format.is_null() {
        return Err("Failed to find H264 demuxer".into());
    }

    if unsafe { avformat_open_input(&mut (fmt_ctx as *mut _), ptr::null(), input_format, ptr::null_mut()) } < 0 {
        return Err("Failed to open input format".into());
    }

    if unsafe { avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    let mut stream_index: i32 = -1;
    for i in 0..unsafe { (*fmt_ctx).nb_streams } {
        let stream = unsafe { *(*fmt_ctx).streams.offset(i as isize) };
        if unsafe { (*(*stream).codecpar).codec_type } == AVMediaType::AVMEDIA_TYPE_VIDEO {
            stream_index = i as i32;
            break;
        }
    }

    if stream_index < 0 {
        return Err("No video stream found".into());
    }

    let codecpar = unsafe { &*(*(*(*fmt_ctx).streams.offset(stream_index as isize))).codecpar };
    let decoder = unsafe { avcodec_find_decoder((*codecpar).codec_id) };
    if decoder.is_null() {
        return Err("Decoder not found".into());
    }

    let codec_ctx = unsafe { avcodec_alloc_context3(decoder) };
    if codec_ctx.is_null() {
        return Err("Failed to allocate codec context".into());
    }

    if unsafe { avcodec_parameters_to_context(codec_ctx, codecpar) } < 0 {
        return Err("Failed to copy codec parameters".into());
    }

    if unsafe { avcodec_open2(codec_ctx, decoder, ptr::null_mut()) } < 0 {
        return Err("Failed to open codec".into());
    }

    // SDL2 window setup
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let window = video_subsystem
        .window("MirrOx", unsafe { (*codec_ctx).width } as u32, unsafe { (*codec_ctx).height } as u32)
        .position_centered()
        .opengl()
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::IYUV,
        unsafe { (*codec_ctx).width } as u32,
        unsafe { (*codec_ctx).height } as u32,
    )?;

    let mut packet = unsafe { av_packet_alloc() };
    let mut frame = unsafe { av_frame_alloc() };

    let mut event_pump = sdl.event_pump()?;

    loop {
        while let Some(event) = event_pump.poll_event() {
            use sdl2::event::Event;
            if let Event::Quit { .. } = event {
                return Ok(());
            }
        }

        if unsafe { av_read_frame(fmt_ctx, packet) } < 0 {
            continue;
        }

        if unsafe { avcodec_send_packet(codec_ctx, packet) } >= 0 {
            while unsafe { avcodec_receive_frame(codec_ctx, frame) } == 0 {
                let (w, h) = (unsafe { (*frame).width }, unsafe { (*frame).height });
                let y = unsafe { std::slice::from_raw_parts((*frame).data[0], (w * h) as usize) };
                let u = unsafe { std::slice::from_raw_parts((*frame).data[1], (w * h / 4) as usize) };
                let v = unsafe { std::slice::from_raw_parts((*frame).data[2], (w * h / 4) as usize) };
                texture.update_yuv(None, y, w as usize, u, (w / 2) as usize, v, (w / 2) as usize)?;
                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, w as u32, h as u32)))?;
                canvas.present();
            }
        }

        unsafe { av_packet_unref(packet) };
    }
}
