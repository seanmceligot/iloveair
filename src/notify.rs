use anyhow::anyhow;
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
pub fn read_to_string_with_expand(path: &String) -> Result<String, anyhow::Error> {
    let expand = shellexpand::full(path)?;
    let expanded = expand.as_ref();

    fs::read_to_string(expanded).map_err(|e| anyhow!("error reading {} {}", expand, e))
}
pub fn read_pushover_json(pushover_config_path: &String) -> Result<PushoverConfig> {
    let contents = read_to_string_with_expand(pushover_config_path)
        .map_err(|e| anyhow!("error reading {} {}", pushover_config_path, e))?;

    serde_json::from_str(&contents).with_context(|| {
        format!(
            "send_pushover_notification: could not parse config {}",
            pushover_config_path
        )
    })
}

pub fn send_pushover_notification(dry_run: bool, config: &PushoverConfig, msg: &str) -> Result<()> {
    if dry_run {
        println!("dry-run: {}", msg);
        return Ok(());
    }
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
