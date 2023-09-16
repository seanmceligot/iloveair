all: weather airapi pol dryrun

airapi: 
	RUST_BACKTRACE=1 cargo run --bin read_waveplus -- --config ~/.config/iloveair/airthings.json --indoor ~/.cache/iloveair/indoor.json --token /home/sean/.cache/iloveair/airthings_token.json

check: 
	cargo check

dryrun: 
	RUST_BACKTRACE=1 cargo run --bin weather_notify -- --pushover ~/.config/iloveair/pushover.json --weather ~/.cache/iloveair/weather.json --indoor ~/.cache/iloveair/indoor.json --window ~/.cache/iloveair/open_windows.state --dry-run
notify: 
	RUST_BACKTRACE=1 cargo run --bin weather_notify -- --pushover ~/.config/iloveair/pushover.json --weather ~/.cache/iloveair/weather.json --indoor ~/.cache/iloveair/indoor.json --window ~/.cache/iloveair/open_windows.state

weather:
	cargo check
	RUST_BACKTRACE=1 cargo run --bin getweather -- --config ~/.config/iloveair/openweathermap.json --out ~/.cache/iloveair/weather.json

pol:
	RUST_BACKTRACE=1 cargo run --bin getpollution -- --config ~/.config/iloveair/openweathermap.json --out ~/.cache/iloveair/pollution.json

pyair: 
	python python/read_waveplus.py

hourly: 
	./hourly.sh

install_service:
	systemctl --user daemon-reload
	systemctl --user enable --now iloveair.timer
	systemctl --user enable --now iloveair.service 

journalctl:
	journalctl --user -u iloveair.service

status:
	systemctl --user status iloveair.service
	systemctl --user status iloveair.timer

backup:
	git bundle create /drive/bundles/iloveair.bunde --all
