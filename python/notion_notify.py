import subprocess
import requests
import os
import json
import argparse
from typing import List, Final
from icecream import ic

# Configuration paths
NOTION_CONFIG_PATH: Final[str] = os.path.expanduser("~/.config/iloveair/notion.json")
CACHE_DIR: Final[str] = os.path.expanduser("~/.cache/iloveair/")


class Config:
    notion_api_key: Final[str]
    notion_page_id: Final[str]

    def __init__(self, notion_api_key: str, notion_page_id: str):
        self.notion_api_key = notion_api_key
        self.notion_page_id = notion_page_id


def load_config(name: str) -> Config:
    # load notion api key and page id from config file
    if os.path.exists(NOTION_CONFIG_PATH):
        with open(NOTION_CONFIG_PATH, "r") as config_file:
            config_data = json.load(config_file)
            notion_api_key = config_data.get("notion_api_key", None)
            notion_page_id = config_data.get(f"{name}_page_id", None)
    ic(notion_page_id)
    assert notion_page_id, "notion_page_id"
    assert notion_api_key, "notion_api_key"
    return Config(notion_api_key=notion_api_key, notion_page_id=notion_page_id)


def run_command(command_args: List[str]) -> str:
    """Runs the given command to fetch output data."""
    command = " ".join(command_args)
    ic(command)
    try:
        output = subprocess.check_output(command, shell=True).decode("utf-8")
        ic(output)
        return output
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e}")
        return None


def parse_output_data(data: str) -> List[str]:
    """Parses the output data from the command."""
    lines = data.strip().split("\n")
    return lines


def replace_notion_page_content(config: Config, lines: List[str]) -> None:
    """Replaces the existing content of the Notion page with the new output data."""
    # Delete existing children
    ic(config.notion_page_id)
    assert config.notion_page_id, "config.notion_page_id"
    url = f"https://api.notion.com/v1/blocks/{config.notion_page_id}/children"
    ic(url)
    # Define headers for Notion API requests
    headers = {
        "Authorization": f"Bearer {config.notion_api_key}",
        "Content-Type": "application/json",
        "Notion-Version": "2022-06-28",
    }

    response = requests.get(url, headers=headers)
    if response.status_code == 200:
        existing_children = response.json().get("results", [])
        for child in existing_children:
            delete_url = f"https://api.notion.com/v1/blocks/{child['id']}"
            delete_response = requests.delete(delete_url, headers=headers)
            if delete_response.status_code != 200:
                print(
                    f"Failed to delete block {child['id']}. Status Code: {delete_response.status_code}, Response: {delete_response.text}"
                )
    else:
        print(
            f"Failed to retrieve existing children. Status Code: {response.status_code}, Response: {response.text}"
        )
        return

    # Add new children with updated content
    children = []
    for line in lines:
        children.append(
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{"type": "text", "text": {"content": f"{line}"}}]
                },
            }
        )
    response = requests.patch(url, headers=headers, json={"children": children})
    if response.status_code == 200:
        print("Notion page replaced successfully!")
    else:
        print(
            f"Failed to replace Notion page content. Status Code: {response.status_code}, Response: {response.text}"
        )


def write_output_to_file(lines: List[str], name: str) -> None:
    """Writes the output data to a file."""
    cache_output_txt = os.path.join(CACHE_DIR, f"{name}.txt")
    with open(cache_output_txt, "w") as file:
        file.write("\n".join(lines))


def read_output_from_file(name: str) -> List[str]:
    """Reads the output data from the file if it exists."""
    cache_output_txt = os.path.join(CACHE_DIR, f"{name}.txt")
    if os.path.exists(cache_output_txt):
        with open(cache_output_txt, "r") as file:
            return file.read().splitlines()
    return []


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run a command and update Notion with its output."
    )
    parser.add_argument(
        "command", nargs="*", help="The command to run, with arguments."
    )
    parser.add_argument(
        "--name", required=True, help="The name to use for caching the output file."
    )
    args = parser.parse_args()

    config = load_config(args.name)

    if not args.command:
        print("No command provided. Exiting.")
        return

    output_data = run_command(args.command)
    if output_data:
        lines = parse_output_data(output_data)
        cached_lines = read_output_from_file(args.name)

        if lines != cached_lines:
            replace_notion_page_content(config, lines)
            write_output_to_file(lines, args.name)
        else:
            print("No changes in output data. Notion page update skipped.")


if __name__ == "__main__":
    main()
