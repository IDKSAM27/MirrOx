mod adb;
mod utils;
mod tcp_client;
mod video;

fn main() {
    println!("Starting MirrOx Server...");

    let version = utils::get_scrcpy_server_version()
        .expect("[*] Could not read server version from server/version.txt");

    if let Err(e) = adb::start_scrcpy_server(&version) {
        eprintln!("Failed to start server: {e}");
        return;
    }

    println!("Connecting to server on localhost:27183...");

    match tcp_client::connect_to_scrcpy() {
        Ok(_) => println!("[*] Connected to server successfully."),
        Err(e) => eprintln!("Failed to connect to server: {e}"),
    }

    video::start_video_streaming().unwrap();
}
