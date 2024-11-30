use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::Local;
use clap::{command, Arg};
use crc32fast::Hasher;
use fs::File;
use iloveair::airthings_radon::celsius_to_fahrenheit;
use iloveair::airthings_radon::Indoor;
use iloveair::config::file_older_than_minutes;
use iloveair::notify::read_pushover_json;
use iloveair::notify::send_pushover_notification;
use iloveair::notify::PushoverConfig;
use iloveair::pretty::PrettyBool;
use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::fs;
use std::io::Write;

static DOWN: &str = "↓";
static UP: &str = "↗";
static EQ: &str = "=";

struct IndoorSettings {
    max_humidity: u64,
    min_temp: f64,
    max_temp: f64,
}
#[derive(Clone, Debug)]
struct HumidityTemp {
    humidity: u64,
    temp: f64,
}

fn read_indoor_json(indoor_cache_path: &String) -> Result<HumidityTemp> {
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
    Ok(HumidityTemp {
        humidity: humidity as u64,
        temp: indoor_temp,
    })
}
fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new("pushover_config")
                .short('p')
                .long("pushover")
                .value_name("FILE")
                .default_value("~/.config/iloveair/pushover.json")
                .required(false)
                .help("config ~/.config/iloveair/pushover.json"),
        )
        .arg(
            Arg::new("text_in")
                .short('i')
                .long("text-in")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/Indoor.txt"),
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
    let Some(text_in_path) = matches.get_one::<String>("text_in") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };

    let is_dry_run = matches.get_flag("dry_run");
    match app_main(pushover_config_path, text_in_path, is_dry_run) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn is_changed(text_in_path: &str, text_in: &String) -> Result<bool> {
    // Step 1: Define checksum path by changing extension from .txt to .md5
    let checksum_path = text_in_path.replace(".txt", ".crc32");

    let mut hasher = Hasher::new();
    hasher.update(text_in.as_bytes());
    let computed_checksum = hasher.finalize().to_string();

    // Step 3: Determine if the checksum has changed
    let changed = match fs::read_to_string(&checksum_path) {
        Ok(existing_checksum) => existing_checksum.trim() != computed_checksum,
        Err(_) => true,
    };

    // Step 4: If changed, write the new checksum to the checksum file
    if changed {
        let mut file = File::create(&checksum_path)?;
        writeln!(file, "{}", computed_checksum)?;
    }

    Ok(changed)
}

fn app_main(pushover_config_path: &String, text_in_path: &String, is_dry_run: bool) -> Result<()> {
    let pushover_config = read_pushover_json(pushover_config_path)?;
    let text_in = fs::read_to_string(text_in_path)
        .with_context(|| anyhow!("could not read {}", text_in_path))?;
    let is_changed =
        is_changed(text_in_path, &text_in).with_context(|| anyhow!("error checking checksum"))?;
    if is_changed {
        notify_pushover(&pushover_config, is_dry_run, &text_in)?;
    }
    Ok(())
}
fn updown<T: PartialOrd + ToString>(fst: T, snd: T) -> String {
    if let Some(o) = fst.partial_cmp(&snd) {
        match o {
            std::cmp::Ordering::Less => DOWN.into(),
            std::cmp::Ordering::Greater => UP.into(),
            std::cmp::Ordering::Equal => EQ.into(),
        }
    } else {
        "?".into()
    }
}
fn print_report(
    indoor: HumidityTemp,
    outdoor: HumidityTemp,
    can_let_in_humidify: bool,
    can_let_in_temperature: bool,
    window_should_be_open: bool,
    window_state: Option<bool>,
) {
    //let today = Local::now().date_naive();

    let now = Local::now().naive_local(); // Get current date and time in naive format
    println!("Time: {}", now.format("%A %Y-%m-%d %I:%M:%S %p"));

    //println!("Date Time: {}", today.format("%A %Y-%m-%d %I:%M %p"));
    //println!("Date Time: {}", today.format("%A %Y-%m-%d %H"));

    println!(
        "indoor temp: 🏠{} {}🌡️",
        updown(indoor.temp, outdoor.temp),
        indoor.temp
    );
    println!(
        "outdoor temp: 🌳{} {}🌡️",
        updown(outdoor.temp, indoor.temp),
        outdoor.temp
    );
    println!(
        "Indoor humidity: 🏠{} {}💧",
        updown(indoor.humidity, outdoor.humidity),
        indoor.humidity
    );
    println!(
        "outdoor humidity: 🌳 {} {}💧",
        updown(outdoor.humidity, indoor.humidity),
        outdoor.humidity
    );
    println!(
        "can_let_in_humidify: 💧{}",
        PrettyBool::new(can_let_in_humidify)
    );
    println!(
        "can_let_in_temperature: 🌡️{}",
        PrettyBool::new(can_let_in_temperature)
    );

    println!(
        "window_should_be_open: 🪟{}",
        PrettyBool::new(window_should_be_open)
    );
    let (unknown_window_state, is_open_window) = match window_state {
        Some(is_open_window) => (false, is_open_window),
        None => (true, false),
    };
    if unknown_window_state {
        println!("unknown_window_state {}", unknown_window_state);
    } else {
        println!("is_window_open: 🪟{}", PrettyBool::new(is_open_window));
    }
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
enum NotifyState {
    OpenWindows,
    CloseWindow,
    NoChange,
}
fn determine_notify_state(window_should_be_open: bool, window_state: Option<bool>) -> NotifyState {
    const CAN_CLOSE_WINDOW: bool = false;
    const WINDOW_IS_CLOSED: Option<bool> = Some(false);
    const IS_OPEN_WINDOW: Option<bool> = Some(true);
    const CAN_OPEN_WINDOW: bool = true;
    const UNKNOWN_STATE: Option<bool> = None;
    match (window_should_be_open, window_state) {
        (CAN_OPEN_WINDOW, UNKNOWN_STATE) | (CAN_OPEN_WINDOW, WINDOW_IS_CLOSED) => {
            NotifyState::OpenWindows
        }
        (CAN_CLOSE_WINDOW, UNKNOWN_STATE) | (CAN_CLOSE_WINDOW, IS_OPEN_WINDOW) => {
            NotifyState::CloseWindow
        }
        _ => NotifyState::NoChange,
    }
}
fn notify_pushover(
    pushover_config: &PushoverConfig,
    is_dry_run: bool,
    text_in: &str,
) -> Result<()> {
    println!("send notification");
    send_pushover_notification(is_dry_run, pushover_config, text_in)?;
    Ok(())
}
fn save_is_window_open(is_dry_run: bool, window_open_path: &String, can_open_window: bool) {
    if !is_dry_run {
        let mut file = File::create(window_open_path).unwrap();
        file.write_all(can_open_window.to_string().as_bytes())
            .unwrap();
    }
}
