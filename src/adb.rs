use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::path::Path;

const SCRCPY_SERVER_PATH: &str = "assets/scrcpy-server.jar";
const DEVICE_PATH: &str = "/data/local/tmp/scrcpy-server.jar";
const SERVER_VERSION: &str = "1.25"; // I'll have to adjust according to the latest server version, later will write a script for the same.

pub fn push_scrcpy_server() -> io::Result<()> {
    if !Path::new(SCRCPY_SERVER_PATH).exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "scrcpy-server.jar not found"));
    }

    Command::new("adb")
        .args(["push", SCRCPY_SERVER_PATH, DEVICE_PATH])
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

    // We do not wait here, as scrcpy-server runs continuously
    Ok(())
}

pub fn forward_port() -> io::Result<()> {
    Command::new("adb")
        .args(["forward", "tcp:27183", "localabstract:scrcpy"])
        .status()?;

    Ok(())
}
