use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

pub fn start_scrcpy_server() -> std::io::Result<()> {
    // Push the JAR to /data/local/tmp
    Command::new("adb")
        .args(["Push", "server/scrcpy-server.jar", "/data/local/tmp/scrcpy-server-jar"])
        .status()?;

    // Run it using app_process and capture stdout
    let mut child = Command::new("adb")
        .args([
            "shell",
            "CLASSPATH=/data/local/tmp/scrcpy-server.jar",
            "app_process",
            "/",
            "com.genymobile.scrcpy.Server",
            "3.2", // version string (arbitary but expected)
            "log_level=info", // optional args
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