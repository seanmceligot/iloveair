#!/bin/bash

trap 'notify-send "i love air script" "An error occurred while running the script."; exit 1' ERR

# Define configurations and cache paths
cd ~/git/rust/iloveair
CONFIG_AIRTHINGS=~/.config/iloveair/airthings.json
CONFIG_PUSHOVER=~/.config/iloveair/pushover.json
CONFIG_WEATHER=~/.config/iloveair/openweathermap.json
CONFIG_NOTION=~/.config/iloveair/notion.json

CACHE_INDOOR=~/.cache/iloveair/indoor.json
CACHE_POLLUTION=~/.cache/iloveair/pollution.json
CACHE_TOKEN=~/.cache/iloveair/airthings_token.json
CACHE_WEATHER=~/.cache/iloveair/weather.json
CACHE_WINDOW=~/.cache/iloveair/open_windows.state

INDOOR_REPORT=~/.cache/iloveair/Indoor.txt
RAIN_REPORT=~/.cache/iloveair/Rain.txt

CAT="bat -P --style=header-filename"

# Define functions
clean() {
    rm -v ${INDOOR_REPORT}
}

getweather() {
    cargo run --bin getweather -- --config $CONFIG_WEATHER --out $CACHE_WEATHER
    ${CAT} $CACHE_WEATHER
}

getpollution() {
    cargo run --bin getpollution -- --config $CONFIG_WEATHER --out $CACHE_POLLUTION
    ${CAT} $CACHE_POLLUTION
}

read_waveplus() {
    cargo run --bin read_waveplus -- --config $CONFIG_AIRTHINGS --indoor $CACHE_INDOOR --token $CACHE_TOKEN
    ${CAT} $CACHE_INDOOR
}

weather_notify() {
    cargo run --bin weather_report -- --weather ${CACHE_WEATHER} --indoor ${CACHE_INDOOR} --text-out ${INDOOR_REPORT}
    ${CAT} $INDOOR_REPORT
}

pushover_notify() {
    cargo run --bin pushover_notify -- --pushover ${CONFIG_PUSHOVER} --text-in ${INDOOR_REPORT}
}

notion_notify() {
    cargo run --bin notion_notify -- --config ${CONFIG_NOTION} --name Indoor --text-in ${INDOOR_REPORT}
}
notion_notify_rain() {
    cargo run --bin notion_notify -- --config ${CONFIG_NOTION} --name Rain --text-in ${RAIN_REPORT}
}
rain() {
	/home/sean/git/python/venv/bin/python python/save_rain_data.py
}

# Check for argument or run all functions
if [ -n "$1" ]; then
    case "$1" in
        clean) clean ;;
        getweather) getweather ;;
        getpollution) getpollution ;;
        read_waveplus) read_waveplus ;;
        weather_notify) weather_notify ;;
        pushover_notify) pushover_notify ;;
        notion_notify) notion_notify ;;
        notion_notify_rain) notion_notify_rain ;;
        rain) rain;;
        *) echo "Unknown command: $1" ;;
    esac
else
    getweather
    getpollution
    read_waveplus
    weather_notify
    #notion_notify
fi

