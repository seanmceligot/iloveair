use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize)]
pub struct WeatherConfig {
    pub api_key: String,
    pub city: String,
    pub country: String,
}
const PUSHOVER_FILE: &str = "/home/sean/.config/iloveair/pushover.json";
const INDOOR_FILE: &str = "/home/sean/.cache/iloveair/waveplus.json";
const WEATHER_JSON_PATH: &str = "/home/sean/.cache/iloveair/weather.json";
const OPEN_WINDOWS_FILE: &str = "/home/sean/.cache/iloveair/open_windows.state";

pub fn get_pushover_path() -> String {
    PUSHOVER_FILE.to_string()
}
pub fn get_indoor_path() -> String {
    INDOOR_FILE.to_string()
}
pub fn get_weather_path() -> String {
    WEATHER_JSON_PATH.to_string()
}
pub fn get_open_windows_path() -> String {
    OPEN_WINDOWS_FILE.to_string()
}
pub fn read_weather_config(filename: &str) -> Result<WeatherConfig> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: WeatherConfig = serde_json::from_str(&contents)
        .with_context(|| format!("read_weather_config: could not parse {}", filename))?;

    Ok(config)
}
