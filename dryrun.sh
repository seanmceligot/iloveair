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
cargo run --bin getpollution -- --config $CONFIG_WEATHER --out $CACHE_POLLUTION
cargo run --bin read_waveplus -- --config $CONFIG_AIRTHINGS --indoor $CACHE_INDOOR --token $CACHE_TOKEN
cargo run --bin weather_notify -- --dry-run --pushover $CONFIG_PUSHOVER --weather $CACHE_WEATHER --indoor $CACHE_INDOOR --window $CACHE_WINDOW

