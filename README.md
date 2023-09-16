# I Love Air

A collection of tools to monitor and notify about weather and air quality.

## Configuration Files

The tools utilize various configuration files which are expected to be located in the `~/.config/iloveair/` directory:

- **Airthings Configuration**: `~/.config/iloveair/airthings.json`
- **Pushover Configuration**: `~/.config/iloveair/pushover.json`
- **OpenWeatherMap Configuration**: `~/.config/iloveair/openweathermap.json`

## Cache Files

Some commands generate or utilize cache files stored in the `~/.cache/iloveair/` directory:

- **Indoor Data**: `~/.cache/iloveair/indoor.json`
- **Pollution Data**: `~/.cache/iloveair/pollution.json`
- **Airthings Token**: `~/.cache/iloveair/airthings_token.json`
- **Weather Data**: `~/.cache/iloveair/weather.json`
- **Window State**: `~/.cache/iloveair/open_windows.state`

## Commands

### Get Weather

Fetches the current weather data:

```bash
cargo run --bin getweather -- --config $CONFIG_WEATHER --out $CACHE_WEATHER
```

Get Pollution

Fetches the current pollution data:

```bash
cargo run --bin getpollution -- --config $CONFIG_WEATHER --out $CACHE_POLLUTION

```

Read WavePlus

Reads data from the Airthings API:

```bash
cargo run --bin read_waveplus -- --config $CONFIG_AIRTHINGS --indoor $CACHE_INDOOR --token $CACHE_TOKEN

```

Notification

Tells you if you can open your windows based on indoor and outdoor temp and humidity

```bash
cargo run --bin weather_notify -- --pushover $CONFIG_PUSHOVER --weather $CACHE_WEATHER --indoor $CACHE_INDOOR --window $CACHE_WINDOW
```
