use std::fs;

use anyhow::{Context, Result};
use clap::{command, Arg};
use fs::File;
use iloveair::airthings_radon::celsius_to_fahrenheit;
use iloveair::airthings_radon::Indoor;
use iloveair::config::file_older_than_minutes;
use iloveair::notify::read_pushover_json;
use iloveair::notify::send_pushover_notification;
use iloveair::notify::PushoverConfig;
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

    let window_should_be_open = can_let_in_humidify && can_let_in_temperature;
    println!("window_should_be_open: {}", window_should_be_open);
    let window_state = load_saved_window_state(window_state_path, 8 * 60);
    let pushover_config = read_pushover_json(pushover_config_path)?;
    notify_if_needed(
        window_should_be_open,
        window_state,
        &pushover_config,
        is_dry_run,
        indoor_temp,
        indoor_humidity,
        outdoor_temp,
        outdoor_humidity,
        window_state_path,
    )
}
fn load_saved_window_state(window_open_path: &String, stale_minutes: u64) -> Option<bool> {
    // if state file does not exist or is invalid the None
    // None means send notify
    // true or false means send only if state changed
    if !std::path::Path::new(window_open_path).exists() {
        // not yet set
        println!("no window.state file");
        None
    } else if file_older_than_minutes(window_open_path, stale_minutes) {
        println!("window.state is stale");
        // stale
        None
    } else {
        let maybe_contents = fs::read_to_string(window_open_path);
        match maybe_contents {
            Ok(contents) => match contents.parse::<bool>() {
                Ok(b) => Some(b),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
}
fn notify_if_needed(
    window_should_be_open: bool,
    window_state: Option<bool>,
    pushover_config: &PushoverConfig,
    is_dry_run: bool,
    indoor_temp: f32,
    indoor_humidity: u64,
    outdoor_temp: f64,
    outdoor_humidity: u64,
    window_state_path: &String,
) -> Result<()> {
    let (unknown_window_state, is_open_window) = match window_state {
        Some(is_open_window) => (false, is_open_window),
        None => (true, false),
    };
    if unknown_window_state {
        println!("unknown_window_state {}", unknown_window_state);
    } else {
        println!("is_open_window {}", is_open_window);
    }
    const CAN_CLOSE_WINDOW: bool = false;
    const WINDOW_IS_CLOSED: Option<bool> = Some(false);
    const IS_OPEN_WINDOW: Option<bool> = Some(true);
    const CAN_OPEN_WINDOW: bool = true;
    const UNKNOWN_STATE: Option<bool> = None;

    match (window_should_be_open, window_state) {
        (CAN_OPEN_WINDOW, UNKNOWN_STATE) | (CAN_OPEN_WINDOW, WINDOW_IS_CLOSED) => {
            println!("send notification");
            send_pushover_notification(
                is_dry_run,
                &pushover_config,
                &format!(
                    "open the windows 🪟  \
                    outdoor temp: {}  \
                    indoor temp: {}  \
                    outdoor_humidity: {} \
                    indoor_humidity: {}",
                    outdoor_temp, indoor_temp, outdoor_humidity, indoor_humidity
                ),
            )?;
            save_is_window_open(is_dry_run, window_state_path, window_should_be_open);
        }
        (CAN_CLOSE_WINDOW, UNKNOWN_STATE) | (CAN_CLOSE_WINDOW, IS_OPEN_WINDOW) => {
            println!("send notification");
            send_pushover_notification(
                is_dry_run,
                &pushover_config,
                &format!(
                    "close the windows 🪟 
                    outdoor temp: {} \
                    indoor temp: {} \
                    outdoor_humidity: {}
                    indoor_humidity: {}
                ",
                    outdoor_temp, indoor_temp, outdoor_humidity, indoor_humidity
                ),
            )?;
            save_is_window_open(is_dry_run, window_state_path, window_should_be_open);
        }
        _ => {
            assert!(unknown_window_state == false);
            println!(
                "no change can open window {} is open window {}",
                window_should_be_open, is_open_window
            );
        }
    }
    Ok(())
}
fn save_is_window_open(is_dry_run: bool, window_open_path: &String, can_open_window: bool) {
    if !is_dry_run {
        let mut file = File::create(window_open_path).unwrap();
        file.write_all(can_open_window.to_string().as_bytes())
            .unwrap();
    }
}
