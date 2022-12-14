test:
	cargo test
	cargo test --features serde1
	cargo test --all-features

watch:
	cargo watch -x test

watch_dummy:
	cargo watch -s "cd dummy && cargo run"
