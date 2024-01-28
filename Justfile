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
    #!/usr/bin/env bash
    set -euxo pipefail
    ROOT_DIR=$(pwd)
    for EXAMPLE in `ls examples`; do
        cd $ROOT_DIR/examples/$EXAMPLE;
        if [[ "$EXAMPLE" == "no_std_example" ]]
        then
            cargo build
        else
            cargo run
        fi
    done

typos:
    which typos >/dev/null || cargo install typos-cli
    typos
