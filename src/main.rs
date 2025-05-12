mod adb;

fn main() {
    println!("Starting MirrOx using scrcpy-server...");

    if let Err(e) = adb::start_scrcpy_server() {
        eprintln!("Failed to start server: {}", e);
    }
}