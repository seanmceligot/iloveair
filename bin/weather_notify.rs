use std::fs;

use anyhow::{Context, Result};
use clap::{command, Arg};
use fs::File;
use iloveair::airthings_radon::celsius_to_fahrenheit;
use iloveair::airthings_radon::Indoor;
use iloveair::notify::read_pushover_json;
use iloveair::notify::send_pushover_notification;
use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::io::Write;

fn read_indoor_json(indoor_cache_path: &String) -> Result<(u64, f32)> {
    let contents = fs::read_to_string(indoor_cache_path).with_context(|| {
        format!(
            "load_weather_response: could not read {}",
            indoor_cache_path
        )
    })?;
    let indoor: Indoor = serde_json::from_str(&contents).with_context(|| {
        format!(
            "load_weather_response: could not parse {}",
            indoor_cache_path
        )
    })?;
    let indoor_temp_celsius = indoor.temp;
    let humidity = indoor.humidity;
    let indoor_temp = celsius_to_fahrenheit(indoor_temp_celsius);
    Ok((humidity as u64, indoor_temp as f32))
}
fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new("pushover_config")
                .short('p')
                .long("pushover")
                .value_name("FILE")
                .required(true)
                .help("config ~/.config/iloveair/pushover.json"),
        )
        .arg(
            Arg::new("weather_cache")
                .short('w')
                .long("weather")
                .value_name("FILE")
                .required(true)
                .help("config ~/.cache/iloveair/weather.json"),
        )
        .arg(
            Arg::new("indoor_cache")
                .short('i')
                .long("indoor")
                .value_name("FILE")
                .required(true)
                .help("config ~/.cache/iloveair/indoor.json"),
        )
        .arg(
            Arg::new("window_state")
                .short('d')
                .long("window")
                .value_name("FILE")
                .required(true)
                .help("config ~/.cache/iloveair/open_windows.state"),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .required(false)
                .num_args(0)
                .help("don't send notification or write window state"),
        );
    let matches = command.get_matches();

    let Some(pushover_config_path) = matches.get_one::<String>("pushover_config") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };

    let Some(weather_cache_path) = matches.get_one::<String>("weather_cache") else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(indoor_cache_path) = matches.get_one::<String>("indoor_cache") else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(window_state_path) = matches.get_one::<String>("window_state") else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };

    let is_dry_run = matches.get_flag("dry_run");
    match app_main(
        pushover_config_path,
        weather_cache_path,
        indoor_cache_path,
        window_state_path,
        is_dry_run,
    ) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}
fn app_main(
    pushover_config_path: &String,
    weather_json_path: &String,
    indoor_cache_path: &String,
    window_state_path: &String,
    is_dry_run: bool,
) -> Result<()> {
    let (indoor_humidity, indoor_temp) = read_indoor_json(indoor_cache_path)?;
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
    println!("can_open_window: {}", can_open_window);
    if is_dry_run {
    } else {
        let is_open_window: bool = read_is_window_open(window_state_path);
        let pushover_config = read_pushover_json(pushover_config_path)?;
        if can_open_window && !is_open_window {
            println!("send notification");
            send_pushover_notification(
                &pushover_config,
                &format!(
                    "open the windows 🪟 outdoor temp: {} outdoor_humidity: {}",
                    outdoor_temp, outdoor_humidity
                ),
            )?;
        } else if !can_open_window && is_open_window {
            println!("send notification");
            send_pushover_notification(
                &pushover_config,
                &format!(
                    "close the windows 🪟 outdoor temp: {} outdoor_humidity: {}",
                    outdoor_temp, outdoor_humidity
                ),
            )?;
        } else {
            println!(
                "no notification can open window {} is open window {}",
                can_open_window, is_open_window
            );
        }
        save_is_window_open(window_state_path, can_open_window);
    }
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
fn read_is_window_open(window_open_path: &String) -> bool {
    // return false for closed if file does not exist
    if !std::path::Path::new(window_open_path).exists() {
        return false;
    }
    // if state file is overe 8 hours old then assume closed and return False
    if is_modified_older_than(window_open_path, 8 * 60 * 60) {
        return false;
    }

    let contents = fs::read_to_string(window_open_path).unwrap();
    contents.parse::<bool>().unwrap()
}
fn save_is_window_open(window_open_path: &String, can_open_window: bool) {
    let mut file = File::create(window_open_path).unwrap();
    file.write_all(can_open_window.to_string().as_bytes())
        .unwrap();
}
