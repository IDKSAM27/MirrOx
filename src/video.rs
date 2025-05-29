use anyhow::{Context, Result};
use ffmpeg_next::{
    codec,
    decoder::Video as VideoDecoder,
    format,
    frame::Video,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::{pixels::PixelFormatEnum, rect::Rect};
use std::{
    io::Read,
    process::{Command, Stdio},
    thread,
};

pub fn start_video_stream() -> Result<()> {
    ffmpeg_next::init().context("Failed to initialize FFmpeg")?;

    // Launch scrcpy-server on the Android device
    let mut adb_child = Command::new("adb")
        .args(["exec-out", "scrcpy-server"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to launch scrcpy-server via ADB")?;

    let stdout = adb_child
        .stdout
        .take()
        .context("Failed to capture stdout from ADB child")?;

    // Create a temp file for FFmpeg to read from
    let mut temp_file = tempfile::NamedTempFile::new().context("Failed to create temp file")?;
    let temp_path = temp_file.path().to_path_buf();

    // Thread: copy ADB stdout to temp file
    thread::spawn({
        let mut reader = stdout;
        let mut writer = temp_file;
        move || {
            let _ = std::io::copy(&mut reader, &mut writer);
        }
    });

    // Wait a bit to buffer enough data
    thread::sleep(std::time::Duration::from_secs(1));

    let mut ictx = format::input(&temp_path).context("Failed to open input via FFmpeg")?;
    let input = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .context("Couldn't find best video stream")?;

    let video_stream_index = input.index();
    let context_decoder = codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    let sdl = sdl2::init().map_err(|e| anyhow::anyhow!("{}", e))?;
    let video_subsystem = sdl.video().map_err(|e| anyhow::anyhow!("{}", e))?;
    let window = video_subsystem
        .window("MirrOx", decoder.width() as u32, decoder.height() as u32)
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
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            decoder.width() as u32,
            decoder.height() as u32,
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut event_pump = sdl.event_pump().map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut receive_and_render = |frame: &Video| -> Result<()> {
        let mut rgb_frame = Video::empty();
        scaler.run(frame, &mut rgb_frame)?;

        texture
            .with_lock(None, |buffer, pitch| {
                let data = rgb_frame.data(0);
                let linesize = rgb_frame.stride(0);
                for y in 0..decoder.height() {
                    let y_usize = y as usize;
                    let src_start = y_usize * linesize as usize;
                    let dst_start = y_usize * pitch as usize;
                    let row_width = decoder.width() as usize * 3;
                    let src = &data[src_start..src_start + row_width];
                    let dst = &mut buffer[dst_start..dst_start + row_width];
                    dst.copy_from_slice(src);
                }
            })
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        canvas.present();
        Ok(())
    };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                receive_and_render(&decoded)?;

                for event in event_pump.poll_iter() {
                    use sdl2::event::Event;
                    if let Event::Quit { .. } = event {
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}
