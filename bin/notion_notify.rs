use anyhow::{anyhow, Context, Result};
use clap::{command, Arg};
use crc32fast::Hasher;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Deserialize, Serialize)]
struct Config {
    notion_api_key: String,
    notion_page_id: String,
}

fn main() {
    let command = command!()
        .version("0.2")
        .arg(
            Arg::new("notion_config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .required(true)
                .help("Path to Notion config file."),
        )
        .arg(
            Arg::new("text_in")
                .short('i')
                .long("text-in")
                .value_name("FILE")
                .required(true)
                .help("Path to the input text file."),
        )
        .arg(
            Arg::new("name")
                .short('m')
                .long("name")
                .value_name("NAME")
                .required(true)
                .help("Name for caching output."),
        );
    let matches = command.get_matches();

    let config_path = matches.get_one::<String>("notion_config").unwrap();
    let text_in_path = matches.get_one::<String>("text_in").unwrap();
    let name = matches.get_one::<String>("name").unwrap();

    match app_main(config_path, text_in_path, name) {
        Ok(_) => println!("Process completed successfully."),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn app_main(config_path: &str, text_in_path: &str, name: &str) -> Result<()> {
    let config = load_config(config_path)?;
    let lines = read_lines_from_file(text_in_path)?;
    if is_changed(text_in_path, &lines)? {
        replace_notion_page_content(&config, &lines)?;
        cache_output(name, &lines)?;
    } else {
        println!("No changes detected. Notion update skipped.");
    }
    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let mut file = File::open(shellexpand::tilde(path).as_ref())
        .with_context(|| anyhow!("Failed to open config file at {}", path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)
        .with_context(|| anyhow!("Failed to parse config file at {}", path))?;
    Ok(config)
}

fn read_lines_from_file(path: &str) -> Result<Vec<String>> {
    let mut file = File::open(shellexpand::tilde(path).as_ref())
        .with_context(|| anyhow!("Failed to open text input file at {}", path))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.lines().map(String::from).collect())
}

fn is_changed(path: &str, lines: &[String]) -> Result<bool> {
    let checksum_path = format!("{}.crc32", path);
    let mut hasher = Hasher::new();
    hasher.update(lines.join("\n").as_bytes());
    let computed_checksum = hasher.finalize().to_string();

    let changed = match fs::read_to_string(&checksum_path) {
        Ok(existing_checksum) => existing_checksum.trim() != computed_checksum,
        Err(_) => true,
    };

    if changed {
        let mut file = File::create(&checksum_path)?;
        writeln!(file, "{}", computed_checksum)?;
        println!("Updated checksum at: {}", checksum_path);
    }

    Ok(changed)
}

fn cache_output(name: &str, lines: &[String]) -> Result<()> {
    let cache_file = format!("{}/{}.txt", cache_dir(), name);
    fs::create_dir_all(cache_dir())?;
    let mut file = File::create(&cache_file)?;
    file.write_all(lines.join("\n").as_bytes())?;
    Ok(())
}

fn replace_notion_page_content(config: &Config, lines: &[String]) -> Result<()> {
    let client = Client::new();
    let url = format!(
        "https://api.notion.com/v1/blocks/{}/children",
        config.notion_page_id
    );
    let headers = [
        ("Authorization", format!("Bearer {}", config.notion_api_key)),
        ("Content-Type", "application/json".to_string()),
        ("Notion-Version", "2022-06-28".to_string()),
    ];

    delete_existing_children(&client, &url, &headers)?;
    add_new_children(&client, &url, &headers, lines)?;

    Ok(())
}

fn delete_existing_children(client: &Client, url: &str, headers: &[(&str, String)]) -> Result<()> {
    let response = client.get(url).headers(construct_headers(headers)).send()?;
    if response.status().is_success() {
        let body: serde_json::Value = response.json()?;
        if let Some(results) = body["results"].as_array() {
            for child in results {
                if let Some(id) = child["id"].as_str() {
                    let delete_url = format!("https://api.notion.com/v1/blocks/{}", id);
                    client
                        .delete(&delete_url)
                        .headers(construct_headers(headers))
                        .send()?;
                }
            }
        }
    } else {
        Err(anyhow!(
            "Failed to fetch children: {}",
            response.text().unwrap_or_default()
        ))?;
    }
    Ok(())
}

fn add_new_children(
    client: &Client,
    url: &str,
    headers: &[(&str, String)],
    lines: &[String],
) -> Result<()> {
    let children: Vec<_> = lines
        .iter()
        .map(|line| {
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": { "content": line }
                    }]
                }
            })
        })
        .collect();
    let response = client
        .patch(url)
        .headers(construct_headers(headers))
        .json(&json!({ "children": children }))
        .send()?;
    if response.status().is_success() {
        println!("Notion page updated successfully.");
    } else {
        Err(anyhow!(
            "Failed to update Notion page: {}",
            response.text().unwrap_or_default()
        ))?;
    }
    Ok(())
}

fn construct_headers(headers: &[(&str, String)]) -> reqwest::header::HeaderMap {
    let mut header_map = reqwest::header::HeaderMap::new();
    for (key, value) in headers {
        header_map.insert(
            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(value).unwrap(),
        );
    }
    header_map
}

fn cache_dir() -> String {
    shellexpand::tilde("~/.cache/iloveair").to_string()
}
