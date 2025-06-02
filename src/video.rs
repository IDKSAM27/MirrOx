use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use ffmpeg::{
    codec, frame::Video, media, software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::pixels::PixelFormatEnum;
use std::{
    ffi::c_void,
    io::{Read},
    process::{Command, Stdio},
    thread,
};
use crossbeam_channel::{bounded, Receiver};
use ffmpeg_sys_next as ffmpeg_sys;

struct FifoIO {
    rx: Receiver<u8>,
    buffer: Vec<u8>,
}

impl Read for FifoIO {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        while self.buffer.len() < out.len() {
            match self.rx.recv() {
                Ok(byte) => self.buffer.push(byte),
                Err(_) => break,
            }
        }
        let n = std::cmp::min(out.len(), self.buffer.len());
        out[..n].copy_from_slice(&self.buffer[..n]);
        self.buffer.drain(..n);
        Ok(n)
    }
}

unsafe extern "C" fn read_packet(
    opaque: *mut c_void,
    buf: *mut u8,
    buf_size: i32,
) -> i32 {
    let fifo: &mut FifoIO = &mut *(opaque as *mut FifoIO);
    let out_buf = std::slice::from_raw_parts_mut(buf, buf_size as usize);

    match fifo.read(out_buf) {
        Ok(n) => n as i32,
        Err(_) => ffmpeg_sys::AVERROR_EOF,
    }
}

pub fn start_video_stream() -> Result<()> {
    ffmpeg::init().context("Failed to initialize FFmpeg")?;

    let (tx, rx) = bounded::<u8>(1024 * 1024);

    let mut adb_child = Command::new("adb")
        .args(["exec-out", "scrcpy-server"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start scrcpy-server")?;

    let mut stdout = adb_child.stdout.take().context("Failed to capture ADB stdout")?;

    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while let Ok(n) = stdout.read(&mut buf) {
            if n == 0 {
                break;
            }
            for &byte in &buf[..n] {
                if tx.send(byte).is_err() {
                    break;
                }
            }
        }
    });

    // Create AVIOContext
    let fifo = Box::new(FifoIO { rx, buffer: Vec::new() });
    let fifo_ptr = Box::into_raw(fifo);

    let buffer_size = 4096;
    let buffer = unsafe { ffmpeg_sys::av_malloc(buffer_size) as *mut u8 };
    let avio_ctx = unsafe {
        ffmpeg_sys::avio_alloc_context(
            buffer,
            buffer_size as i32,
            0,
            fifo_ptr as *mut _,
            Some(read_packet),
            None,
            None,
        )
    };

    if avio_ctx.is_null() {
        return Err(anyhow::anyhow!("Failed to allocate AVIO context"));
    }

    let fmt_ctx = unsafe { ffmpeg_sys::avformat_alloc_context() };
    if fmt_ctx.is_null() {
        return Err(anyhow::anyhow!("Failed to allocate format context"));
    }

    unsafe {
        (*fmt_ctx).pb = avio_ctx;
        (*fmt_ctx).flags |= ffmpeg_sys::AVFMT_FLAG_CUSTOM_IO;
    }

    if unsafe { ffmpeg_sys::avformat_open_input(&mut (fmt_ctx as *mut _), std::ptr::null(), std::ptr::null_mut(), std::ptr::null_mut()) } != 0 {
        return Err(anyhow::anyhow!("Failed to open custom AV input"));
    }

    if unsafe { ffmpeg_sys::avformat_find_stream_info(fmt_ctx, std::ptr::null_mut()) } < 0 {
        return Err(anyhow::anyhow!("Failed to find stream info"));
    }

    let mut ictx = unsafe { ffmpeg::format::context::Input::wrap(fmt_ctx) };   

    let input = ictx
        .streams()
        .best(media::Type::Video)
        .context("Couldn't find best video stream")?;

    let video_stream_index = input.index();
    let context_decoder = codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let width = decoder.width();
    let height = decoder.height();
    let src_format = decoder.format();

    let mut scaler = Scaler::get(
        src_format,
        width,
        height,
        Pixel::RGB24,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let sdl = sdl2::init().map_err(|e| anyhow::anyhow!("{}", e))?;
    let video_subsystem = sdl.video().map_err(|e| anyhow::anyhow!("{}", e))?;
    let window = video_subsystem
        .window("MirrOx", width as u32, height as u32)
        .position_centered()
        .build()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, width as u32, height as u32)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut event_pump = sdl.event_pump().map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut rgb_frame = Video::empty();

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                scaler.run(&decoded, &mut rgb_frame)?;

                texture
                    .with_lock(None, |buffer, pitch| {
                        let data = rgb_frame.data(0);
                        let linesize = rgb_frame.stride(0);
                        for y in 0..height {
                            let y_usize = y as usize;
                            let src_start = y_usize * linesize as usize;
                            let dst_start = y_usize * pitch as usize;
                            let row_width = width as usize * 3;
                            let src = &data[src_start..src_start + row_width];
                            let dst = &mut buffer[dst_start..dst_start + row_width];
                            dst.copy_from_slice(src);
                        }
                    })
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                canvas.clear();
                canvas.copy(&texture, None, None).map_err(|e| anyhow::anyhow!("{}", e))?;
                canvas.present();

                use sdl2::event::Event;
                for event in event_pump.poll_iter() {
                    if let Event::Quit { .. } = event {
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}
