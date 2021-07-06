fmt: src/
	cargo fmt -- --check

clippy: src/
	cargo clippy -- -D warnings

check: fmt clippy

run: src/ assets/
	cargo run --release --bin main

wasm: src/ assets/
	rm -rf ./web
	./build_wasm.sh main --release
	cp -a ./assets/ ./web
