.PHONY: build build-debug build-locked release test test-rust test-npm fmt fmt-check check ci install clean

# Optimized release build (default)
build:
	cargo build --release --locked

# Debug build
build-debug:
	cargo build

# Alias for `build`
release: build

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
