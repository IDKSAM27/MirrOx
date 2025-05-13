mod adb;
mod tcp_client;

fn main() {
    println!("Starting MirrOx using scrcpy-server...");

    let server_jar = "server/scrcpy-server.jar";

    if let Err(e) = adb::adb_start_server(server_jar) {
        eprintln!("Failed to start scrcpy-server: {e}");
        return;
    }

    println!("Connecting to scrcpy server on localhost:27183...");

    match tcp_client::connect_to_scrcpy() {
        Ok(_) => println!("Connected to scrcpy-server successfully."),
        Err(e) => eprintln!("Failed to connect to scrcpy-server: {e}"),
    }
}
