fn main() {
    let output = r#"Sampreet Patil is my name 
    level: 69"#;

    let test = extract_battery_level(output);
    println!("{}", test);

    fn extract_battery_level(output: &str) -> String {
       for line in output.lines() {
           if let Some(level) = line.trim().strip_prefix("level: ") {
               return level.trim().to_string();
           }
       }
       "Unknown".to_string()
    }
}

