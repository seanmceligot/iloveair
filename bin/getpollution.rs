extern crate reqwest;

use anyhow::{Context, Result};
use iloveair::config::read_weather_config;
//use iloveair::weather::{load_weather_response, weather_humidity, weather_tempurature};
use std::fs::OpenOptions;
use std::io::Write;

// https://openweathermap.org/api/air-pollution

fn main() {
    match the_main() {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}
fn save_pollution_response(pollution_json_path: &str, response: &serde_json::Value) -> Result<()> {
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
fn the_main() -> Result<()> {
    let config = read_weather_config("/home/sean/.config/iloveair/openweathermap.json")
        .with_context(|| "the_main: could not read config")?;
    let pollution_json_path = "/home/sean/.cache/iloveair/pollution.json";

    let update_no_more_than_minutes = 10;
    if file_modified_in_last_minutes(pollution_json_path, update_no_more_than_minutes) {
        println!(
            "weather.json is less than {} minutes old",
            update_no_more_than_minutes
        );
    } else {
        println!("API Key: {}", config.api_key);
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

        save_pollution_response(pollution_json_path, &response)?;
        let aqi = response["list"][0]["main"]["aqi"].as_u64().unwrap();

        println!("The Air Quality Index is {} {}", aqi, aqi_description(aqi));
    }
    Ok(())
}
