extern crate reqwest;

use anyhow::{Context, Result};
pub fn weather_tempurature(weather_json: &serde_json::Value) -> Result<f64> {
    let temperature = weather_json["main"]["temp"].as_f64().unwrap();
    Ok(temperature)
}
pub fn weather_humidity(weather_json: &serde_json::Value) -> Result<u64> {
    let humidity = weather_json["main"]["humidity"].as_u64().unwrap();
    Ok(humidity)
}

pub fn load_weather_response(weather_json_path: &str) -> Result<serde_json::Value> {
    let weather_json = std::fs::read_to_string(weather_json_path).with_context(|| {
        format!(
            "load_weather_response: could not load {}",
            weather_json_path
        )
    })?;
    let weather_json: serde_json::Value =
        serde_json::from_str(&weather_json).with_context(|| {
            format!(
                "load_weather_response: could not load {}",
                weather_json_path
            )
        })?;
    Ok(weather_json)
}
