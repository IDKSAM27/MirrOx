mod adb;
mod tcp_client;

fn main() {
    println!("Starting MirrOx using scrcpy-server...");

    if let Err(e) = adb::start_scrcpy_server() {
        eprintln!("Failed to start server: {}", e);
        return;
    }

    // Wait a moment for server to boot up
    std::thread::sleep(std::time::Duration::from_millis(1000));

    if let Err(e) = tcp_client::connect_to_server() {
        eprintln!("Failed to connect to scrcpy-server: {}", e);
    }
}