use std::ffi::CStr;
use std::ptr;

use crossbeam_channel::Receiver;
use sdl2::{pixels::PixelFormatEnum, rect::Rect};
use sdl2::render::TextureAccess;

use crate::mux::FifoIO;
use ffmpeg_sys_next::*;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    unsafe {
        av_register_all();
        avcodec_register_all();
    }

    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let window = video_subsystem
        .window("MirrOx", 1280, 720)
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::IYUV, 1280, 720)?;

    // Create FifoIO
    let mut fifo_io = FifoIO::new(receiver);
    let mut avio_ctx_buffer = vec![0u8; 8192];
    let avio_ctx = unsafe {
        let read_cb = Some(FifoIO::read_packet);
        let opaque = &mut fifo_io as *mut _ as *mut _;
        avio_alloc_context(
            avio_ctx_buffer.as_mut_ptr(),
            avio_ctx_buffer.len() as i32,
            0,
            opaque,
            read_cb,
            None,
            None,
        )
    };

    if avio_ctx.is_null() {
        return Err("Failed to allocate AVIO context".into());
    }

    let fmt_ctx = unsafe { avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate format context".into());
    }

    unsafe { (*fmt_ctx).pb = avio_ctx };

    let h264_format = unsafe { av_find_input_format(CStr::from_bytes_with_nul(b"h264\0")?.as_ptr()) };
    if h264_format.is_null() {
        return Err("Could not find H264 format".into());
    }

    if unsafe { avformat_open_input(&mut (fmt_ctx as *mut _), ptr::null(), h264_format, ptr::null_mut()) } < 0 {
        return Err("Failed to open input from custom AVIO".into());
    }

    if unsafe { avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    let mut stream_index = -1;
    for i in 0..unsafe { (*fmt_ctx).nb_streams } {
        let stream = unsafe { *(*fmt_ctx).streams.offset(i as isize) };
        let codecpar = unsafe { &*(*stream).codecpar };
        if codecpar.codec_type == AVMediaType::AVMEDIA_TYPE_VIDEO {
            stream_index = i as i32;
            break;
        }
    }

    if stream_index == -1 {
        return Err("No video stream found".into());
    }

    let codecpar = unsafe {
        &*(*(*fmt_ctx).streams.offset(stream_index as isize)).codecpar
    };
    let codec = unsafe { avcodec_find_decoder(codecpar.codec_id) };
    if codec.is_null() {
        return Err("Decoder not found".into());
    }

    let codec_ctx = unsafe { avcodec_alloc_context3(codec) };
    if codec_ctx.is_null() {
        return Err("Failed to allocate codec context".into());
    }

    if unsafe { avcodec_parameters_to_context(codec_ctx, codecpar) } < 0 {
        return Err("Failed to copy codec parameters".into());
    }

    if unsafe { avcodec_open2(codec_ctx, codec, ptr::null_mut()) } < 0 {
        return Err("Failed to open codec".into());
    }

    let mut pkt = unsafe { std::mem::zeroed::<AVPacket>() };
    let frame = unsafe { av_frame_alloc() };

    while unsafe { av_read_frame(fmt_ctx, &mut pkt) } >= 0 {
        if pkt.stream_index == stream_index {
            if unsafe { avcodec_send_packet(codec_ctx, &pkt) } >= 0 {
                while unsafe { avcodec_receive_frame(codec_ctx, frame) } == 0 {
                    let w = unsafe { (*frame).width } as u32;
                    let h = unsafe { (*frame).height } as u32;
                    texture.update_yuv(
                        Rect::new(0, 0, w, h),
                        unsafe { std::slice::from_raw_parts((*frame).data[0], (*frame).linesize[0] as usize * h as usize) },
                        (*frame).linesize[0] as usize,
                        unsafe { std::slice::from_raw_parts((*frame).data[1], (*frame).linesize[1] as usize * (h as usize / 2)) },
                        (*frame).linesize[1] as usize,
                        unsafe { std::slice::from_raw_parts((*frame).data[2], (*frame).linesize[2] as usize * (h as usize / 2)) },
                        (*frame).linesize[2] as usize,
                    )?;
                    canvas.clear();
                    canvas.copy(&texture, None, None)?;
                    canvas.present();
                }
            }
        }
        unsafe { av_packet_unref(&mut pkt) };
    }

    unsafe {
        av_frame_free(&mut (frame as *mut _));
        avcodec_free_context(&mut (codec_ctx as *mut _));
        avformat_close_input(&mut (fmt_ctx as *mut _));
        av_free((*avio_ctx).buffer as *mut _);
        avio_context_free(&mut (avio_ctx as *mut _));
    }

    Ok(())
}
