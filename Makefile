desktop:
	cargo run -p gb-desktop

web:
	cargo run -p gb-web

lint:
	cargo fmt -- --check --color always
	cargo clippy --all-targets --all-features -- -D warnings