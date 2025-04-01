// use sdl2::pixels::PixelFormatEnum;
// use sdl2::rect::Rect;
// use sdl2::render::TextureAccess;
// use sdl2::video::Window;
// use sdl2::Sdl;
// use std::sync::mpsc::Receiver;

// pub fn start_gui(rx: Receiver<Vec<u8>>) -> Result<(), String> {
//     let sdl_context = sdl2::init()?;
//     let video_subsystem = sdl_context.video()?;
//     let window = video_subsystem
//         .window("MirrOx", 800, 600)
//         .position_centered()
//         .build()
//         .map_err(|e| e.to_string())?;

//     let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
//     let texture_creator = canvas.texture_creator();
//     let mut texture = texture_creator
//         .create_texture_streaming(PixelFormatEnum::RGB24, 800, 600)
//         .map_err(|e| e.to_string())?;

//     let mut event_pump = sdl_context.event_pump()?;

//     'running: loop {
//         if let Ok(frame) = rx.try_recv() {
//             texture.update(None, &frame, 800 * 3).unwrap();
//             canvas.copy(&texture, None, Some(Rect::new(0, 0, 800, 600)))?;
//             canvas.present();
//         }

//         for event in event_pump.poll_iter() {
//             use sdl2::event::Event;
//             match event {
//                 Event::Quit { .. } => break 'running,
//                 _ => {}
//             }
//         }
//     }
//     Ok(())
// }


use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
// use sdl2::render::TextureAccess;
// use sdl2::video::Window;
// use sdl2::Sdl;
use tokio::sync::broadcast::Receiver;

pub async fn start_gui(mut rx: Receiver<Vec<u8>>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("MirrOx", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 800, 600)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        // Check for a new frame
        if let Ok(frame) = rx.try_recv() {
            texture.update(None, &frame, 800 * 3).unwrap();
            canvas.copy(&texture, None, Some(Rect::new(0, 0, 800, 600)))?;
            canvas.present();
        }

        // Handle quit event
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10)); // Prevent CPU overuse
    }
    Ok(())
}

