pub fn extract_battery_level(output: &str) -> String {
    for line in output.lines() {
        if let Some(level) = line.trim().strip_prefix("level: ") {
            return level.trim().to_string();
        }
    }
    "Unknown".to_string()
}
