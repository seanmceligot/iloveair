use crate::config::get_pushover_path;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs;

#[derive(Debug, Deserialize)]
struct PushoverConfig {
    api_key: String,
    user_key: String,
}
fn read_pushover_json() -> PushoverConfig {
    let contents = fs::read_to_string(get_pushover_path()).unwrap();
    serde_json::from_str(&contents).unwrap()
}

pub fn send_pushover_notification(msg: &str) -> Result<()> {
    let config = read_pushover_json();
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
