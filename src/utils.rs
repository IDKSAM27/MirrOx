pub fn extract_battery_level(output: &str) -> String {
    for line in output.lines() {
        if let Some(level) = line.trim().strip_prefix("level: ") {  // line.trim() was not present, hence it was not able to trim the leading white spaces in the total battery info
            return level.trim().to_string();
        }
    }
    "Unknown".to_string()
}
