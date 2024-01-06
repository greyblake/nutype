all: fmt test-all clippy examples typos

test-all:
	cargo test
	cargo test --features serde
	cargo test --features regex
	cargo test --features new_unchecked
	cargo test --features schemars08
	cargo test --all-features

test:
	cargo test

test-ui:
	cargo test --features ui

fmt:
  cargo fmt

watch:
	cargo watch -x test

watch-dummy:
	cargo watch -s "cd dummy && cargo run"

clippy:
	cargo clippy -- -D warnings

examples:
  for example in `ls examples`; do cargo run --bin $example; done

typos:
    which typos >/dev/null || cargo install typos-cli
    typos
