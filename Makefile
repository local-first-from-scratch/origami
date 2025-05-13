all: frontend/dist/index.html

Cargo.lock: Cargo.toml document/Cargo.toml
	cargo build

document/pkg/package.json: document/src/* document/Cargo.toml Cargo.lock
	wasm-pack build document

frontend/bun.lock: frontend/package.json
	cd frontend; bun install

frontend/dist/index.html: frontend/src/* frontend/package.json frontend/bun.lock document/pkg/package.json
	cd frontend; bun run build

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy -- -D warnings
	cd frontend; bun run check
