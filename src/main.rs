mod adb;
mod mux;
mod video;
mod tcp_client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start server
    adb::start_scrcpy_server("v3.2")?;

    // 2. Connect TCP and send "client hello"
    let _tcp = tcp_client::connect_to_server()?; // sends client hello

    // 3. Begin demuxing the stream
    let receiver = mux::start_muxed_stream()?;

    // 4. Begin decoding and rendering
    video::start_video_stream(receiver)?;

    Ok(())
}
