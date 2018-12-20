test: faketime-test system-test

faketime-test:
	cargo test

system-test:
	RUSTDOCFLAGS="--cfg disable_faketime" RUSTFLAGS="--cfg disable_faketime" cargo test

doc:
	cargo doc --no-deps

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy

.PHONY: test faketime-test system-test doc
.PHONY: fmt clippy
