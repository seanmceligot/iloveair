extern crate reqwest;
use std::path::Path;

use anyhow::{Context, Result};

use crate::audit::read_to_string_with_path;

pub fn weather_tempurature(weather_json: &serde_json::Value) -> Result<f64> {
    let temperature = weather_json["main"]["temp"].as_f64().unwrap();
    Ok(temperature)
}
pub fn weather_humidity(weather_json: &serde_json::Value) -> Result<u64> {
    let humidity = weather_json["main"]["humidity"].as_u64().unwrap();
    Ok(humidity)
}

pub fn load_weather_response<P: AsRef<Path>>(weather_json_path: P) -> Result<serde_json::Value> {
    let weather_json = read_to_string_with_path(weather_json_path.as_ref())?;
    let weather_json: serde_json::Value =
        serde_json::from_str(&weather_json).with_context(|| {
            format!(
                "load_weather_response: could not load {:?}",
                weather_json_path.as_ref()
            )
        })?;
    Ok(weather_json)
}
