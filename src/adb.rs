use std::process::{Command,  Stdio};
use std::io::{self, Write};
use std::path::Path;

const SCRCPY_SERVER_PATH: &str = "server/scrcpy-server-v3.2";
const DEVICE_PATH: &str = "/data/local/tmp/scrcpy-server.jar";
const SERVER_VERSION: &str = "1.25"; // I'll have to update this when the server version changes

pub fn push_scrcpy_server() -> io::Result<()> {
    if !Path::new(SCRCPY_SERVER_PATH).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "scrcpy server file not found"));
    }

    Command::new("adb")
        .arg(["push", SCRCPY_SERVER_PATH, DEVICE_PATH])
        .status()?;

    Ok(())
}

pub fn start_scrcpy_server() -> io::Result<()> {
    let mut child = Command::new("adb")
        .args(["shell", "CLASSPATH=/data/local/tmp/scrcpy-server.jar", \
            "app_process", "/", "com.genymobile.scrcpy.Server", SERVER_VERSION])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Not wait here, as scrcpy server runs in the background
    Ok(())
}

