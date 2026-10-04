.DEFAULT_GOAL := help
.PHONY: help build build-debug build-locked release test test-rust test-npm fmt fmt-check check ci install clean

# Every target with a trailing `## description` comment is listed by `make help`.
help: ## Show this help (default target)
	@echo "Usage: make <target>"
	@echo
	@awk 'BEGIN { FS = ":.*## " } /^[a-zA-Z_-]+:.*## / { printf "  %-14s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

build: ## Optimized release build (cargo build --release --locked)
	cargo build --release --locked

build-debug: ## Debug build
	cargo build

release: build ## Alias for `build`

test: test-rust test-npm ## Run all tests (Rust unit tests + npm postinstall tests)

test-rust: ## Run Rust tests
	cargo test --locked

test-npm: ## Run npm postinstall tests
	npm test

fmt: ## Format code in place
	cargo fmt

fmt-check: ## Verify formatting without modifying files
	cargo fmt --check

check: ## Type-check without producing binaries
	cargo check --locked

ci: fmt-check test build-locked ## Everything CI runs

build-locked:
	cargo build --locked

install: ## Build from source and configure Karabiner-Elements (macOS)
	./install.sh

clean: ## Remove build artifacts
	cargo clean
