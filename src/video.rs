use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec::{self, decoder::Video as VideoDecoder},
    format::{self, context::Input},
    frame::Video,
    media::Type,
    software::scaling::{context::Context as Scaler, flag::Flags},
    util::format::pixel,
    Packet,
};
use sdl2::{event::Event, pixels::PixelFormatEnum, render::Canvas, video::Window, EventPump};

use crate::mux::FifoIO;

pub fn start_video_stream(
    receiver: Receiver<u8>,
    canvas: &mut Canvas<Window>,
    event_pump: &mut EventPump,
) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg_next::init()?;

    let mut fifo_io = FifoIO::new(receiver);
    let mut fmt_ctx = format::input(&mut fifo_io)?;

    let input = fmt_ctx.streams().best(Type::Video).ok_or("No video stream found")?;
    let stream_index = input.index();

    let codec_params = input.parameters();
    let decoder_codec = codec::decoder::find(codec_params.codec_id())
        .ok_or("Decoder not found")?;

    let mut decoder: VideoDecoder = VideoDecoder::from_context(codec_params.clone())?;

    let width = decoder.width();
    let height = decoder.height();
    let src_format = decoder.format();

    let dst_format = pixel::Pixel::RGB24;
    let mut scaler = Scaler::get(src_format, width, height, dst_format, width, height, Flags::BILINEAR)?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, width, height)?;

    let mut decoded = Video::empty();
    for packet in fmt_ctx.packets() {
        let packet = packet?;
        if packet.stream() != stream_index {
            continue;
        }

        decoder.send_packet(&packet)?;
        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            scaler.run(&decoded, &mut rgb_frame)?;

            texture.update(None, rgb_frame.data(0), rgb_frame.stride(0))?;
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();

            for event in event_pump.poll_iter() {
                if let Event::Quit { .. } = event {
                    return Ok(());
                }
            }
        }
    }

    decoder.send_eof()?;
    Ok(())
}
