use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec, format, frame::Video, media::Type, software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::{event::Event, pixels::PixelFormatEnum, rect::Rect, render::TextureAccess};
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;
use std::time::Duration;

use crate::mux::{FifoIO};

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init()?;

    // Setup AVIOContext for reading stream from memory via FifoIO
    let mut fifo = Box::new(FifoIO::new(receiver));
    let fifo_ptr = &mut *fifo as *mut _ as *mut c_void;

    let buffer_size = 4096;
    let avio_ctx_buffer = unsafe { ffmpeg_sys_next::av_malloc(buffer_size) as *mut u8 };
    let avio_ctx = unsafe {
        ffmpeg_sys_next::avio_alloc_context(
            avio_ctx_buffer,
            buffer_size as i32,
            0,
            fifo_ptr,
            Some(FifoIO::read_packet),
            None,
            None,
        )
    };

    let mut fmt_ctx = unsafe { ffmpeg_sys_next::avformat_alloc_context() };
    unsafe {
        (*fmt_ctx).pb = avio_ctx;
        (*fmt_ctx).flags |= ffmpeg_sys_next::AVFMT_FLAG_CUSTOM_IO;
    }

    if unsafe { ffmpeg_sys_next::avformat_open_input(&mut fmt_ctx, ptr::null(), ptr::null_mut(), ptr::null_mut()) } != 0 {
        return Err("Failed to open input format context".into());
    }

    if unsafe { ffmpeg_sys_next::avformat_find_stream_info(fmt_ctx, ptr::null_mut()) } < 0 {
        return Err("Failed to find stream info".into());
    }

    // Find video stream
    let mut video_stream_idx = -1;
    for i in 0..unsafe { (*fmt_ctx).nb_streams } {
        let stream = unsafe { *(*fmt_ctx).streams.offset(i as isize) };
        if unsafe { (*stream.codecpar).codec_type } == ffmpeg_sys_next::AVMediaType_AVMEDIA_TYPE_VIDEO {
            video_stream_idx = i as i32;
            break;
        }
    }

    if video_stream_idx == -1 {
        return Err("No video stream found".into());
    }

    let stream = unsafe { *(*fmt_ctx).streams.offset(video_stream_idx as isize) };
    let codec_id = unsafe { (*stream.codecpar).codec_id };
    let decoder = codec::decoder::find(codec_id)
        .ok_or("Decoder not found")?
        .open_as(codec::Id::from(codec_id))?;

    let mut context = decoder;
    let mut scaler = None;

    // SDL2 setup
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("MirrOx - Android Mirror", 1280, 720)
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 1280, 720)?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut packet = unsafe { ffmpeg_sys_next::av_packet_alloc() };
    let mut frame = unsafe { ffmpeg_sys_next::av_frame_alloc() };

    loop {
        while event_pump.poll_iter().next().is_some() {
            if let Some(Event::Quit { .. }) = event_pump.poll_iter().next() {
                break;
            }
        }

        let ret = unsafe { ffmpeg_sys_next::av_read_frame(fmt_ctx, packet) };
        if ret < 0 {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        if unsafe { (*packet).stream_index } == video_stream_idx {
            if context.send_packet(&ffmpeg_next::Packet::wrap(packet)).is_ok() {
                let mut decoded = Video::empty();
                while context.receive_frame(&mut decoded).is_ok() {
                    let width = decoded.width();
                    let height = decoded.height();

                    if scaler.is_none() {
                        scaler = Some(
                            Scaler::get(
                                decoded.format(),
                                width,
                                height,
                                Pixel::RGB24,
                                width,
                                height,
                                Flags::BILINEAR,
                            )?,
                        );
                        texture = texture_creator.create_texture_streaming(
                            PixelFormatEnum::RGB24,
                            width,
                            height,
                        )?;
                    }

                    let mut rgb_frame = Video::empty();
                    scaler.as_mut().unwrap().run(&decoded, &mut rgb_frame)?;

                    texture.update(
                        None,
                        rgb_frame.data(0),
                        rgb_frame.stride(0),
                    )?;

                    canvas.clear();
                    canvas.copy(&texture, None, Some(Rect::new(0, 0, width as u32, height as u32)))?;
                    canvas.present();
                }
            }
        }

        unsafe {
            ffmpeg_sys_next::av_packet_unref(packet);
        }
    }
}
