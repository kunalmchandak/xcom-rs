.PHONY: build test fmt lint clean install run help

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the project
	cargo build

release: ## Build the project in release mode
	cargo build --release

test: ## Run tests
	cargo test

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt -- --check

lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## Clean build artifacts
	cargo clean

install: ## Install pre-commit hooks
	pre-commit install

run: ## Run the application
	cargo run

check: fmt-check lint test ## Run all checks (format, lint, test)

pre-commit: ## Run pre-commit hooks manually
	pre-commit run --all-files
