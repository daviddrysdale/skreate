CRATE=skreate
all:

build: target/wasm32-unknown-unknown/release/$(CRATE)_wasm.wasm

serve: pkg/$(CRATE)_wasm.js
	http

prereqs:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli
	cargo install https # for local webserver

target/wasm32-unknown-unknown/release/$(CRATE)_wasm.wasm: src/lib.rs
	cargo build --lib --release --target wasm32-unknown-unknown

pkg:
	mkdir -p $@
pkg/$(CRATE)_wasm.js: target/wasm32-unknown-unknown/release/$(CRATE)_wasm.wasm | pkg
	wasm-bindgen $< --out-dir pkg --target web --no-typescript
pkg/$(CRATE)_wasm_bg.wasm: target/wasm32-unknown-unknown/release/$(CRATE)_wasm.wasm | pkg
	wasm-bindgen $< --out-dir pkg --target web --no-typescript
