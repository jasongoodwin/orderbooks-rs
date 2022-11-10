test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy

build:
	cargo build

run:
	RUST_LOG=info cargo run
