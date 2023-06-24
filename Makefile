test: clippy
	cargo test --features nutype_test
	cargo test --features nutype_test,serde
	cargo test --features nutype_test,regex
	cargo test --features nutype_test,new_unchecked
	cargo test --features nutype_test,schemars08
	cargo test --all-features

watch:
	cargo watch -x test

watch_dummy:
	cargo watch -s "cd dummy && cargo run"

clippy:
	cargo clippy -- -D warnings
