import json
import os
import re
from datetime import datetime, timedelta
from typing import List

import requests
from icecream import ic
from lxml import etree, html

""" local_weather.py - Extract weather data from a local HTML file or from http://weather.cos.gmu.edu/ 

to install:
    pip install icecream
    pip install lxml
    pip install requests

"""


def parse_int(s):
    ic(s)
    match = re.search(r"\d+", s)
    if match:
        number = int(match.group())
        ic(number)
        return number
    else:
        ic(f"No int {s}")


def parse_float(s):
    ic(s)
    match = re.search(r"\d+\.\d+", s)
    if match:
        number = float(match.group())
        ic(number)
        return number
    else:
        ic(f"No float {s}")


def debug_str(zz) -> str:
    ic(zz)
    if isinstance(zz, List):
        for item in zz:
            return debug_str(item)
    elif isinstance(zz, html.HtmlElement):
        ic(html.tostring(zz))
        raise ValueError(f"Unexpected type {type(zz)}")
    elif isinstance(zz, etree._ElementUnicodeResult):
        return zz.strip()
    else:
        ic((type(zz), zz))
        raise ValueError(f"Unexpected type {type(zz)}")


def debug_int(zz) -> int:
    ic(zz)
    if isinstance(zz, List):
        raise ValueError(f"Unexpected type {type(zz)}")
    # @for item in zz:
    # @       return debug_int(item)
    elif isinstance(zz, html.HtmlElement):
        ic(html.tostring(zz))
        raise ValueError(f"Unexpected type {type(zz)}")
    elif isinstance(zz, etree._ElementUnicodeResult):
        return parse_int(zz.strip())
    else:
        ic((type(zz), zz))
        raise ValueError(f"Unexpected type {type(zz)}")


def debug_float(zz) -> float:
    ic(zz)
    if isinstance(zz, List):
        for item in zz:
            return debug_float(item)
    elif isinstance(zz, html.HtmlElement):
        ic(html.tostring(zz))
        raise ValueError(f"Unexpected type {type(zz)}")
    elif isinstance(zz, etree._ElementUnicodeResult):
        return parse_float(zz.strip())
    else:
        ic((type(zz), zz))
        raise ValueError(f"Unexpected type {type(zz)}")


def main():
    url = "http://weather.cos.gmu.edu/"

    download = not os.path.exists("weather.html")
    # if weather.html doesn't exist, or is more than 10 minutes old, download it
    if os.path.exists("weather.html"):
        last_weather_date = datetime.fromtimestamp(os.stat("weather.html").st_mtime)
        now = datetime.now()
        if now - last_weather_date > timedelta(minutes=10):
            download = True
    if download:
        ic(f"Downloading weather.html from {url}")
        response = requests.get(url)
        with open("weather.html", "w") as json_file:
            json_file.write(response.text)

    # get modify date of weather.html
    weather_date = datetime.fromtimestamp(os.stat("weather.html").st_mtime)
    # remove seconds from weather_date
    weather_date = weather_date.replace(second=0, microsecond=0)
    now = datetime.now()
    ic(weather_date)
    ic(now - weather_date)

    tree = html.parse("weather.html")

    # Find the table element
    html_table = tree.xpath('//table[@border="1"]')[0]

    # Extract the weather data from the table using XPath
    temperature = debug_float(
        html_table.xpath("//tr[3]/td[2]/font/strong/small/font/text()")[0]
    )
    ic(temperature)
    assert temperature
    humidity = debug_int(
        html_table.xpath("//tr[4]/td[2]/font/strong/small/font/text()")[0]
    )
    ic(humidity)
    assert humidity
    # humidity = debug_int(table.xpath("//tr[3]/td[2]/font/strong/small/font/text()"))
    dewpoint = debug_float(
        html_table.xpath("//tr[5]/td[2]/font/strong/small/font/text()")
    )
    ic(dewpoint)
    assert dewpoint
    wind = debug_str(html_table.xpath("//tr[6]/td[2]/font/strong/font/text()"))
    barometer = debug_float(
        html_table.xpath("//tr[7]/td[2]/font/strong/small/font/text()")
    )
    today_rain = debug_float(
        html_table.xpath("//tr[8]/td[2]/font/strong/small/font/text()")
    )
    yearly_rain = debug_float(
        html_table.xpath("//tr[9]/td[2]/font/strong/small/font/text()")
    )
    wind_chill = debug_float(
        html_table.xpath("//tr[10]/td[2]/font/strong/font/small/text()")
    )
    ic(wind_chill)
    thw_index = debug_float(
        html_table.xpath("//tr[11]/td[2]/font/strong/font/small/text()")
    )
    heat_index = debug_float(
        html_table.xpath("//tr[12]/td[2]/font/strong/font/small/text()")
    )

    # Print the extracted weather data
    ic("Temperature:", temperature)
    ic("Humidity:", humidity)
    ic("Dewpoint:", dewpoint)
    ic("Wind:", wind)
    ic("Barometer:", barometer)
    ic("Today's Rain:", today_rain)
    ic("Yearly Rain:", yearly_rain)
    ic("Wind Chill:", wind_chill)
    ic("THW Index:", thw_index)
    ic("Heat Index:", heat_index)

    # Write the weather data to a Parquet file
    names = [
        "weather_date",
        "temperature",
        "humidity",
        "dewpoint",
        "wind",
        "barometer",
        "today_rain",
        "yearly_rain",
        "wind_chill",
        "thw_index",
        "heat_index",
    ]
    row = [
        weather_date,
        temperature,
        humidity,
        dewpoint,
        wind,
        barometer,
        today_rain,
        yearly_rain,
        wind_chill,
        thw_index,
        heat_index,
    ]
    if os.path.exists("weather.json"):
        with open("weather.json", "r") as json_file:
            """Read the weather data from a JSON file"""
            saved_json = json.load(json_file)
            ic(saved_json)
            saved_weather_date = datetime.strptime(
                saved_json["weather_date"], "%Y-%m-%d %H:%M"
            )
            ic(saved_weather_date)
            ic(weather_date)
            write_json = saved_weather_date < weather_date
            if not write_json:
                ic("Already have weather data for", weather_date)
                ic(saved_json)
                ic(row)
    else:
        write_json = True

    if write_json:
        """Write the weather data to a JSON file"""
        ic("New weather data")
        ic(row)
        with open("weather.json", "w") as json_file:
            json.dump(
                {
                    "weather_date": weather_date.strftime("%Y-%m-%d %H:%M"),
                    "temperature": temperature,
                    "humidity": humidity,
                    "dewpoint": dewpoint,
                    "wind": wind,
                    "barometer": barometer,
                    "today_rain": today_rain,
                    "yearly_rain": yearly_rain,
                    "wind_chill": wind_chill,
                    "thw_index": thw_index,
                    "heat_index": heat_index,
                },
                json_file,
            )


if __name__ == "__main__":
    main()
