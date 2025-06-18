use std::os::raw::{c_int, c_uchar, c_void};

use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec,
    codec::traits::Decoder,
    format::{self, context::Input},
    frame::Video,
    media::Type,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel::Pixel,
};
use sdl2::{pixels::PixelFormatEnum, render::Canvas, video::Window, EventPump};

use crate::mux::FifoIO;

pub fn start_video_stream(
    receiver: Receiver<u8>,
    canvas: &mut Canvas<Window>,
    _event_pump: &mut EventPump,
) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init()?;

    let mut fifo_io = FifoIO::new(receiver);
    let mut fmt_ctx = fifo_io.open_format_context()?;

    fmt_ctx.find_stream_info(None)?;

    let input_stream = fmt_ctx
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;

    let codec_params = input_stream.parameters();
    let decoder_codec = codec::decoder::find(codec_params.id()).ok_or("Codec not found")?;

    let mut decoder = decoder_codec.decoder().video()?;
    decoder.open_with(codec_params)?;

    let width = decoder.width();
    let height = decoder.height();
    let src_format = decoder.format();

    let mut scaler = Scaler::get(
        src_format,
        width,
        height,
        Pixel::RGBA,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA32,
        width,
        height,
    )?;

    for (stream, packet) in fmt_ctx.packets() {
        if stream.index() != input_stream.index() {
            continue;
        }

        decoder.send_packet(&packet)?;

        let mut decoded = Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb = Video::empty();
            scaler.run(&decoded, &mut rgb)?;

            let data = rgb.data(0);
            let stride = rgb.stride(0);

            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..height {
                    let src_row = &data[(y * stride) as usize..(y * stride + width * 4) as usize];
                    let dst_row = &mut buffer[(y * pitch) as usize..(y * pitch + width * 4) as usize];
                    dst_row.copy_from_slice(src_row);
                }
            })?;

            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }
    }

    decoder.send_eof()?;
    Ok(())
}
