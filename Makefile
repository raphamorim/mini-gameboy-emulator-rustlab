install:
	cargo install cargo-watch
	cargo install cargo-server
	cargo install wasm-bindgen-cli

desktop:
	cargo run

web-serve:
	cargo server --open

web-watch:
	cargo watch -- make web

web:
	cargo build --target wasm32-unknown-unknown --lib
	wasm-bindgen ./target/wasm32-unknown-unknown/debug/web.wasm --out-dir wasm --target web --no-typescript

lint:
	cargo fmt -- --check --color always
	cargo clippy --all-targets --all-features -- -D warnings
