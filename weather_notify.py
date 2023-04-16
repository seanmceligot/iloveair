"""



read waveplus.json and weather.json

if the outdoor Humidity is less than the indoor Humidity and the outdoor temperature is between 50 and 90 degrees, send a notification

"""

import http.client
import json
import os
import urllib

from icecream import ic


def read_pushover_json():
    pushover_file = os.path.join(os.path.dirname(__file__), "pushover.json")
    with open(pushover_file, "r") as f:
        pushover = json.load(f)
    ic(pushover)
    pushover_api_key = pushover["api_key"]
    pushover_user_key = pushover["user_key"]
    return pushover_api_key, pushover_user_key


def read_waveplus_json():
    waveplus_file = os.path.join(os.path.dirname(__file__), "waveplus.json")

    """waveplus.json example:
    { "date": "2023-04-15 21:18", "Humidity": "61.5 %rH", "Radon ST avg":
    "101 Bq/m3", "Radon LT avg": "81 Bq/m3", "Temperature": "73.13f",
    "Pressure": "998.98 hPa", "CO2 level": "716.0 ppm", "VOC level": "229.0
    ppb", "time": "21:18:01"}
    """
    with open(waveplus_file, "r") as f:
        waveplus = json.load(f)
    ic(waveplus)
    indoor_humidity = float(waveplus["Humidity"].split()[0])
    indoor_temp = float(waveplus["Temperature"].split()[0])
    return indoor_humidity, indoor_temp


def read_weather_json():
    """weather.json example :

    {"weather_date": "2023-04-15 10:21", "temperature": 57.3,
    "humidity": 35, "dewpoint": 29.9, "wind": null, "barometer": 29.988,
    "today_rain": 0.0, "yearly_rain": 1.32, "wind_chill": 57.3, "thw_index":
     54.4, "heat_index": 54.4}
    """
    weather_file = os.path.join(os.path.dirname(__file__), "weather.json")
    with open(weather_file, "r") as f:
        weather = json.load(f)

    outdoor_humidity = float(weather["humidity"])
    outdoor_temp = float(weather["temperature"])
    return outdoor_humidity, outdoor_temp


def send_pushover_notification(msg):
    api_key, user_key = read_pushover_json()
    conn = http.client.HTTPSConnection("api.pushover.net:443")
    conn.request(
        "POST",
        "/1/messages.json",
        urllib.parse.urlencode(
            {
                "token": api_key,
                "user": user_key,
                "message": msg,
            }
        ),
        {"Content-type": "application/x-www-form-urlencoded"},
    )
    conn.getresponse()


def main():
    indoor_humidity, indoor_temp = read_waveplus_json()
    outdoor_humidity, outdoor_temp = read_weather_json()

    print(f"indoor_humidity: {indoor_humidity}")
    print(f"outdoor_humidity: {outdoor_humidity}")
    print(f"indoor temp: {indoor_temp}")
    print(f"outdoor_temp: {outdoor_temp}")

    can_let_in_humidify = outdoor_humidity < indoor_humidity
    can_let_in_temperature = outdoor_temp > 50 and outdoor_temp < 90
    print(f"can_let_in_humidify: {can_let_in_humidify}")
    print(f"can_let_in_temperature: {can_let_in_temperature}")
    if can_let_in_humidify and can_let_in_temperature:
        print("send notification")
        send_pushover_notification(
            f"open the windows 🪟 outdoor temp: {outdoor_temp} outdoor_humidity: {outdoor_humidity}"
        )
    else:
        print("no notification")


if __name__ == "__main__":
    main()
