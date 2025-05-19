use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

pub fn start_scrcpy_server() -> std::io::Result<()> {
    println!("Initializing App_process...");

    // Push the scrcpy-server JAR to the device
    let push_status = Command::new("adb")
        .args(["push", "server/scrcpy-server-v3.2", "/data/local/tmp/"])
        .status()?;

    if !push_status.success() {
        eprintln!("Failed to push scrcpy-server.jar to device");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "adb push failed"));
    }

    // Start the scrcpy server with correct arguments
    // - 1.25 is the protocol version, not the scrcpy version
    // - --listen=tcp://0.0.0.0:27183 sets up the TCP listener

    // let server_command = "CLASSPATH=/data/local/tmp/scrcpy-server.jar app_process / com.genymobile.scrcpy.Server 3.2 --listen=tcp://0.0.0.0:27183"; // 3.2 is the server version, it expects the client version to be same

    let server_command = "CLASSPATH=/data/local/tmp/scrcpy-server-v3.2 app_process / com.genymobile.scrcpy.Server 3.2 --listen=tcp://0.0.0.0:27183"; // 3.2 is the server version, it expects the client version to be same

    let mut child = Command::new("adb")
        .args(["shell", server_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start scrcpy-server");

    // Optionally print the server output for debug
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[scrcpy-server] {}", line);
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[scrcpy-server] {}", line);
                }
            }
        });
    }

    Ok(())
}
