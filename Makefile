document/pkg/package.json: document/src/* document/Cargo.toml Cargo.lock
	wasm-pack build document
