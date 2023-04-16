import json
import os
from datetime import datetime

import requests
from icecream import ic
from lxml import etree, html
from util import is_recent_file, save_request_response
from xml_util import xpath_get_float, xpath_get_int, xpath_get_str

""" local_weather.py - Extract weather data from a local HTML file or from http://weather.cos.gmu.edu/ 

to install:
    pip install icecream
    pip install lxml
    pip install requests

"""


def main():
    url = "http://weather.cos.gmu.edu/"

    download: bool = not os.path.exists("weather.html")
    # if weather.html doesn't exist, or is more than 10 minutes old, download it
    if is_recent_file("weather.html"):
        download = True
    if download:
        ic(f"Downloading weather.html from {url}")
        response = requests.get(url)
        save_request_response(response, "weather.html")

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
    temperature = xpath_get_float(
        html_table.xpath("//tr[3]/td[2]/font/strong/small/font/text()")[0]
    )
    ic(temperature)
    assert temperature
    humidity = xpath_get_int(
        html_table.xpath("//tr[4]/td[2]/font/strong/small/font/text()")[0]
    )
    ic(humidity)
    assert humidity
    # humidity = debug_int(table.xpath("//tr[3]/td[2]/font/strong/small/font/text()"))
    dewpoint = xpath_get_float(
        html_table.xpath("//tr[5]/td[2]/font/strong/small/font/text()")[0]
    )
    ic(dewpoint)
    assert dewpoint
    wind = xpath_get_str(
        html_table.xpath("//tr[6]/td[2]/font/strong/font/small/text()")
    )
    ic(wind)
    assert wind
    barometer = xpath_get_float(
        html_table.xpath("//tr[7]/td[2]/font/strong/small/font/text()")
    )
    today_rain = xpath_get_float(
        html_table.xpath("//tr[8]/td[2]/font/strong/small/font/text()")
    )
    yearly_rain = xpath_get_float(
        html_table.xpath("//tr[9]/td[2]/font/strong/small/font/text()")
    )
    wind_chill = xpath_get_float(
        html_table.xpath("//tr[10]/td[2]/font/strong/font/small/text()")
    )
    ic(wind_chill)
    thw_index = xpath_get_float(
        html_table.xpath("//tr[11]/td[2]/font/strong/font/small/text()")
    )
    heat_index = xpath_get_float(
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
        "date",
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
            saved_weather_date = datetime.strptime(saved_json["date"], "%Y-%m-%d %H:%M")
            ic(saved_weather_date)
            ic(weather_date)
            write_json: bool = saved_weather_date < weather_date
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
                    "date": weather_date.strftime("%Y-%m-%d %H:%M"),
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
