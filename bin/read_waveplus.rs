extern crate reqwest;
extern crate tokio;
use anyhow::{anyhow, Context, Error, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{command, Arg};
use iloveair::audit::read_to_string_with_path;
use iloveair::config::read_airthings_config;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize)]
struct AccessToken {
    access_token: String,
    expiration: DateTime<Utc>,
}

impl From<TokenResponse> for AccessToken {
    fn from(token: TokenResponse) -> Self {
        let expiration = Utc::now() + Duration::seconds(token.expires_in as i64);
        AccessToken {
            access_token: token.access_token,
            expiration,
        }
    }
}

impl AccessToken {
    fn has_expired(&self) -> bool {
        Utc::now() > self.expiration
    }
}

fn read_json_token<P: AsRef<Path>>(path: P) -> Option<AccessToken> {
    let file_content = read_to_string_with_path(path.as_ref()).ok()?;
    let access_token: AccessToken = serde_json::from_str(&file_content).ok()?;
    if access_token.has_expired() {
        None
    } else {
        Some(access_token)
    }
}
fn write_access_token<P: AsRef<Path>>(path: P, access_token: &AccessToken) -> Result<(), Error> {
    let json_data = serde_json::to_string_pretty(access_token)
        .map_err(|e| anyhow!(format!("could not write token {}", e)))?;

    fs::write(path, json_data)?;
    Ok(())
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    grant_type: &'static str,
    client_id: &'a str,
    client_secret: &'a str,
    //scope: Vec<&'static str>,
}

async fn fetch_token<'a>(client_id: &'a str, client_secret: &'a str) -> Result<AccessToken, Error> {
    let client = reqwest::Client::new();
    //let token_request =
    const TOKEN_URL: &str = "https://accounts-api.airthings.com/v1/token";

    let response: Response = client
        .post(TOKEN_URL)
        .json(&TokenRequest {
            grant_type: "client_credentials",
            client_id,
            client_secret,
        })
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send request: {} {}", TOKEN_URL, e))?;
    //scope: vec!["read:device"],

    println!("response {:?}", response);
    let text = response
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get text: {} {}", TOKEN_URL, e))?;

    println!("text {:?}", text);
    let json_data = serde_json::from_str::<Value>(&text)
        .map_err(|e| anyhow::anyhow!("Failed to parse response to JSON: {} {} ", text, e))?;

    check_json_errors(&json_data)?;
    let token_response: TokenResponse = serde_json::from_value(json_data).map_err(|e| {
        anyhow::anyhow!(
            "could not parse json into TokenResponse: {} {}",
            TOKEN_URL,
            e
        )
    })?;
    //let _ = token_response.into())
    let r: AccessToken = token_response.into();
    Ok(r)
    //Ok(token_response.into())
    //Ok(ExitStatus<::into(token_response))
}

