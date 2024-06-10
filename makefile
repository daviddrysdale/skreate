WASM_CRATE=skreate_wasm
all: build

build: web/pkg/$(WASM_CRATE).js

serve: build
	http

open:
	open http://localhost:8000/web

prereqs:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli
	cargo install https # for local webserver

target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm: wasm/src/lib.rs lib/src/*.rs
	cargo build --lib --release --target wasm32-unknown-unknown

cli: cli/src/main.rs
	cargo build --manifest-path cli/Cargo.toml

test:
	cargo test

regenerate:
	SKREATE_REGENERATE=1 cargo test -- test_compare

publish:
	git tag -f `date "+published-%Y%m%dT%H%M"`

web/pkg:
	mkdir -p $@
web/pkg/$(WASM_CRATE).js: target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm | web/pkg
	wasm-bindgen $< --out-dir web/pkg --target web --no-typescript
web/pkg/$(WASM_CRATE)_bg.wasm: target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm | web/pkg
	wasm-bindgen $< --out-dir web/pkg --target web --no-typescript

clean:
	cargo clean

distclean:
	rm -rf web/pkg target
