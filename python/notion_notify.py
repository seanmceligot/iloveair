import subprocess
import requests
import os
import json
import argparse
from typing import List

# Set up the required Notion parameters
NOTION_CONFIG_PATH = os.path.expanduser("~/.config/iloveair/notion.json")
NOTION_API_KEY = ""
NOTION_PAGE_ID = ""

# Load Notion API Key and Page ID from config file
if os.path.exists(NOTION_CONFIG_PATH):
    with open(NOTION_CONFIG_PATH, 'r') as config_file:
        config_data = json.load(config_file)
        NOTION_API_KEY = config_data.get("notion_api_key", "")
        NOTION_PAGE_ID = config_data.get("page_id", "")

# Define headers for Notion API requests
headers = {
    "Authorization": f"Bearer {NOTION_API_KEY}",
    "Content-Type": "application/json",
    "Notion-Version": "2022-06-28"
}

def run_command(command_args: List[str]):
    """Runs the given command to fetch output data."""
    command = " ".join(command_args)
    try:
        output = subprocess.check_output(command, shell=True).decode("utf-8")
        return output
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e}")
        return None

def parse_output_data(data) -> List[str]:
    """Parses the output data from the command."""
    lines = data.strip().split('\n')
    return lines

def replace_notion_page_content(lines):
    """Replaces the existing content of the Notion page with the new output data."""
    # Delete existing children
    url = f"https://api.notion.com/v1/blocks/{NOTION_PAGE_ID}/children"
    response = requests.get(url, headers=headers)
    if response.status_code == 200:
        existing_children = response.json().get("results", [])
        for child in existing_children:
            delete_url = f"https://api.notion.com/v1/blocks/{child['id']}"
            delete_response = requests.delete(delete_url, headers=headers)
            if delete_response.status_code != 200:
                print(f"Failed to delete block {child['id']}. Status Code: {delete_response.status_code}, Response: {delete_response.text}")
    else:
        print(f"Failed to retrieve existing children. Status Code: {response.status_code}, Response: {response.text}")
        return

    # Add new children with updated content
    children = []
    for line in lines:
        children.append({
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{"type": "text", "text": {"content": f"{line}"}}]
            }
        })
    response = requests.patch(url, headers=headers, json={"children": children})
    if response.status_code == 200:
        print("Notion page replaced successfully!")
    else:
        print(f"Failed to replace Notion page content. Status Code: {response.status_code}, Response: {response.text}")

def write_output_to_file(lines, name):
    """Writes the output data to a file."""
    cache_output_txt = os.path.expanduser(f"~/.cache/iloveair/{name}.txt")
    with open(cache_output_txt, 'w') as file:
        file.write('\n'.join(lines))

def read_output_from_file(name) -> List[str]:
    """Reads the output data from the file if it exists."""
    cache_output_txt = os.path.expanduser(f"~/.cache/iloveair/{name}.txt")
    if os.path.exists(cache_output_txt):
        with open(cache_output_txt, 'r') as file:
            return file.read().splitlines()
    return []

def main():
    parser = argparse.ArgumentParser(description="Run a command and update Notion with its output.")
    parser.add_argument('command', nargs='*', help="The command to run, with arguments.")
    parser.add_argument('--name', required=True, help="The name to use for caching the output file.")
    args = parser.parse_args()

    if not args.command:
        print("No command provided. Exiting.")
        return

    output_data = run_command(args.command)
    if output_data:
        lines = parse_output_data(output_data)
        cached_lines = read_output_from_file(args.name)

        if lines != cached_lines:
            replace_notion_page_content(lines)
            write_output_to_file(lines, args.name)
        else:
            print("No changes in output data. Notion page update skipped.")

if __name__ == "__main__":
    main()

