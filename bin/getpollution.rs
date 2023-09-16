extern crate reqwest;

use anyhow::{Context, Result};
use clap::command;
use clap::Arg;
use iloveair::config::read_weather_config;
//use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::fs::OpenOptions;
use std::io::{stdout, Write};

// https://openweathermap.org/api/air-pollution

fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new(r#"out"#)
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
        match app_main(config_file, outfile) {
            Ok(_) => (),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Error: --config -c required");
    }
}
fn save_pollution_response(
    maybe_pollution_json_path: Option<&String>,
    response: &serde_json::Value,
) -> Result<()> {
    if let Some(pollution_json_path) = maybe_pollution_json_path {
        let mut fout = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(pollution_json_path)
            .with_context(|| {
                format!(
                    "save_pollution_response: could not open for write {}",
                    pollution_json_path
                )
            })?;
        fout.write_all(serde_json::to_string_pretty(response)?.as_bytes())
            .with_context(|| {
                format!(
                    "save_pollution_response: could write {}",
                    pollution_json_path
                )
            })?;
        println!("wrote: {}", pollution_json_path);
    } else {
        stdout()
            .write_all(serde_json::to_string_pretty(response)?.as_bytes())
            .with_context(|| "save_pollution_response: could write pollution_json to stdout")?;
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
fn aqi_description(aqi: u64) -> &'static str {
    match aqi {
        1 => "Good",
        2 => "Fair",
        3 => "Moderate",
        4 => "Poor",
        5 => "Very Poor",
        _ => "Unknown",
    }
}
fn app_main(config_file: &String, maybe_pollution_json_path: Option<&String>) -> Result<()> {
    let config = read_weather_config(config_file)
        .with_context(|| format!("could not read config {}", config_file))?;

    let update_no_more_than_minutes = 10;
    if let Some(pollution_json_path) = maybe_pollution_json_path {
        if file_modified_in_last_minutes(pollution_json_path, update_no_more_than_minutes) {
            println!(
                "pollution.json is less than {} minutes old",
                update_no_more_than_minutes
            );
            return Ok(());
        }
    }

    //println!("API Key: {}", config.api_key);
    println!("Latitude: {}", config.latitude);
    println!("longitude: {}", config.longitude);
    let api_key = config.api_key;
    // let city_name = config.city;
    // let country_code = config.country;

    let url = format!(
        "http://api.openweathermap.org/data/2.5/air_pollution?lat={}&lon={}&appid={}",
        config.latitude, config.longitude, api_key
    );
    // example response
    // {
    //   "coord":[
    //     50,
    //     50
    //   ],
    //   "list":[
    //     {
    //       "dt":1605182400,
    //       "main":{
    //         "aqi":1
    //       },
    //       "components":{
    //         "co":201.94053649902344,
    //         "no":0.01877197064459324,
    //         "no2":0.7711350917816162,
    //         "o3":68.66455078125,
    //         "so2":0.6407499313354492,
    //         "pm2_5":0.5,
    //         "pm10":0.540438711643219,
    //         "nh3":0.12369127571582794
    //       }
    //     }
    //   ]
    // }
    let response = reqwest::blocking::get(&url)?.json::<serde_json::Value>()?;

    save_pollution_response(maybe_pollution_json_path, &response)?;
    let aqi = response["list"][0]["main"]["aqi"].as_u64().unwrap();

    println!("The Air Quality Index is {} {}", aqi, aqi_description(aqi));
    Ok(())
}
