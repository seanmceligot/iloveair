use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct PushoverConfig {
    api_key: String,
    user_key: String,
}
pub fn read_pushover_json(pushover_config_path: &String) -> Result<PushoverConfig> {
    let contents = fs::read_to_string(pushover_config_path).with_context(|| {
        format!(
            "send_pushover_notification: could not read config {}",
            pushover_config_path
        )
    })?;
    serde_json::from_str(&contents).with_context(|| {
        format!(
            "send_pushover_notification: could not parse config {}",
            pushover_config_path
        )
    })
}

pub fn send_pushover_notification(config: &PushoverConfig, msg: &str) -> Result<()> {
    let params = json!({
        "token": config.api_key,
        "user": config.user_key,
        "message": msg,
    });

    let pushover_url = "https://api.pushover.net/1/messages.json";
    let client = Client::new();
    let res = client
        .post(pushover_url)
        .json(&params)
        .send()
        .with_context(|| {
            format!(
                "send_pushover_notification: could not send notification to {}",
                pushover_url
            )
        })?;
    // print response and error code
    println!("pushover status: {}", res.status());
    println!("pushover response: {}", res.text().unwrap());
    Ok(())
}
