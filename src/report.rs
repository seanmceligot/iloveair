use crate::audit::read_to_string_with_shellexpand;
use crate::HumidityTemp;
use anyhow::Result;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

pub fn save_json_report(
    indoor: HumidityTemp,
    outdoor: HumidityTemp,
    can_let_in_humidify: bool,
    humidity_diff: u64,
    can_let_in_temperature: bool,
    temp_diff: f64,
    window_should_be_open: bool,
    json_out_path: &String,
    is_dry_run: bool,
) -> Result<()> {
    let json = json!({
        "indoor": indoor.temp,
        "outdoor": outdoor.temp,
        "can_let_in_humidify": can_let_in_humidify,
        "humidity_diff": humidity_diff,
        "can_let_in_temperature": can_let_in_temperature,
        "temp_diff": temp_diff,
        "window_should_be_open": window_should_be_open,
    });

    // Print to stdout
    println!("{}", json);

    // Write to file if not a dry run
    if !is_dry_run {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(json_out_path)
        {
            if let Err(e) = writeln!(file, "{}", json) {
                eprintln!("Failed to write to file '{}': {}", json_out_path, e);
            }
        } else {
            eprintln!(
                "Failed to open file '{}': {}",
                json_out_path,
                std::io::Error::last_os_error()
            );
        }
    }
    Ok(())
}

pub fn load_json_window_should_be_open(json_path: &String) -> Result<bool> {
    let json = read_to_string_with_shellexpand(json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json)?;
    let window_should_be_open = json["window_should_be_open"].as_bool().unwrap_or(false);
    Ok(window_should_be_open)
}
