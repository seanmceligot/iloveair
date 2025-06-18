#!/bin/bash

# cd to this directory
cd "$(dirname "$0")"

trap 'notify-send "ERROR: iloveair" "journalctl --user -u iloveair"; exit 1' ERR

# Define configurations and cache paths
CONFIG_AIRTHINGS=~/.config/iloveair/airthings.json
CONFIG_PUSHOVER=~/.config/iloveair/pushover.json
CONFIG_WEATHER=~/.config/iloveair/openweathermap.json
CONFIG_NOTION=~/.config/iloveair/notion.json
CONFIG_EMAIL=~//.config/iloveair/email.json

CACHE_INDOOR=~/.cache/iloveair/indoor.json
CACHE_POLLUTION=~/.cache/iloveair/pollution.json
CACHE_TOKEN=~/.cache/iloveair/airthings_token.json
CACHE_WEATHER=~/.cache/iloveair/weather.json
CACHE_WINDOW=~/.cache/iloveair/open_windows.state

REPORT_TEXT=~/.cache/iloveair/report.txt
REPORT_JSON=~/.cache/iloveair/report.json
RAIN_REPORT=~/.cache/iloveair/Rain.txt

CAT="bat -P --style=header-filename"

# Define functions
clean() {
    rm -v ${REPORT_TEXT}
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

weather_report() {
    cargo run --bin weather_report -- --weather ${CACHE_WEATHER} --indoor ${CACHE_INDOOR} --text-out ${REPORT_TEXT} --json-out ${REPORT_JSON}
    ${CAT} $REPORT_TEXT
    ${CAT} $REPORT_JSON
}

pushover_notify() {
    cargo run --bin pushover_notify -- --pushover ${CONFIG_PUSHOVER} --text-in ${REPORT_TEXT}
}
notion_notify() {
    cargo run --bin notion_notify -- --config ${CONFIG_NOTION} --name Indoor --text-in ${REPORT_TEXT}
}
notion_notify_rain() {
    cargo run --bin notion_notify -- --config ${CONFIG_NOTION} --name Rain --text-in ${RAIN_REPORT}
}
rain() {
	python python/save_rain_data.py
}
email_notify() {
    cargo run --bin email_notify -- --email ${CONFIG_EMAIL} --text-in ${REPORT_TEXT} --json-in ${REPORT_JSON}
}

# Check for argument or run all functions
if [ -n "$1" ]; then
    case "$1" in
        clean) clean ;;
        getweather) getweather ;;
        getpollution) getpollution ;;
        read_waveplus) read_waveplus ;;
        report) weather_report ;;
        weather_report) weather_report ;;
        pushover_notify) pushover_notify ;;
        notion_notify) notion_notify ;;
        notion_notify_rain) notion_notify_rain ;;
        rain) rain;;
        email) email_notify;;
        *) echo "Unknown command: $1" ;;
    esac
else
    getweather
    getpollution
    read_waveplus
    weather_report
    email_notify
fi
