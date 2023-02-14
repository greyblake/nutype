test:
	cargo test
	cargo test --features serde1
	cargo test --all-features

watch:
	cargo watch -x test

watch_dummy:
	cargo watch -s "cd dummy && cargo run"

build: build-readme test
	cargo build --release

# Depends on `cargo-readme`: `cargo install cargo-readme`
build-readme:
	cargo readme -r nutype -i src/lib.rs -o README.md
