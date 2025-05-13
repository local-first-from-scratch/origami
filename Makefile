document/pkg/package.json: document/src/* document/Cargo.toml Cargo.lock
	wasm-pack build document

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy -- -D warnings
	cd frontend; bun run check
