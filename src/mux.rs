use std::net::TcpStream;
use std::io::{self, Read};
use std::thread;
use ffmpeg_next as ffmpeg;
use ffmpeg_sys_next as ffmpeg_sys;
use thiserror::Error;
use std::ffi::c_void;

// --- Custom Error Type ---
#[derive(Debug, Error)]
pub enum MuxerError {
    #[error("An FFmpeg error occurred: {0}")]
    FfmpegError(#[from] ffmpeg::Error),

    #[error("An I/O error occurred: {0}")]
    IoError(#[from] io::Error),
}

extern "C" fn read_packet(opaque: *mut c_void, buf: *mut u8, buf_size: i32) -> i32 {
    let pipe_reader = unsafe { &mut *(opaque as *mut Box<dyn Read + Send>) };
    let rust_slice = unsafe { std::slice::from_raw_parts_mut(buf, buf_size as usize) };
    
    match pipe_reader.read(rust_slice) {
        Ok(0) => ffmpeg_sys::AVERROR_EOF,
        Ok(n) => n as i32,
        Err(_) => ffmpeg_sys::AVERROR_UNKNOWN,
    }
}

pub fn bridge_stream(mut stream: TcpStream) -> Result<ffmpeg::format::context::Input, MuxerError> {
    let mut header_buffer = [0u8; 2];
    stream.read_exact(&mut header_buffer)?;
    let device_name_length = header_buffer[1] as usize;
    if device_name_length > 0 {
        let mut device_name_buffer = vec![0u8; device_name_length];
        stream.read_exact(&mut device_name_buffer)?;
        let device_name = String::from_utf8_lossy(&device_name_buffer);
        println!("Connected to device: {}", device_name);
    } else {
        println!("Connected to device (no name provided).");
    }

    let (pipe_reader, mut pipe_writer) = pipe::pipe();
    let mut boxed_reader: Box<dyn Read + Send> = Box::new(pipe_reader);

    thread::Builder::new()
        .name("tcp-to-pipe-bridge".to_string())
        .spawn(move || {
            match io::copy(&mut stream, &mut pipe_writer) {
                Ok(bytes) => println!("Bridge thread finished: copied {} bytes.", bytes),
                Err(e) => eprintln!("Error in bridge thread: {}", e),
            }
        })?;

    unsafe {
        let buffer_size = 4096;
        let buffer = ffmpeg_sys::av_malloc(buffer_size);
        if buffer.is_null() {
            return Err(MuxerError::FfmpegError(ffmpeg::Error::from(ffmpeg_sys::AVERROR(ffmpeg_sys::ENOMEM))));
        }

        let avio_ctx = ffmpeg_sys::avio_alloc_context(
            buffer as *mut u8,
            buffer_size as i32,
            0,
            &mut *boxed_reader as *mut (dyn Read + Send) as *mut c_void,
            Some(read_packet),
            None,
            None,
        );

        if avio_ctx.is_null() {
            // FIX 1: Cast the buffer pointer to *mut c_void for av_free.
            ffmpeg_sys::av_free(buffer as *mut c_void);
            return Err(MuxerError::FfmpegError(ffmpeg::Error::from(ffmpeg_sys::AVERROR(ffmpeg_sys::ENOMEM))));
        }

        let mut av_format_ctx = ffmpeg_sys::avformat_alloc_context();
        if av_format_ctx.is_null() {
            ffmpeg_sys::av_free((*avio_ctx).buffer as *mut c_void);
            ffmpeg_sys::av_free(avio_ctx as *mut c_void);
            return Err(MuxerError::FfmpegError(ffmpeg::Error::from(ffmpeg_sys::AVERROR(ffmpeg_sys::ENOMEM))));
        }
        
        (*av_format_ctx).pb = avio_ctx;

        if ffmpeg_sys::avformat_open_input(&mut av_format_ctx, std::ptr::null(), std::ptr::null_mut(), std::ptr::null_mut()) < 0 {
            // FIX 1 (again): Cast the buffer pointer for av_free on error.
            ffmpeg_sys::av_free((*avio_ctx).buffer as *mut c_void);
            ffmpeg_sys::av_free(avio_ctx as *mut c_void);
            ffmpeg_sys::avformat_free_context(av_format_ctx);
            return Err(MuxerError::FfmpegError(ffmpeg::Error::from(ffmpeg_sys::AVERROR_UNKNOWN)));
        }

        // FIX 2: Use the correct `wrap` function instead of `from_ptr`.
        let ictx = ffmpeg::format::context::Input::wrap(av_format_ctx);
        
        println!("Custom IO stream bridge created. FFmpeg context is ready.");
        std::mem::forget(boxed_reader);
        
        Ok(ictx)
    }
}
