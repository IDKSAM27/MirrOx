use std::fs;

pub fn get_scrcpy_server_version() -> std::io::Result<String> {
    let version_path = "server/version.txt";
    let version = fs::read_to_string(version_path)?
        .trim()
        .to_string();
    Ok(version)
}