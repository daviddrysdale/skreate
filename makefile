WASM_CRATE=skreate_wasm
all: build

LIBRARY_SRC=wasm/src/lib.rs lib/src/*.rs lib/src/moves/*rs

build: web/pkg/$(WASM_CRATE).js

serve: build
	http

open:
	open http://localhost:8000/web

prereqs:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli
	cargo install https # for local webserver

target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm: $(LIBRARY_SRC)
	cargo build --lib --release --target wasm32-unknown-unknown

cli: target/debug/skreate-cli
target/debug/skreate-cli: cli/src/main.rs $(LIBRARY_SRC)
	cargo build --manifest-path cli/Cargo.toml

run-cli: target/debug/skreate-cli
	$<

test:
	cargo test

clippy:
	cargo clippy --all-targets

regenerate:
	SKREATE_REGENERATE=1 cargo test -- test_compare

publish: clean build publish_build publish_tag
publish_build:
	git diff-index --quiet HEAD -- && \
	rm -rf published && \
	cp -r web published

publish_tag:
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
