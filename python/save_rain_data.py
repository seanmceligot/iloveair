import requests
from bs4 import BeautifulSoup
import re
import polars as pl
from datetime import datetime, timedelta
import os
import json

NOTION_CONFIG_PATH = os.path.expanduser("~/.config/iloveair/rain.json")
RAIN_DATA_URL = ""
RAIN_REGEX = ""

if os.path.exists(NOTION_CONFIG_PATH):
    with open(NOTION_CONFIG_PATH, 'r') as config_file:
        config_data = json.load(config_file)
        RAIN_DATA_URL = config_data.get("rain_data_url", "")
        RAIN_REGEX = config_data.get("rain_regex", "")

def scrape_weather_data():
    url = RAIN_DATA_URL
    
    try:
        response = requests.get(url)
        response.raise_for_status()
    except requests.RequestException as e:
        print(f"Error fetching the webpage: {e}")
        return None

    soup = BeautifulSoup(response.text, 'html.parser')
    pre_tag = soup.find('pre', class_='glossaryProduct')

    if not pre_tag:
        print("Could not find the required <pre> tag.")
        return None

    content = pre_tag.get_text()

    rain_regex = RAIN_REGEX
    
    match = re.search(rain_regex, content)

    if not match:
        print(f"Could not match regex data. {RAIN_DATA_URL} {rain_regex}")
        print(content)
        return None

    high_temp, low_temp, precipitation = match.groups()

    return {
        "high_temperature": int(high_temp),
        "low_temperature": int(low_temp),
        "precipitation": 0.0 if precipitation in ("M","T")  else float(precipitation)
    }

def save_precipitation_to_csv(precipitation):
    # Calculate yesterday's date
    yesterday = (datetime.now() - timedelta(days=1)).strftime('%Y-%m-%d')
    csv_path = os.path.expanduser('~/.cache/air/rain.csv')
    
    # Ensure the directory exists
    os.makedirs(os.path.dirname(csv_path), exist_ok=True)
    
    # Create a new DataFrame with yesterday's data
    new_row = pl.DataFrame({
        'Date': [yesterday],
        'Pcpn': [precipitation]
    })
    
    # If file exists, read it and append the new row. Otherwise, use the new row as is.
    if os.path.exists(csv_path):
        df = pl.read_csv(csv_path)
        df = pl.concat([df, new_row])
    else:
        df = new_row
    
    # Write the DataFrame to CSV
    df.write_csv(csv_path)
    
    print(f"Precipitation data saved to {csv_path}")

def main():
    data = scrape_weather_data()
    if data:
        print("Values represent:")
        print("- High temperature from yesterday")
        print("- Low temperature over the last 12 hours (since 7 PM LST yesterday)")
        print("- Precipitation over the last 24 hours (since 7 AM LST yesterday)")

        print("\nDulles International Airport Weather Data:")
        print(f"High Temperature: {data['high_temperature']}°F")
        print(f"Low Temperature: {data['low_temperature']}°F")
        print(f"Precipitation: {data['precipitation']} inches")
        
        save_precipitation_to_csv(data['precipitation'])
    else:
        print("Failed to retrieve weather data.")

if __name__ == "__main__":
    main()
