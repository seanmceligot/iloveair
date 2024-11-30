use anyhow::anyhow;
use anyhow::{Context, Result};
use clap::{command, Arg};
use crc32fast::Hasher;
use fs::File;
use iloveair::notify::read_pushover_json;
use iloveair::notify::send_pushover_notification;
use iloveair::notify::PushoverConfig;
use std::fs;
use std::io::Write;

fn main() {
    let command = command!()
        .version("0.9")
        .arg(
            Arg::new("pushover_config")
                .short('p')
                .long("pushover")
                .value_name("FILE")
                .default_value("~/.config/iloveair/pushover.json")
                .required(false)
                .help("config ~/.config/iloveair/pushover.json"),
        )
        .arg(
            Arg::new("text_in")
                .short('i')
                .long("text-in")
                .value_name("FILE")
                .required(true)
                .help("~/.cache/iloveair/Indoor.txt"),
        )
        .arg(
            Arg::new("dry_run")
                .long("dry-run")
                .required(false)
                .num_args(0)
                .help("don't send notification or write window state"),
        );
    let matches = command.get_matches();

    let Some(pushover_config_path) = matches.get_one::<String>("pushover_config") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };
    let Some(text_in_path) = matches.get_one::<String>("text_in") else {
        // This else block is unreachable because the argument is required.
        unreachable!();
    };

    let is_dry_run = matches.get_flag("dry_run");
    match app_main(pushover_config_path, text_in_path, is_dry_run) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn is_changed(text_in_path: &str, text_in: &String) -> Result<bool> {
    // Step 1: Define checksum path by changing extension from .txt to .md5
    let checksum_path = text_in_path.replace(".txt", ".crc32");

    let mut hasher = Hasher::new();
    hasher.update(text_in.as_bytes());
    let computed_checksum = hasher.finalize().to_string();

    // Step 3: Determine if the checksum has changed
    let changed = match fs::read_to_string(&checksum_path) {
        Ok(existing_checksum) => existing_checksum.trim() != computed_checksum,
        Err(_) => true,
    };

    // Step 4: If changed, write the new checksum to the checksum file
    if changed {
        let mut file = File::create(&checksum_path)?;
        writeln!(file, "{}", computed_checksum)?;
    }

    Ok(changed)
}

fn app_main(pushover_config_path: &String, text_in_path: &String, is_dry_run: bool) -> Result<()> {
    let pushover_config = read_pushover_json(pushover_config_path)?;
    let text_in = fs::read_to_string(text_in_path)
        .with_context(|| anyhow!("could not read {}", text_in_path))?;
    let is_changed =
        is_changed(text_in_path, &text_in).with_context(|| anyhow!("error checking checksum"))?;
    if is_changed {
        notify_pushover(&pushover_config, is_dry_run, &text_in)?;
    }
    Ok(())
}
fn notify_pushover(
    pushover_config: &PushoverConfig,
    is_dry_run: bool,
    text_in: &str,
) -> Result<()> {
    println!("send notification");
    send_pushover_notification(is_dry_run, pushover_config, text_in)?;
    Ok(())
}
