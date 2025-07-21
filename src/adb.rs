// src/adb.rs

use std::process::{Command, Stdio, Child};
use std::path::{Path, PathBuf};
use thiserror::Error; // A great crate for creating custom errors. Add `thiserror = "1.0"` to Cargo.toml

// --- Custom Error Enum ---
// Provides specific, actionable error information to the caller.
#[derive(Debug, Error)]
pub enum AdbError {
    #[error("`adb` command not found in PATH. Is Android SDK Platform Tools installed?")]
    AdbNotFound(#[source] std::io::Error),

    #[error("Scrcpy server binary not found at path: {0}")]
    ServerBinaryNotFound(PathBuf),
    
    #[error("Failed to execute adb command: {command}")]
    CommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("ADB command executed but was not successful: {command}")]
    CommandNotSuccessful {
        command: String,
        stdout: String,
        stderr: String,
    },

    #[error("Failed to kill server process on drop: {0}")]
    CleanupFailed(#[source] std::io::Error),
}


// --- Configuration Struct ---
// Decouples the session logic from the configuration data.
#[derive(Debug, Clone)]
pub struct AdbConfig {
    pub scrcpy_server_path: PathBuf,
    pub device_server_path: String,
    pub server_version: String,
    pub local_port: u16,
    pub device_socket_name: String,
}

impl Default for AdbConfig {
    fn default() -> Self {
        AdbConfig {
            scrcpy_server_path: PathBuf::from("server/scrcpy-server-v3.2"),
            device_server_path: "/data/local/tmp/scrcpy-server-v3.2".to_string(),
            server_version: "3.2".to_string(),
            local_port: 27183,
            device_socket_name: "localabstract:scrcpy".to_string(),
        }
    }
}

// --- The Session Manager ---
// Its single responsibility is managing the scrcpy server lifecycle.
pub struct AdbSession {
    config: AdbConfig,
    server_process: Child,
}

impl AdbSession {
    pub fn new(config: AdbConfig) -> Result<Self, AdbError> {
        // Check for server binary before doing anything else
        if !config.scrcpy_server_path.exists() {
            return Err(AdbError::ServerBinaryNotFound(config.scrcpy_server_path));
        }

        Self::push_server(&config)?;
        Self::forward_port(&config)?;
        let server_process = Self::start_server(&config)?;
        
        // Give the server a moment to initialize on the device
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(AdbSession { config, server_process })
    }

    fn push_server(config: &AdbConfig) -> Result<(), AdbError> {
        println!("Pushing scrcpy-server to device...");
        let command_str = format!("adb push {} {}", config.scrcpy_server_path.display(), config.device_server_path);
        
        let output = Command::new("adb")
            .args(["push", config.scrcpy_server_path.to_str().unwrap(), &config.device_server_path])
            .output()
            .map_err(|e| AdbError::AdbNotFound(e))?;

        if !output.status.success() {
            return Err(AdbError::CommandNotSuccessful { 
                command: command_str,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(())
    }

    fn forward_port(config: &AdbConfig) -> Result<(), AdbError> {
        println!("Forwarding port {}...", config.local_port);
        let command_str = format!("adb forward tcp:{} {}", config.local_port, config.device_socket_name);

        let output = Command::new("adb")
            .args(["forward", &format!("tcp:{}", config.local_port), &config.device_socket_name])
            .output()
            .map_err(|e| AdbError::AdbNotFound(e))?;
            
        if !output.status.success() {
            return Err(AdbError::CommandNotSuccessful {
                command: command_str,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(())
    }

    fn start_server(config: &AdbConfig) -> Result<Child, AdbError> {
        println!("Starting scrcpy-server on device...");
        let command_str = "adb shell CLASSPATH=...".to_string(); // Simplified for error reporting

        Command::new("adb")
            .args([
                "shell",
                &format!("CLASSPATH={}", config.device_server_path),
                "app_process",
                "/",
                "com.genymobile.scrcpy.Server",
                &config.server_version,
                "tunnel_forward=true",
                "log_level=info",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| AdbError::CommandFailed { command: command_str, source: e })
    }
}

// RAII: Cleanup is handled automatically when AdbSession goes out of scope.
impl Drop for AdbSession {
    fn drop(&mut self) {
        println!("Cleaning up ADB session...");

        // Kill the server process on the device
        if let Err(e) = self.server_process.kill() {
            // In a drop implementation, we shouldn't panic. We just log the error.
            eprintln!("Failed to kill scrcpy-server process: {}", e);
        }
        // Wait for the process to ensure it's terminated
        let _ = self.server_process.wait();

        // Remove the port forwarding rule
        let forward_arg = format!("tcp:{}", self.config.local_port);
        let output = Command::new("adb")
            .args(["forward", "--remove", &forward_arg])
            .output();

        if let Err(e) = output {
            eprintln!("Failed to execute adb forward --remove command: {}", e);
        }
    }
}
