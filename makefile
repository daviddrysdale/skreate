all: build manual

WASM_CRATE=skreate_wasm
CLI=target/debug/skreate-cli
DOCGEN=target/debug/skreate-docgen
LIBRARY_SRC=wasm/src/lib.rs $(wildcard lib/src/*.rs) $(wildcard lib/src/moves/*rs) $(wildcard lib/src/parser/*/mod.rs)
EXAMPLES=$(wildcard web/examples/*.skate)
EXAMPLES_SVG=$(EXAMPLES:.skate=.svg)

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

cli: $(CLI)
$(CLI): cli/src/main.rs $(LIBRARY_SRC)
	cargo build --manifest-path cli/Cargo.toml

run-cli: target/debug/skreate-cli
	$<

web/examples/%.svg: web/examples/%.skate $(CLI)
	$(CLI) $< > $@

test:
	cargo test

clippy:
	cargo clippy --all-targets

regenerate: regenerate_examples manual
	SKREATE_REGENERATE=1 cargo test -- test_compare

regenerate_examples: $(EXAMPLES_SVG)

manual: web/doc/manual.html
web/doc/manual.html: doc/manual.hbs $(DOCGEN) $(LIBRARY_SRC)
	rm -f web/doc/*
	$(DOCGEN) --in-file $< --eg-dir web/examples/ --out-dir web/doc/
clean_manual:
	rm -f web/doc/* web/doc/manual.html

$(DOCGEN): doc/src/main.rs $(LIBRARY_SRC)
	cargo build --manifest-path doc/Cargo.toml

publish: clean clean_manual build regenerate publish_build publish_tag
publish_build:
	git diff-index --quiet HEAD --
	rm -rf published
	cp -r web published

published_update:
	rm -rf published
	cp -r web published

publish_tag:
	git tag -f `date "+published-%Y%m%dT%H%M"`

force_publish: published_update publish_tag

web/pkg:
	mkdir -p $@
web/pkg/$(WASM_CRATE).js: target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm | web/pkg
	wasm-bindgen $< --out-dir web/pkg --target web --no-typescript
web/pkg/$(WASM_CRATE)_bg.wasm: target/wasm32-unknown-unknown/release/$(WASM_CRATE).wasm | web/pkg
	wasm-bindgen $< --out-dir web/pkg --target web --no-typescript

clean:
	cargo clean
	rm -f doc/generated/*    # regenerate with `make regenerate`
	rm -f web/doc/*          # regenerate with `make manual`
	rm -f web/examples/*.svg # regenerate with `make regenerate`

distclean:
	rm -rf web/pkg target
