use std::fs;

use anyhow::{Context, Result};
use fs::File;
use iloveair::config::get_indoor_path;
use iloveair::config::get_open_windows_path;
use iloveair::config::get_weather_path;
use iloveair::notify::send_pushover_notification;
use iloveair::sensordata::pretty_print_sensor_data;
use iloveair::sensordata::SensorData;
use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::io::Write;

fn read_indoor_json() -> Result<(u64, f32)> {
    let contents = fs::read_to_string(get_indoor_path()).unwrap();
    let indoor: SensorData = serde_json::from_str(&contents).unwrap();
    pretty_print_sensor_data(&indoor);
    let indoor_temp = indoor.temperature.val;
    let humidity = indoor.humidity.val;
    Ok((humidity as u64, indoor_temp))
}

fn main() {
    match real_main() {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            for cause in e.chain() {
                println!("  caused by: {}", cause);
            }
        }
    }
}
fn real_main() -> Result<()> {
    let (indoor_humidity, indoor_temp) = read_indoor_json()?;
    let weather_json_path = get_weather_path();
    let weather_json = load_weather_response(weather_json_path.as_str()).with_context(|| {
        format!(
            "load_weather_response: could not load {}",
            weather_json_path
        )
    })?;
    let outdoor_humidity = weather_humidity(&weather_json).with_context(|| {
        format!(
            "load_weather_response: could parse humidity {}",
            weather_json_path
        )
    })?;
    let outdoor_temp = weather_tempurature(&weather_json).with_context(|| {
        format!(
            "load_weather_response: could parse temperature {}",
            weather_json_path
        )
    })?;

    println!("indoor_humidity: {}", indoor_humidity);
    println!("outdoor_humidity: {}", outdoor_humidity);
    println!("indoor temp: {}", indoor_temp);
    println!("outdoor_temp: {}", outdoor_temp);

    let can_let_in_humidify = outdoor_humidity < indoor_humidity || outdoor_humidity < 60;
    let can_let_in_temperature = outdoor_temp > 50.0 && outdoor_temp < 90.0;
    println!("can_let_in_humidify: {}", can_let_in_humidify);
    println!("can_let_in_temperature: {}", can_let_in_temperature);

    let can_open_window = can_let_in_humidify && can_let_in_temperature;
    let is_open_window: bool = read_is_window_open();
    if can_open_window && !is_open_window {
        println!("send notification");
        send_pushover_notification(&format!(
            "open the windows 🪟 outdoor temp: {} outdoor_humidity: {}",
            outdoor_temp, outdoor_humidity
        ))?;
    } else if !can_open_window && is_open_window {
        println!("send notification");
        send_pushover_notification(&format!(
            "close the windows 🪟 outdoor temp: {} outdoor_humidity: {}",
            outdoor_temp, outdoor_humidity
        ))?;
    } else {
        println!(
            "no notification can open window {} is open window {}",
            can_open_window, is_open_window
        );
    }
    save_is_window_open(can_open_window);
    Ok(())
}
fn is_modified_older_than(path: &str, seconds: u64) -> bool {
    let metadata = fs::metadata(path).unwrap();
    let modified = metadata.modified().unwrap();
    let modified_since_epoch = modified.duration_since(std::time::UNIX_EPOCH).unwrap();
    let modified_seconds = modified_since_epoch.as_secs();
    let now = std::time::SystemTime::now();
    let now_since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let now_seconds = now_since_epoch.as_secs();
    let diff = now_seconds - modified_seconds;
    diff > seconds
}
fn read_is_window_open() -> bool {
    // return false for closed if file does not exist
    if !std::path::Path::new(get_open_windows_path().as_str()).exists() {
        return false;
    }
    // if state file is overe 8 hours old then assume closed and return False
    if is_modified_older_than(get_open_windows_path().as_str(), 8 * 60 * 60) {
        return false;
    }

    let contents = fs::read_to_string(get_open_windows_path()).unwrap();
    contents.parse::<bool>().unwrap()
}
fn save_is_window_open(can_open_window: bool) {
    let mut file = File::create(get_open_windows_path()).unwrap();
    file.write_all(can_open_window.to_string().as_bytes())
        .unwrap();
}
