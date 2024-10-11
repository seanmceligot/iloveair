CONFIG_AIRTHINGS=~/.config/iloveair/airthings.json
CONFIG_PUSHOVER=~/.config/iloveair/pushover.json
CONFIG_WEATHER=~/.config/iloveair/openweathermap.json

CACHE_INDOOR=~/.cache/iloveair/indoor.json
CACHE_POLLUTION=~/.cache/iloveair/pollution.json
CACHE_TOKEN=~/.cache/iloveair/airthings_token.json
CACHE_WEATHER=~/.cache/iloveair/weather.json
CACHE_WINDOW=~/.cache/iloveair/open_windows.state

all: check weather airapi pol dryrun

check:
	cargo check
	cargo clippy


test: check
	cargo test

list_air:
	RUST_BACKTRACE=1 cargo run --bin read_waveplus -- --list-devices --config $(CONFIG_AIRTHINGS) --indoor $(CACHE_INDOOR) --token $(CACHE_TOKEN)

airapi:
	RUST_BACKTRACE=1 cargo run --bin read_waveplus -- --config $(CONFIG_AIRTHINGS) --indoor $(CACHE_INDOOR) --token $(CACHE_TOKEN)

dryrun:
	RUST_BACKTRACE=1 cargo run --bin weather_notify -- --pushover $(CONFIG_PUSHOVER) --weather $(CACHE_WEATHER) --indoor $(CACHE_INDOOR) --window $(CACHE_WINDOW) --dry-run

notify:
	RUST_BACKTRACE=1 cargo run --bin weather_notify -- --pushover $(CONFIG_PUSHOVER) --weather $(CACHE_WEATHER) --indoor $(CACHE_INDOOR) --window $(CACHE_WINDOW)

weather:
	cargo check
	RUST_BACKTRACE=1 cargo run --bin getweather -- --config $(CONFIG_WEATHER) --out $(CACHE_WEATHER)

pol:
	RUST_BACKTRACE=1 cargo run --bin getpollution -- --config $(CONFIG_WEATHER) --out $(CACHE_POLLUTION)

pyair: 
	python python/read_waveplus.py

hourly: 
	./hourly.sh

install_service:
	systemctl --user enable --now systemd/iloveair.service 
	systemctl --user enable --now systemd/iloveair.timer
	systemctl --user enable --now systemd/rain.service 
	systemctl --user enable --now systemd/rain.timer
	systemctl --user daemon-reload

journalctl:
	journalctl --user -u iloveair.service

status:
	systemctl --user status iloveair.service
	systemctl --user status iloveair.timer

backup:
	git bundle create /drive/bundles/iloveair.bunde --all
docs:
	cargo doc --open

rain: 
	python save_rain_data.py
