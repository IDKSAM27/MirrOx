mod adb;
mod tcp_client;

fn main() {
    println!("Starting MirrOx Server...");

    if let Err(e) = adb::start_scrcpy_server() {
        eprintln!("Failed to start server: {e}");
        return;
    }

    println!("Connecting to server on localhost:27183...");

    match tcp_client::connect_to_scrcpy() {
        Ok(_) => println!("[*] Connected to server successfully."),
        Err(e) => eprintln!("Failed to connect to server: {e}"),
    }
}
