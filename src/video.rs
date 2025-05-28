use std::io::{Read};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use ffmpeg_next::{
    codec,
    format::{self, input},
    media::Type,
    software::scaling::{context::Context as ScalingContext, flag::Flags},
    util::frame::video::Video,
    decoder::Video as VideoDecoder,
};

use sdl2::{pixels::PixelFormatEnum, rect::Rect};

pub fn start_video_stream() -> anyhow::Result<()> {
    println!("[*] Connecting to scrcpy stream...");

    // Connect to scrcpy server over TCP
    let mut stream = TcpStream::connect("127.0.0.1:27183")?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;

    println!("[*] Connected! Reading scrcpy header...");

    // === STEP 1: Read and discard scrcpy header ===
    let mut header = [0u8; 1024]; // generous buffer
    stream.read_exact(&mut header[..1])?; // first byte is version length

    let version_len = header[0] as usize;
    stream.read_exact(&mut header[..version_len])?; // skip version string

    let mut device_name_len = [0u8; 1];
    stream.read_exact(&mut device_name_len)?;
    let name_len = device_name_len[0] as usize;

    stream.read_exact(&mut header[..name_len])?; // skip device name

    let mut resolution = [0u8; 4 * 2];
    stream.read_exact(&mut resolution)?; // width + height

    // === STEP 2: FFmpeg expects input from a file/pipe, so create a pipe ===
    use std::os::unix::io::{FromRawFd, IntoRawFd};
    use os_pipe::pipe;

    let (mut pipe_reader, mut pipe_writer) = pipe()?;

    // === STEP 3: Spawn a thread to read from TcpStream and write raw video to pipe ===
    thread::spawn(move || {
        let mut video_buf = [0u8; 8192];
        loop {
            match stream.read(&mut video_buf) {
                Ok(0) => break,
                Ok(n) => {
                    if pipe_writer.write_all(&video_buf[..n]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // === STEP 4: Decode using FFmpeg from the reader ===
    unsafe {
        let mut ictx = format::input(&mut pipe_reader)?;
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or_else(|| anyhow::anyhow!("No video stream found"))?;

        let video_stream_index = input.index();
        let context_decoder = codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut scaler = ScalingContext::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            PixelFormatEnum::ARGB8888.into(),
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let sdl = sdl2::init()?;
        let video_subsystem = sdl.video()?;
        let window = video_subsystem
            .window("MirrOx", decoder.width() as u32, decoder.height() as u32)
            .position_centered()
            .opengl()
            .build()?;

        let mut canvas = window.into_canvas().build()?;
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, decoder.width(), decoder.height())?;

        let mut decoded = Video::empty();
        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;

                    texture.update(
                        None,
                        rgb_frame.data(0),
                        rgb_frame.stride(0),
                    )?;
                    canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width() as u32, decoder.height() as u32)))?;
                    canvas.present();
                }
            }
        }
    }

    Ok(())
}
