# PTY Terminal Makefile
# Provides convenient targets for building, testing, and packaging

.PHONY: help build release test clean fmt clippy install-deps dev installer package all

# Default target
help:
	@echo "PTY Terminal Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  help          Show this help message"
	@echo "  build         Build in debug mode"
	@echo "  release       Build in release mode"
	@echo "  test          Run all tests"
	@echo "  clean         Clean build artifacts"
	@echo "  fmt           Format code"
	@echo "  clippy        Run clippy lints"
	@echo "  install-deps  Install required dependencies"
	@echo "  dev           Quick development build and test"
	@echo "  installer     Create installer for current platform"
	@echo "  package       Create release package"
	@echo "  all           Build everything (release + installer)"
	@echo ""
	@echo "Advanced targets:"
	@echo "  build-macos   Build for macOS (all architectures)"
	@echo "  build-linux   Build for Linux (all architectures)"
	@echo "  build-windows Build for Windows (all architectures)"
	@echo "  build-all     Build for all platforms"
	@echo ""
	@echo "Environment variables:"
	@echo "  TARGET        Specify target triple"
	@echo "  VERBOSE       Enable verbose output (1)"
	@echo "  SIGN          Enable code signing (1)"

# Development workflow
dev: fmt clippy test build
	@echo "Development build complete"

# Basic build targets
build:
	@./scripts/build-local.sh

release:
	@./scripts/build-local.sh --release

test:
	@cargo test

clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf dist/
	@rm -rf target/installers/
	@rm -rf target/local-build/

# Code quality
fmt:
	@echo "Formatting code..."
	@cargo fmt

clippy:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings

# Dependency management
install-deps:
	@echo "Installing dependencies..."
	@rustup component add rustfmt clippy
	@if command -v apt-get >/dev/null 2>&1; then \
		sudo apt-get update && sudo apt-get install -y \
			build-essential pkg-config libssl-dev \
			libgtk-3-dev libayatana-appindicator3-dev \
			librsvg2-dev libwebkit2gtk-4.0-dev \
			libfontconfig1-dev libfreetype6-dev \
			libasound2-dev libudev-dev; \
	fi
	@if command -v brew >/dev/null 2>&1; then \
		brew install create-dmg; \
	fi

# Installation targets
installer:
	@./scripts/build-local.sh --release --installer

package: release installer
	@echo "Creating release package..."
	@mkdir -p dist/packages
	@if [ -f "dist/PTY_Terminal.dmg" ]; then \
		cp dist/PTY_Terminal.dmg dist/packages/; \
	fi
	@if [ -f "dist/PTY_Terminal.AppImage" ]; then \
		cp dist/PTY_Terminal.AppImage dist/packages/; \
	fi
	@if [ -f "dist/"*.deb ]; then \
		cp dist/*.deb dist/packages/; \
	fi
	@if [ -f "dist/"*.msi ]; then \
		cp dist/*.msi dist/packages/; \
	fi

all: release installer
	@echo "Complete build finished"

# Cross-compilation targets
build-macos:
	@echo "Building for macOS..."
	@./scripts/build-local.sh --release --target aarch64-apple-darwin
	@./scripts/build-local.sh --release --target x86_64-apple-darwin

build-linux:
	@echo "Building for Linux..."
	@./scripts/build-local.sh --release --target x86_64-unknown-linux-gnu
	@./scripts/build-local.sh --release --target aarch64-unknown-linux-gnu

build-windows:
	@echo "Building for Windows..."
	@./scripts/build-local.sh --release --target x86_64-pc-windows-msvc
	@./scripts/build-local.sh --release --target aarch64-pc-windows-msvc

build-all: build-macos build-linux build-windows
	@echo "All platform builds complete"

# Advanced build using build-installers.rs
build-installers-current:
	@cargo run --bin build-installers

build-installers-all:
	@cargo run --bin build-installers -- --all-platforms

# Docker builds (if Docker is available)
docker-build:
	@if command -v docker >/dev/null 2>&1; then \
		echo "Building in Docker container..."; \
		docker build -t pty-terminal-builder .; \
		docker run --rm -v $(PWD)/dist:/app/dist pty-terminal-builder; \
	else \
		echo "Docker not found, skipping container build"; \
	fi

# Benchmarks
bench:
	@cargo bench

# Documentation
docs:
	@cargo doc --open

# Security audit
audit:
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "cargo-audit not installed. Install with: cargo install cargo-audit"; \
	fi

# Check for updates
update-deps:
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		cargo outdated; \
	else \
		echo "cargo-outdated not installed. Install with: cargo install cargo-outdated"; \
	fi

# Local server for testing
serve:
	@echo "Starting local test server..."
	@cd dist && python3 -m http.server 8080 || python -m SimpleHTTPServer 8080

# Performance profiling
profile:
	@cargo build --release
	@echo "Run with: cargo run --release"
	@echo "For profiling, use tools like perf, Instruments, or flamegraph"

# GitHub release preparation
prepare-release:
	@echo "Preparing release..."
	@./scripts/build-local.sh --release --installer --sign
	@echo "Release artifacts ready in dist/"

# CI simulation
ci-test:
	@echo "Running CI-like tests..."
	@make fmt
	@make clippy  
	@make test
	@make audit
	@make build
	@echo "CI tests passed"

# Quick fixes
fix:
	@echo "Applying automatic fixes..."
	@cargo fmt
	@cargo clippy --fix --allow-dirty --allow-staged

# Platform detection
detect-platform:
	@echo "Detected platform: $$(./scripts/build-local.sh --help | head -1)"
	@rustc -vV | grep host

# Version information
version:
	@echo "PTY Terminal version: $$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"