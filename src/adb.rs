use std::process::{Command};

pub fn adb_push(local_path: &str, remote_path: &str) -> std::io::Result<()> {
    let status = Command::new("adb")
        .args(["push", local_path, remote_path])
        .status()?;

    if !status.success() {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "adb push failed"))
    } else {
        Ok(())
    }
}

pub fn adb_exec_shell(cmd: &str) -> std::io::Result<()> {
    let status = Command::new("adb")
        .args(["shell", cmd])
        .status()?;

    if !status.success() {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "adb shell command failed"))
    } else {
        Ok(())
    }
}

pub fn adb_start_server(server_jar_path: &str) -> std::io::Result<()> {
    adb_push(server_jar_path, "/data/local/tmp/scrcpy-server.jar")?;
    adb_exec_shell("CLASSPATH=/data/local/tmp/scrcpy-server.jar app_process / com.genymobile.scrcpy.Server 3.2")?;
    Ok(())
}
