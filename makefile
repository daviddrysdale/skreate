WASM_CRATE=skreate_wasm
all: build

build: pkg/$(WASM_CRATE).js

serve: build
	http

prereqs:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli
	cargo install https # for local webserver

target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm: wasm/src/lib.rs
	cargo build --lib --release --target wasm32-unknown-unknown

pkg:
	mkdir -p $@
pkg/$(WASM_CRATE).js: target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm | pkg
	wasm-bindgen $< --out-dir pkg --target web --no-typescript
pkg/$(WASM_CRATE)_bg.wasm: target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm | pkg
	wasm-bindgen $< --out-dir pkg --target web --no-typescript

clean:
	cargo clean

distclean:
	rm -rf pkg target
