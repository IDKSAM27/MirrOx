use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

// Constants
const SCRCPY_SERVER_VERSION: &str = "3.2";
const SCRCPY_SERVER_JAR_PATH: &str = "/data/local/tmp/scrcpy-server-v3.2.jar";

pub fn start_scrcpy_server() -> std::io::Result<()> {
    println!("Initializing scrcpy server...");

    // Push the scrcpy-server JAR to the device
    let local_jar_path = format!("server/scrcpy-server-v{}", SCRCPY_SERVER_VERSION);
    let push_status = Command::new("adb")
        .args(["push", &local_jar_path, SCRCPY_SERVER_JAR_PATH])
        .status()?;

    if !push_status.success() {
        eprintln!("❌ Failed to push scrcpy-server JAR to device");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "adb push failed"));
    }

    println!("✅ scrcpy-server pushed to device");

    // Start the scrcpy server via adb shell
    let server_command = format!(
        "CLASSPATH={} app_process / com.genymobile.scrcpy.Server {} scid=12345678 log_level=info audio=false max_size=1920",
        SCRCPY_SERVER_JAR_PATH,
        SCRCPY_SERVER_VERSION
    );

    let mut child = Command::new("adb")
        .args(["shell", &server_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start scrcpy-server");

    // Capture stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[scrcpy-server stdout] {}", line);
                }
            }
        });
    }

    // Capture stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[scrcpy-server stderr] {}", line);
                }
            }
        });
    }

    Ok(())
}
