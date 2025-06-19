mod adb;
mod mux;
mod tcp_stream;
mod video;

use crossbeam_channel::unbounded;
use mux::start_muxing;
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::Sdl;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start scrcpy server
    adb::start_scrcpy_server("3.2")?;
    adb::adb_forward_port()?; // forwards 27183

    // Connect to server (starts stream)
    let stream = tcp_stream::connect_scrcpy()?;

    // Set up SDL2
    let sdl_context: Sdl = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("MirrOx", 1280, 720)
        .position_centered()
        .resizable()
        .opengl()
        .build()?;
    let mut canvas: Canvas<Window> = window.into_canvas().present_vsync().build()?;
    let mut event_pump = sdl_context.event_pump()?;

    // Channel for mux -> video
    let (sender, receiver) = unbounded();

    // Start muxing thread
    start_muxing(stream, sender);

    // Start video loop
    video::start_video_stream(receiver, &mut canvas, &mut event_pump)?;

    Ok(())
}
