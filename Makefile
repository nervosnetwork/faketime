test: faketime-test system-test

faketime-test:
	cargo test

system-test:
	RUSTDOCFLAGS="--cfg disable_faketime" RUSTFLAGS="--cfg disable_faketime" cargo test

wasm-headless-browser-test:
	wasm-pack test --headless --chrome

doc:
	cargo doc --no-deps

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy

.PHONY: test faketime-test system-test doc
.PHONY: fmt clippy
