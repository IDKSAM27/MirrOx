mod adb;
mod mux;
mod tcp_stream;
mod video;

use mux::FifoIO;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup ADB and forward port
    adb::start_scrcpy_server()?;
    adb::adb_forward_port()?;

    // Connect to TCP stream
    let stream = tcp_stream::connect_scrcpy()?;
    let (sender, receiver) = crossbeam_channel::bounded(100);

    // Start mux thread
    std::thread::spawn(move || {
        if let Err(e) = mux::start_mux(stream, sender) {
            eprintln!("[mux] Error: {e}");
        }
    });

    // Setup SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("MirrOx", 1280, 720)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let mut event_pump = sdl_context.event_pump()?;

    // Start video decoding and rendering
    video::start_video_stream(receiver, &mut canvas, &mut event_pump)?;
    Ok(())
}
