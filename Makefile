fmt: src/
	cargo fmt -- --check

clippy: src/
	cargo clippy -- -D warnings

check: fmt clippy

run: src/
	cargo run --release --bin main

wasm: src/
	./build-wasm.sh main --release
