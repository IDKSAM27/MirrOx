mod adb;
mod mux;
mod tcp_client;
mod video;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_version = "v3.2";

    adb::start_scrcpy_server(server_version)?;
    let _tcp_stream = tcp_client::connect_to_server()?; // Just to trigger server handshake

    let video_receiver = mux::start_muxed_stream()?;
    video::start_video_stream(video_receiver)?;

    Ok(())
}
