mod adb;
mod mux;
mod video;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start scrcpy server over ADB
    adb::start_scrcpy_server("v3.2")?;

    // 2. Delay a bit to allow the server to boot
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // 3. Start TCP stream and get video byte receiver
    let receiver = mux::start_muxed_stream()?;

    // 4. Start decoding and rendering
    video::start_video_stream(receiver)?;

    Ok(())
}
