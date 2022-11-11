test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy

build:
	cargo build

client:
	RUST_LOG=info cargo run --bin client

server:
	RUST_LOG=info cargo run --bin server
