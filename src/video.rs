use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Read

use crossbeam_channel::Receiver;
use ffmpeg_sys::*;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use crate::mux::FifoIO;

pub fn start_video_stream(receiver: Receiver<Vec<u8>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    unsafe {
        av_register_all();
        avcodec_register_all();
        avformat_network_init();
    }

    let io_buffer_size = 32768;
    let fifo = Arc::new(Mutex::new(FifoIO::new()));

    let reader_fifo = fifo.clone();
    thread::spawn(move || {
        while let Ok(data) = receiver.recv() {
            let mut fifo = reader_fifo.lock().unwrap();
            fifo.push_data(&data);
        }
    });

    // Allocate buffer for AVIOContext
    let avio_buffer = unsafe { av_malloc(io_buffer_size) as *mut u8 };
    if avio_buffer.is_null() {
        return Err("Failed to allocate AVIO buffer".into());
    }

    extern "C" fn read_packet(
        opaque: *mut c_void,
        buf: *mut u8,
        buf_size: c_int,
    ) -> c_int {
        let fifo = unsafe { &mut *(opaque as *mut Arc<Mutex<FifoIO>>) };
        let mut buffer = vec![0u8; buf_size as usize];
        match fifo.lock().unwrap().read(&mut buffer) {
            Ok(n) if n > 0 => {
                unsafe {
                    std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf, n);
                }
                n as c_int
            }
            _ => AVERROR_EAGAIN,
        }
    }

    let opaque_fifo = Box::new(fifo);
    let opaque_ptr = Box::into_raw(opaque_fifo) as *mut c_void;

    let avio_ctx = unsafe {
        avio_alloc_context(
            avio_buffer,
            io_buffer_size,
            0,
            opaque_ptr,
            Some(read_packet),
            None,
            None,
        )
    };
    if avio_ctx.is_null() {
        return Err("Failed to create AVIO context".into());
    }

    // Allocate AVFormatContext and set custom AVIO
    let mut fmt_ctx = unsafe { avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err("Failed to allocate AVFormatContext".into());
    }
    unsafe {
        (*fmt_ctx).pb = avio_ctx;
    }

    // Use "h264" demuxer explicitly
    let input_format_name = CString::new("h264")?;
    let input_format = unsafe { av_find_input_format(input_format_name.as_ptr()) };
    if input_format.is_null() {
        return Err("Failed to find H264 demuxer".into());
    }

    // Open input
    let res = unsafe {
        avformat_open_input(&mut fmt_ctx, ptr::null(), input_format, ptr::null_mut())
    };
    if res < 0 {
        return Err(format!("Failed to open input: {}", res).into());
    }

    // Find stream info (optional for raw H264 but kept for future formats)
    unsafe {
        avformat_find_stream_info(fmt_ctx, ptr::null_mut());
    }

    // Find video stream index
    let mut stream_index = -1;
    unsafe {
        for i in 0..(*fmt_ctx).nb_streams {
            let stream = *(*fmt_ctx).streams.offset(i as isize);
            if (*(*stream).codecpar).codec_type == AVMediaType::AVMEDIA_TYPE_VIDEO {
                stream_index = i as i32;
                break;
            }
        }
    }

    if stream_index == -1 {
        return Err("No video stream found".into());
    }

    // Get codec parameters and find decoder
    let codecpar = unsafe { (*(*(*fmt_ctx).streams.offset(stream_index as isize))).codecpar };
    let codec = unsafe { avcodec_find_decoder((*codecpar).codec_id) };
    if codec.is_null() {
        return Err("Codec not found".into());
    }

    // Create codec context and copy parameters
    let codec_ctx = unsafe { avcodec_alloc_context3(codec) };
    if codec_ctx.is_null() {
        return Err("Failed to allocate codec context".into());
    }
    let ret = unsafe { avcodec_parameters_to_context(codec_ctx, codecpar) };
    if ret < 0 {
        return Err("Failed to copy codec params".into());
    }

    if unsafe { avcodec_open2(codec_ctx, codec, ptr::null_mut()) } < 0 {
        return Err("Failed to open codec".into());
    }

    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("MirrOx Video", 640, 480)
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::IYUV,
        640,
        480,
    )?;

    let mut event_pump = sdl.event_pump()?;

    // Frame decoding loop
    let packet = unsafe { av_packet_alloc() };
    let frame = unsafe { av_frame_alloc() };

    loop {
        while let Some(event) = event_pump.poll_iter().next() {
            use sdl2::event::Event;
            if matches!(event, Event::Quit { .. }) {
                break;
            }
        }

        if unsafe { av_read_frame(fmt_ctx, packet) } < 0 {
            continue;
        }

        if unsafe { (*packet).stream_index } != stream_index {
            unsafe { av_packet_unref(packet) };
            continue;
        }

        if unsafe { avcodec_send_packet(codec_ctx, packet) } < 0 {
            continue;
        }
        unsafe { av_packet_unref(packet) };

        while unsafe { avcodec_receive_frame(codec_ctx, frame) } == 0 {
            let (width, height) = (unsafe { (*frame).width }, unsafe { (*frame).height });
            texture.update_yuv(
                None,
                unsafe { std::slice::from_raw_parts((*frame).data[0], (*frame).linesize[0] as usize * height as usize) },
                (*frame).linesize[0] as usize,
                unsafe { std::slice::from_raw_parts((*frame).data[1], (*frame).linesize[1] as usize * height as usize / 2) },
                (*frame).linesize[1] as usize,
                unsafe { std::slice::from_raw_parts((*frame).data[2], (*frame).linesize[2] as usize * height as usize / 2) },
                (*frame).linesize[2] as usize,
            )?;
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }
    }
}
