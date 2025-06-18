use anyhow::anyhow;
use anyhow::{Context, Result};
use clap::{command, Arg};
use crc32fast::Hasher;
use fs::File;
use iloveair::audit::read_to_string_with_shellexpand;
use iloveair::notify::read_email_json;
use iloveair::notify::send_email_notification;
use iloveair::notify::EmailConfig;
use iloveair::report::load_json_window_should_be_open;
use std::fs;
use std::io::Write;

fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new("email_config")
                .short('p')
                .long("email")
                .value_name("FILE")
                .default_value("~/.config/iloveair/email.json")
                .required(false)
                .help("config ~/.config/iloveair/email.json"),
        )
        .arg(
            Arg::new("text_in")
                .short('i')
                .long("text-in")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/report.txt"),
        )
        .arg(
            Arg::new("json_in")
                .short('j')
                .long("json-in")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/report.json"),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .required(false)
                .num_args(0)
                .help("don't send notification or write window state"),
        );
    let matches = command.get_matches();

    let Some(email_config_path) = matches.get_one::<String>("email_config") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(text_in_path) = matches.get_one::<String>("text_in") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(json_in_path) = matches.get_one::<String>("json_in") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };

    let is_dry_run = matches.get_flag("dry_run");
    match app_main(email_config_path, text_in_path, json_in_path, is_dry_run) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn is_changed(text_in_path: &str, text_in: &String) -> Result<bool> {
    // Step 1: Define checksum path by changing extension from .txt to .md5
    let checksum_path = text_in_path.replace(".txt", ".email.crc32");

    let mut hasher = Hasher::new();
    hasher.update(text_in.as_bytes());
    let computed_checksum = hasher.finalize().to_string();

    // Step 3: Determine if the checksum has changed
    let changed = match read_to_string_with_shellexpand(&checksum_path) {
        Ok(existing_checksum) => existing_checksum.trim() != computed_checksum,
        Err(_) => true,
    };

    // Step 4: If changed, write the new checksum to the checksum file
    if changed {
        let mut file = File::create(&checksum_path)?;
        writeln!(file, "{}", computed_checksum)?;
        println!("WROTE: {}", checksum_path);
    }

    Ok(changed)
}

fn app_main(
    email_config_path: &String,
    text_in_path: &String,
    json_in_path: &String,
    is_dry_run: bool,
) -> Result<()> {
    let email_config = read_email_json(email_config_path)?;
    let json_window_should_be_open = load_json_window_should_be_open(&json_in_path)?;
    let text_in = read_to_string_with_shellexpand(text_in_path)
        .with_context(|| anyhow!("could not read {}", text_in_path))?;
    let is_changed =
        is_changed(text_in_path, &text_in).with_context(|| anyhow!("error checking checksum"))?;
    if is_changed {
        // subject is: Open Windows: <date>
        let subject = if json_window_should_be_open {
            format!(
                "Open Windows: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            )
        } else {
            format!(
                "Close Windows: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            )
        };
        notify_email(&email_config, is_dry_run, &subject, &text_in)?;
    }
    Ok(())
}
fn notify_email(
    email_config: &EmailConfig,
    is_dry_run: bool,
    subject: &str,
    text_in: &str,
) -> Result<()> {
    println!("send notification");
    send_email_notification(is_dry_run, email_config, subject, text_in)?;
    Ok(())
}
