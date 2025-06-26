mod adb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    adb::push_scrcpy_server()?;
    adb::start_scrcpy_server()?;
    adb::forward_port()?;

    println!("Server started and port forwarded. Ready for connection.");

    Ok(())
}

