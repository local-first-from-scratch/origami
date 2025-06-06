all: frontend/dist/index.html

Cargo.lock: Cargo.toml store/Cargo.toml
	cargo build

store/pkg/package.json: store/* store/Cargo.toml Cargo.lock
	rm -rf $(@D)
	wasm-pack build store
	find $(@D) | grep -ve package.json -e .gitignore | sed "s|$(@D)||g" > $(@D)/.gitignore

frontend/bun.lock: frontend/package.json store/pkg/package.json
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

.PHONY: dev
dev: frontend/bun.lock
	cd frontend; bun run dev

.PHONY: format
format:
	cd frontend; bun run format
	cargo fmt