// https://airthings.org/api/air-indoor

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new("airthing_config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .required(true)
                .help("config ~/.config/iloveair/airthings.json"),
        )
        .arg(
            Arg::new("indoor_cache")
                .short('i')
                .long("indoor")
                .value_name("FILE")
                .required(true)
                .help("config ~/.cache/iloveair/waveplus.json"),
        )
        .arg(
            Arg::new("airthings_token_cache")
                .short('t')
                .long("token")
                .value_name("FILE")
                .required(true)
                .help("config ~/.cache/iloveair/airthings_token.json"),
        )
        .arg(
            Arg::new("list_devices")
                .long("list-devices")
                .required(false)
                .num_args(0)
                .help("list devices instead of downloading data"),
        );
    let matches = command.get_matches();

    let Some(airthings_config_path) = matches.get_one::<String>("airthing_config") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };

    let Some(indoor_cache_path) = matches.get_one::<String>("indoor_cache") else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(airthings_token_cache_path) = matches.get_one::<String>("airthings_token_cache")
    else {
        // this else block is unreachable because the argument is required.
        unreachable!();
    };
    let do_list_devices = matches.get_flag("list_devices");

    match app_main(
        airthings_config_path,
        indoor_cache_path,
        airthings_token_cache_path,
        do_list_devices,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn save_sample_data(indoor_json_path: &str, sample: &SampleData) -> Result<()> {
    let fout = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(indoor_json_path)
        .with_context(|| {
            format!(
                "save_indoor_response: could not open for write {}",
                indoor_json_path
            )
        })?;
    serde_json::to_writer_pretty(fout, sample)
        .map_err(|e| anyhow!(format!("could not convert Sample to json {}", e)))?;

    println!("wrote: {}", indoor_json_path);
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
fn check_json_errors(json: &Value) -> Result<(), Error> {
    if let Some(error) = json.get("error").and_then(Value::as_str) {
        let error_desc = json
            .get("error_description")
            .and_then(Value::as_str)
            .unwrap_or("No description provided");

        let error_code = json
            .get("error_code")
            .and_then(Value::as_str)
            .unwrap_or("No code provided");

        return Err(anyhow!(
            "Error: {}. Description: {}. Code: {}.",
            error,
            error_desc,
            error_code
        ));
    }

    Ok(())
}

async fn app_main(
    airthings_config_json_path: &String,
    indoor_json_cache_path: &String,
    airthings_token_cache_path: &String,
    do_list_devices: bool,
) -> Result<()> {
    let config = read_airthings_config(airthings_config_json_path).map_err(|e| {
        anyhow!(format!(
            "could not read config {} {}",
            airthings_config_json_path, e
        ))
    })?;
    let update_no_more_than_minutes = 10;
    if file_modified_in_last_minutes(indoor_json_cache_path, update_no_more_than_minutes) {
        println!(
            "SKIPPING: {} less than {} minutes old",
            indoor_json_cache_path, update_no_more_than_minutes
        );
        return Ok(());
    }
    let access_token = if let Some(access_token) = read_json_token(airthings_token_cache_path) {
        access_token
    } else {
        println!("client_id: {}", config.client_id);
        println!("client_secret: {}", config.client_secret);
        let access_token = fetch_token(config.client_id.as_str(), config.client_secret.as_str())
            .await
            .map_err(|e| anyhow!(format!("fetch_token {}", e)))?;
        write_access_token(airthings_token_cache_path, &access_token)
            .map_err(|e| anyhow!(format!("write failed {} {}", airthings_token_cache_path, e)))?;
        access_token
    };

    if do_list_devices {
        list_devices(&access_token)
            .await
            .map_err(|e| anyhow!(format!("list_devices {}", e)))?;
        return Ok(());
    }

    let device_id = config.device_id;
    let sample = get_latest_reading(&device_id, &access_token)
        .await
        .map_err(|e| anyhow!(format!("get_latest_reading {} {}", &device_id, e)))?;
    println!("sample: {:?}", sample);
    save_sample_data(indoor_json_cache_path, &sample)
}

async fn list_devices(token: &AccessToken) -> Result<()> {
    let client = reqwest::Client::new();
    let url = "https://ext-api.airthings.com/v1/devices";
    let text = client
        .get(url) // assuming this is the correct endpoint
        .bearer_auth(&token.access_token) // set the authorization header
        .send()
        .await
        .map_err(|e| anyhow!(format!("list devices {}", e)))?
        .text()
        .await
        .map_err(|e| anyhow!(format!("list devices {}", e)))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("list_devices: could not parse devices {}", text))?;

    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SampleData {
    battery: u8,
    humidity: f64,
    radon_short_term_avg: f64,
    temp: f64,
    time: u64,
    relay_device_type: String,
}
#[derive(Deserialize)]
struct SampleDataKey {
    data: SampleData,
}
async fn get_latest_reading(device_id: &String, token: &AccessToken) -> Result<SampleData> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://ext-api.airthings.com/v1/devices/{}/latest-samples",
        device_id
    );
    let response = client
        .get(&url) // assuming this is the correct endpoint
        .bearer_auth(&token.access_token) // set the authorization header
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send request to get latest reading: {}", e))?;

    println!("latest: {:?}", response);
    let text = response
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read request to get latest reading: {}", e))?;
    println!("reading: {}", text);

    let json_data = serde_json::from_str::<Value>(&text)
        .map_err(|e| anyhow::anyhow!("Failed to parse response to JSON: {} {} ", text, e))?;

    check_json_errors(&json_data)?;
    let sample: SampleDataKey = serde_json::from_value(json_data)
        .map_err(|e| anyhow::anyhow!("could not parse json into Reading : {} {}", url, e))?;
    Ok(sample.data)
}
