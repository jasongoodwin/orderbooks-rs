test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy

build:
	cargo build --release	
	cp ./Settings.toml target/release

client:
	RUST_LOG=info cargo run --bin client

server:
	RUST_LOG=debug cargo run --bin server
