use anyhow::anyhow;
use std::{fs, path::Path};

pub fn read_to_string_with_shellexpand(path: &String) -> Result<String, anyhow::Error> {
    let expand = shellexpand::full(path)?;
    let expanded = expand.as_ref();

    let content =
        fs::read_to_string(expanded).map_err(|e| anyhow!("error reading {} {}", expand, e));
    println!("READ: {}", expanded);
    content
}
pub fn read_to_string_with_path(path: &Path) -> Result<String, anyhow::Error> {
    let content: Result<String, anyhow::Error> = std::fs::read_to_string(path).map_err(|e| {
        anyhow!(
            "load_weather_response: could not load {} {}",
            path.display(),
            e
        )
    });

    println!("READ: {:?}", path);
    content
}
