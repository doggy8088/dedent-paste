.PHONY: build build-locked release test test-rust test-npm fmt fmt-check check ci install clean

# Debug build
build:
	cargo build

# Optimized release build
release:
	cargo build --release --locked

# Run all tests (Rust unit tests + npm postinstall tests)
test: test-rust test-npm

test-rust:
	cargo test --locked

test-npm:
	npm test

# Format code in place
fmt:
	cargo fmt

# Verify formatting without modifying files
fmt-check:
	cargo fmt --check

# Type-check without producing binaries
check:
	cargo check --locked

# Everything CI runs
ci: fmt-check test build-locked

build-locked:
	cargo build --locked

# Build from source and configure Karabiner-Elements (macOS)
install:
	./install.sh

clean:
	cargo clean
