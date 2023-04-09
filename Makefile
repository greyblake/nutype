test: clippy
	cargo test
	cargo test --features serde1
	cargo test --features regex1
	cargo test --features new_unchecked
	cargo test --features schemars08
	cargo test --all-features

watch:
	cargo watch -x test

watch_dummy:
	cargo watch -s "cd dummy && cargo run"

clippy:
	cargo clippy -- -D warnings
