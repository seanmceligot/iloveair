use anyhow::anyhow;
use anyhow::{Context, Result};
use chrono::Local;
use clap::{command, value_parser, Arg};
use iloveair::airthings_radon::celsius_to_fahrenheit;
use iloveair::airthings_radon::Indoor;
use iloveair::audit::read_to_string_with_shellexpand;
use iloveair::pretty::PrettyBool;
use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::fs::OpenOptions;
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
    let contents = read_to_string_with_shellexpand(indoor_cache_path).with_context(|| {
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
            Arg::new("weather_cache")
                .short('w')
                .long("weather")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/weather.json"),
        )
        .arg(
            Arg::new("indoor_cache")
                .short('i')
                .long("indoor")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/indoor.json"),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .required(false)
                .num_args(0)
                .help("don't save output"),
        )
        .arg(
            Arg::new("max_humidity")
                .value_parser(value_parser!(u64))
                .long("max-humidity")
                .value_name("VALUE")
                .default_value("60")
                .help("Maximum allowable humidity"),
        )
        .arg(
            Arg::new("min_temp")
                .value_parser(value_parser!(f64))
                .long("min-temp")
                .value_name("VALUE")
                .default_value("50.0")
                .help("Minimum allowable temperature"),
        )
        .arg(
            Arg::new("text_out_path")
                .short('o')
                .long("text-out")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/Indoor.txt"),
        )
        .arg(
            Arg::new("max_temp")
                .value_parser(value_parser!(f64))
                .long("max-temp")
                .value_name("VALUE")
                .default_value("84.0")
                .help("Maximum allowable temperature"),
        );
    let matches = command.get_matches();

    let Some(weather_cache_path) = matches.get_one::<String>("weather_cache") else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(indoor_cache_path) = matches.get_one::<String>("indoor_cache") else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(max_humidity) = matches.get_one::<u64>("max_humidity") else {
        // this else block is unreachable because default value would be retuned
        unreachable!();
    };
    let Some(min_temp) = matches.get_one::<f64>("min_temp") else {
        // This else block is unreachable because of the default value.
        unreachable!();
    };

    let Some(max_temp) = matches.get_one::<f64>("max_temp") else {
        // This else block is unreachable because of the default value.
        unreachable!();
    };
    let Some(text_out_path) = matches.get_one::<String>("text_out_path") else {
        // This else block is unreachable because of the default value.
        unreachable!();
    };
    let indoor_settings = IndoorSettings {
        max_humidity: *max_humidity,
        min_temp: *min_temp,
        max_temp: *max_temp,
    };
    let is_dry_run = matches.get_flag("dry_run");
    match app_main(
        weather_cache_path,
        indoor_cache_path,
        is_dry_run,
        indoor_settings,
        text_out_path,
    ) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}
fn app_main(
    weather_json_path: &String,
    indoor_cache_path: &String,
    is_dry_run: bool,
    indoor_settings: IndoorSettings,
    text_out_path: &String,
) -> Result<()> {
    let indoor = read_indoor_json(indoor_cache_path)?;
    let weather_json = load_weather_response(weather_json_path).with_context(|| {
        anyhow!(
            "load_weather_response: could not load {}",
            weather_json_path
        )
    })?;
    let outdoor_humidity = weather_humidity(&weather_json).with_context(|| {
        anyhow!(
            "load_weather_response: could parse humidity {}",
            weather_json_path
        )
    })?;
    let outdoor_temp = weather_tempurature(&weather_json).with_context(|| {
        anyhow!(
            "load_weather_response: could parse temperature {}",
            weather_json_path
        )
    })?;
    let outdoor = HumidityTemp {
        humidity: outdoor_humidity,
        temp: outdoor_temp,
    };
    let can_let_in_humidify =
        outdoor.humidity <= indoor.humidity || outdoor.humidity <= indoor_settings.max_humidity;
    let can_let_in_temperature =
        outdoor.temp >= indoor_settings.min_temp && outdoor.temp <= indoor_settings.max_temp;
    let window_should_be_open = can_let_in_humidify && can_let_in_temperature;
    print_report(
        indoor.clone(),
        outdoor.clone(),
        can_let_in_humidify,
        can_let_in_temperature,
        window_should_be_open,
        text_out_path,
        is_dry_run,
    );
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
    text_out_path: &String,
    is_dry_run: bool,
) {
    let now = Local::now().naive_local(); // Get current date and time in naive format
    let mut report = String::new();

    report.push_str(&format!("Time: {}\n", now.format("%A %Y-%m-%d %I:%M %p")));
    report.push_str(&format!(
        "window_should_be_open: 🪟{}\n",
        PrettyBool::new(window_should_be_open)
    ));
    report.push_str(&format!(
        "indoor temp: 🏠{} {}🌡️\n",
        updown(indoor.temp, outdoor.temp),
        indoor.temp
    ));
    report.push_str(&format!(
        "outdoor temp: 🌳{} {}🌡️\n",
        updown(outdoor.temp, indoor.temp),
        outdoor.temp
    ));
    report.push_str(&format!(
        "Indoor humidity: 🏠{} {}💧\n",
        updown(indoor.humidity, outdoor.humidity),
        indoor.humidity
    ));
    report.push_str(&format!(
        "outdoor humidity: 🌳 {} {}💧\n",
        updown(outdoor.humidity, indoor.humidity),
        outdoor.humidity
    ));
    report.push_str(&format!(
        "can_let_in_humidify: 💧{}\n",
        PrettyBool::new(can_let_in_humidify)
    ));
    report.push_str(&format!(
        "can_let_in_temperature: 🌡️{}\n",
        PrettyBool::new(can_let_in_temperature)
    ));

    // Print to stdout
    println!("{}", report);

    // Write to file if not a dry run
    if !is_dry_run {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(text_out_path)
        {
            if let Err(e) = writeln!(file, "{}", report) {
                eprintln!("Failed to write to file: {}", e);
            }
        } else {
            eprintln!("Failed to open file: {}", text_out_path);
        }
    }
}
