notify: 
	cargo check
	cargo build
	RUST_BACKTRACE=1 cargo run --bin weather_notify

weather:
	cargo check
	RUST_BACKTRACE=1 cargo run --bin getweather

air: 
	python python/read_waveplus.py


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
