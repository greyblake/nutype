all: test-all clippy

test-all:
	cargo test --features nutype_test
	cargo test --features nutype_test,serde
	cargo test --features nutype_test,regex
	cargo test --features nutype_test,new_unchecked
	cargo test --features nutype_test,schemars08
	cargo test --all-features

test:
	cargo test --features nutype_test

test-ui:
	cargo test --features nutype_test,ui

watch:
	cargo watch -x test

watch-dummy:
	cargo watch -s "cd dummy && cargo run"

clippy:
	cargo clippy -- -D warnings
