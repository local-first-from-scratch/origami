all: frontend/dist/index.html

Cargo.lock: Cargo.toml store/Cargo.toml
	cargo build

store/pkg/package.json: store/src/* store/Cargo.toml Cargo.lock
	wasm-pack build store

frontend/bun.lock: frontend/package.json
	cd frontend; bun install

frontend/dist/index.html: frontend/src/* frontend/package.json frontend/bun.lock store/pkg/package.json
	cd frontend; bun run build

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy -- -D warnings
	cd frontend; bun run check

.PHONY: clean
clean:
	rm -rf frontend/dist store/pkg target
