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
    // Step 1: Push the DAMN JAR
    adb_push(server_jar_path, "/data/local/tmp/scrcpy-server.jar")?;

    // Step 2: Reverse the TCP port (for more info go the notes 10-15/05/2025)
    adb_exec_shell("exit")?; // Trigger ADB startup if needed
    Command::new("adb")
        .args(["reverse", "tcp:27183", "localabstract:scrcpy"])
        .status()?;

    // Step 3: Launch server in a new thread or background process
    Command::new("adb")
        .args([
            "shell",
            "CLASSPATH=/data/local/tmp/scrcpy-server.jar app_process / com.genymobile.scrcpy.Server 3.2 --port=27183 --listen",
        ])
        .spawn()?; // Don't block

    Ok(())
}
