use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

pub fn start_scrcpy_server(version: &str) -> std::io::Result<()> {
    println!("Initializing server...");

    // Push the scrcpy-server JAR to the device
    let jar_name = format!("scrcpy-server-{}", version);
    let local_path = format!("server/{}", jar_name);
    let device_path = format!("/data/local/tmp/{}.jar", jar_name);

    let push_status = Command::new("adb")
        .args(["push", &local_path, &device_path])
        .status()?;

    if !push_status.success() {
        eprintln!("[*] Failed to push server JAR to device");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "adb push failed"));
    }

    println!("[*] server pushed to device");

    // adb forward tcp:27183 localabstract:scrcpy
    let tcp_port_number = "tcp:27183";
    let local_abstract = "localabstract:scrcpy";
    let forward_tcp = Command::new("adb")
        .args(["forward", tcp_port_number, local_abstract])
        .status()?;

    if !forward_tcp.success() {
        eprintln!("[*] Failed to forward TCP port number");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "adb forward failed"));
    }

    println!("[*] TCP port number forwarded to device");

    // Correct way to start the server using app_process via shell
    let server_command = format!(
        "CLASSPATH={} app_process / com.genymobile.scrcpy.Server {} scid=12345678 log_level=info audio=false max_size=1920",
        device_path,
        version
    );

    let mut child = Command::new("adb")
        .args(["shell", "sh", "-c", &server_command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Capture stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        std::thread::spawn(move || {
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[server stdout] {}", line);
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
                    eprintln!("[server stderr] {}", line);
                }
            }
        });
    }

    Ok(())
}
