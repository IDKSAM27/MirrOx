use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};

pub fn start_scrcpy_server() -> std::io::Result<()> {
    // Push server JAR
    Command::new("adb")
        .args(["push", "server/scrcpy-server.jar", "/data/local/tmp/scrcpy-server.jar"])
        .status()?;

    // Set up reverse tunnel (server will listen on localabstract:scrcpy)
    Command::new("adb")
        .args(["reverse", "localabstract:scrcpy", "tcp:27183"])
        .status()?;

    // Start the server via app_process
    let mut child = Command::new("adb")
        .args([
            "shell",
            "CLASSPATH=/data/local/tmp/scrcpy-server.jar",
            "app_process",
            "/",
            "com.genymobile.scrcpy.Server",
            // "1.25",
            "3.2",
            "log_level=info",
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        println!("[scrcpy-server] {}", line?);
    }

    Ok(())
}
