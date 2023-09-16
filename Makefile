t:  weather

airapi: 
	RUST_BACKTRACE=1 cargo run --bin read_waveplus

check: 
	cargo check

notify: 
	cargo build
	RUST_BACKTRACE=1 cargo run --bin weather_notify

weather:
	cargo check
	RUST_BACKTRACE=1 cargo run --bin getweather -- --config ~/.config/iloveair/openweathermap.json --out ~/.cache/iloveair/weather.json
air: 
	python python/read_waveplus.py

pol:
	cargo run --bin getpollution

all: weather air notify

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
