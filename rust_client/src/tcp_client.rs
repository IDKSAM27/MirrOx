use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::io; // <- Removed unused `ErrorKind`

pub fn start_client() -> Result<(), std::io::Error> {
    let sdl_context = sdl2::init().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let video_subsystem = sdl_context.video().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let window = video_subsystem
        .window("MirrOx Client", 1080, 2400)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 1080, 2400)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut event_pump = sdl_context.event_pump().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    println!("Connecting to server...");
    let mut stream = TcpStream::connect("192.168.1.100:8080")?; // Change IP

    println!("Setting read timeout...");
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;

    let mut buffer = vec![0u8; 1024 * 64]; // 64KB buffer
    println!("Entering main loop...");

    'running: loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                println!("Received {} bytes", size);
                size
            }
            Ok(_) => {
                println!("Server sent empty data");
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("No data yet, retrying...");
                continue;
            }
            Err(e) => {
                eprintln!("Failed to read from server: {}", e);
                break;
            }
        };

        // Decode the image if it's a PNG
        if buffer.starts_with(&[137, 80, 78, 71]) {
            match ImageReader::new(Cursor::new(&buffer[..bytes_read]))
                .with_guessed_format()
                .unwrap()
                .decode()
            {
                Ok(img) => {
                    let rgb_img = img.into_rgb8();
                    texture.update(None, &rgb_img, 1080 * 3).unwrap();
                    canvas.copy(&texture, None, Some(Rect::new(0, 0, 1080, 2400)))
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    canvas.present();
                }
                Err(e) => eprintln!("Failed to decode PNG: {}", e),
            }
        }

        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}
