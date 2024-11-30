#!/bin/bash

cd ~/git/rust/iloveair
CONFIG_AIRTHINGS=~/.config/iloveair/airthings.json
CONFIG_PUSHOVER=~/.config/iloveair/pushover.json
CONFIG_WEATHER=~/.config/iloveair/openweathermap.json

CACHE_INDOOR=~/.cache/iloveair/indoor.json
CACHE_POLLUTION=~/.cache/iloveair/pollution.json
CACHE_TOKEN=~/.cache/iloveair/airthings_token.json
CACHE_WEATHER=~/.cache/iloveair/weather.json
CACHE_WINDOW=~/.cache/iloveair/open_windows.state


cargo run --bin getweather -- --config $CONFIG_WEATHER --out $CACHE_WEATHER
#bat $CACHE_WEATHER
cargo run --bin getpollution -- --config $CONFIG_WEATHER --out $CACHE_POLLUTION
#bat $CACHE_POLLUTION
cargo run --bin read_waveplus -- --config $CONFIG_AIRTHINGS --indoor $CACHE_INDOOR --token $CACHE_TOKEN
#bat $CACHE_INDOOR
#cargo run --bin weather_notify -- --pushover $CONFIG_PUSHOVER --weather $CACHE_WEATHER --indoor $CACHE_INDOOR --window $CACHE_WINDOW
cargo run --bin weather_notify --dry-run --pushover $CONFIG_PUSHOVER --weather $CACHE_WEATHER --indoor $CACHE_INDOOR --window $CACHE_WINDOW
set -e
/home/sean/git/python/venv/bin/python python/notion_notify.py --name Rain "~/git/python/venv/bin/python python/save_rain_data.py"

# original
#/home/sean/git/python/venv/bin/python python/notion_notify.py --name Indoor "~/.cargo/bin/weather_notify --dry-run --pushover ${CONFIG_PUSHOVER} --weather ${CACHE_WEATHER} --indoor ${CACHE_INDOOR} --window ${CACHE_WINDOW}"

# without pushover
~/.cargo/bin/weather_notify --weather ${CACHE_WEATHER} --indoor ${CACHE_INDOOR} --window ${CACHE_WINDOW} --text-out  /home/sean/.cache/iloveair/Indoor.txt

# pushover only
/home/sean/git/python/venv/bin/python python/pushover_notify.py --name Indoor --pushover ${CONFIG_PUSHOVER}  --text-in /home/sean/.cache/iloveair/Indoor.txt

# notion only Indoor
/home/sean/git/python/venv/bin/python python/notion_notify.py --name Indoor --text-in /home/sean/.cache/iloveair/Indoor.txt
