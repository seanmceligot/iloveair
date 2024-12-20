use crate::audit::read_to_string_with_shellexpand;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AirthingsConfig {
    pub client_id: String,
    pub client_secret: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct WeatherConfig {
    pub api_key: String,
    pub city: String,
    pub country: String,
    pub latitude: String,
    pub longitude: String,
}
pub fn read_airthings_config(filename: &str) -> Result<AirthingsConfig> {
    let contents = read_to_string_with_shellexpand(&filename.to_string())?;

    let config: AirthingsConfig = serde_json::from_str(&contents)
        .map_err(|e| anyhow!(format!("could not parse {} {}", filename, e)))?;

    Ok(config)
}
pub fn read_weather_config(filename: &str) -> Result<WeatherConfig> {
    let contents = read_to_string_with_shellexpand(&filename.to_string())?;

    let config: WeatherConfig = serde_json::from_str(&contents)
        .with_context(|| format!("read_weather_config: could not parse {}", filename))?;

    Ok(config)
}
pub fn file_older_than_minutes(path: &str, minutes: u64) -> bool {
    if !std::path::Path::new(path).exists() {
        return false;
    }
    let metadata = std::fs::metadata(path).unwrap();
    let modified = metadata.modified().unwrap();
    let elapsed_secs = modified.elapsed().unwrap().as_secs();
    elapsed_secs < minutes / 60
}
