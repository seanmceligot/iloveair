extern crate reqwest;

use anyhow::{anyhow, Context, Result};
use clap::{command, Arg};
use iloveair::config::read_weather_config;
use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};

fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new("out")
                .short('o')
                .long("out")
                .value_name("FILE")
                .help(
                    "output file, defaults to stdout if not present ~/.cache/iloveair/weather.json",
                ),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .required(true)
                .help("config $HOME/.config/iloveair/openweathermap.json"),
        );
    let matches = command.get_matches();

    let outfile = matches.get_one::<String>("out");
    if let Some(config_file) = matches.get_one::<String>("config") {
        match the_main(outfile, config_file) {
            Ok(_) => (),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Error: --config -c required");
    }
}
fn save_weather_response(
    maybe_weather_json_path: Option<&String>,
    response: &serde_json::Value,
) -> Result<()> {
    if let Some(weather_json_path) = maybe_weather_json_path {
        let mut fout = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(weather_json_path)
            .with_context(|| {
                format!(
                    "save_weather_response: could not open for write {}",
                    weather_json_path
                )
            })?;
        fout.write_all(serde_json::to_string_pretty(response)?.as_bytes())
            .with_context(|| format!("save_weather_response: could write {}", weather_json_path))?;
        println!("wrote: {}", weather_json_path);
    } else {
        io::stdout()
            .write_all(serde_json::to_string_pretty(response)?.as_bytes())
            .with_context(|| "save_weather_response: could write weather_json to stdout")?;
    }
    Ok(())
}
fn file_modified_in_last_minutes(path: &str, minutes: u64) -> bool {
    if !std::path::Path::new(path).exists() {
        return false;
    }
    let metadata = std::fs::metadata(path).unwrap();
    let modified = metadata.modified().unwrap();
    let modified = modified.elapsed().unwrap().as_secs();
    modified < minutes * 60
}
fn the_main(maybe_weather_json_path: Option<&String>, config_file: &String) -> Result<()> {
    let config = read_weather_config(config_file)
        .with_context(|| format!("could not read config {}", config_file))?;

    let update_no_more_than_minutes = 10;
    if let Some(weather_json_path) = maybe_weather_json_path {
        if file_modified_in_last_minutes(weather_json_path, update_no_more_than_minutes) {
            println!(
                "weather.json is less than {} minutes old",
                update_no_more_than_minutes
            );
            return Ok(());
        }
    }
    println!("API Key: {}", config.api_key);
    println!("City: {}", config.city);
    println!("Country: {}", config.country);
    let api_key = config.api_key;
    let city_name = config.city;
    let country_code = config.country;

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={},{}&appid={}&units=imperial",
        city_name, country_code, api_key
    );

    let response = reqwest::blocking::get(&url)?.json::<serde_json::Value>()?;
    save_weather_response(maybe_weather_json_path, &response)?;

    let temperature = response["main"]["temp"].as_f64().unwrap();
    let humidity = response["main"]["humidity"].as_u64().unwrap();

    println!(
        "The temperature in {} is {:.2}°F and the humidity is {}%",
        city_name, temperature, humidity
    );

    //let weather_json = load_weather_response(weather_json_path)?;
    //let temperature = weather_tempurature(&weather_json)?;
    //let humidity = weather_humidity(&weather_json)?;

    println!(
        "The temperature is {:.2}°F and the humidity is {}%",
        temperature, humidity
    );
    Ok(())
}
