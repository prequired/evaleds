# EvalEds Makefile
# Provides convenient commands for development and installation

.PHONY: help install uninstall build test clean dev docs release

# Default target
help:
	@echo "EvalEds - AI evaluation platform with PromptEds integration"
	@echo ""
	@echo "Available targets:"
	@echo "  install     Install EvalEds locally"
	@echo "  uninstall   Uninstall EvalEds"
	@echo "  build       Build release binary"
	@echo "  test        Run all tests"
	@echo "  clean       Clean build artifacts"
	@echo "  dev         Start development environment"
	@echo "  docs        Generate documentation"
	@echo "  release     Build release packages"
	@echo "  check       Run linting and checks"
	@echo ""
	@echo "Environment variables:"
	@echo "  INSTALL_DIR    Installation directory (default: ~/.local/bin)"
	@echo "  CONFIG_DIR     Configuration directory (default: ~/.config/evaleds)"
	@echo ""

# Installation
install:
	@echo "🚀 Installing EvalEds..."
	@chmod +x install.sh
	@./install.sh

uninstall:
	@echo "🗑️ Uninstalling EvalEds..."
	@chmod +x uninstall.sh
	@./uninstall.sh

# Development
build:
	@echo "🔨 Building EvalEds..."
	cargo build --release

test:
	@echo "🧪 Running tests..."
	cargo test

check:
	@echo "🔍 Running checks..."
	cargo check
	cargo clippy -- -D warnings
	cargo fmt --check

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf target/
	rm -f *.log

dev:
	@echo "🛠️ Starting development environment..."
	cargo run -- --help

# Documentation
docs:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --open

# Release management
release: clean build test check
	@echo "📦 Building release packages..."
	@mkdir -p dist
	
	# Build for current platform
	cargo build --release
	
	# Create tarball
	@echo "Creating release tarball..."
	@tar -czf dist/evaleds-$(shell cargo pkgid | cut -d# -f2)-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz \
		-C target/release evaleds \
		-C ../../ README.md ALIGNMENT_SUMMARY.md \
		-C . install.sh uninstall.sh
	
	@echo "✅ Release package created in dist/"

# Cross-compilation targets (requires cross)
release-all: clean
	@echo "📦 Building release packages for all platforms..."
	@mkdir -p dist
	
	# Linux x86_64
	@if command -v cross >/dev/null 2>&1; then \
		echo "Building for Linux x86_64..."; \
		cross build --release --target x86_64-unknown-linux-gnu; \
		tar -czf dist/evaleds-$(shell cargo pkgid | cut -d# -f2)-linux-x86_64.tar.gz \
			-C target/x86_64-unknown-linux-gnu/release evaleds \
			-C ../../../ README.md ALIGNMENT_SUMMARY.md install.sh uninstall.sh; \
	fi
	
	# macOS x86_64
	@if command -v cross >/dev/null 2>&1; then \
		echo "Building for macOS x86_64..."; \
		cross build --release --target x86_64-apple-darwin; \
		tar -czf dist/evaleds-$(shell cargo pkgid | cut -d# -f2)-macos-x86_64.tar.gz \
			-C target/x86_64-apple-darwin/release evaleds \
			-C ../../../ README.md ALIGNMENT_SUMMARY.md install.sh uninstall.sh; \
	fi
	
	# Windows x86_64
	@if command -v cross >/dev/null 2>&1; then \
		echo "Building for Windows x86_64..."; \
		cross build --release --target x86_64-pc-windows-gnu; \
		zip -j dist/evaleds-$(shell cargo pkgid | cut -d# -f2)-windows-x86_64.zip \
			target/x86_64-pc-windows-gnu/release/evaleds.exe \
			README.md ALIGNMENT_SUMMARY.md scripts/install.ps1 scripts/uninstall.ps1; \
	fi
	
	@echo "✅ All release packages created in dist/"

# Development helpers
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt

lint:
	@echo "🔍 Linting code..."
	cargo clippy -- -D warnings

# Database management
db-reset:
	@echo "🗄️ Resetting database..."
	rm -f ~/.local/share/evaleds/*.db
	rm -f ~/.evaleds/*.db

# Configuration
config-reset:
	@echo "⚙️ Resetting configuration..."
	rm -f ~/.config/evaleds/config.toml
	rm -f ~/.evaleds/config.toml

# Quick development setup
setup-dev:
	@echo "🛠️ Setting up development environment..."
	@rustup component add clippy rustfmt
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch; \
	fi
	@echo "✅ Development environment ready"
	@echo "💡 Run 'make watch' to start file watching"

watch:
	@echo "👀 Watching for changes..."
	cargo watch -x "build" -x "test"

# Benchmarks
bench:
	@echo "🏃 Running benchmarks..."
	cargo bench

# Security audit
audit:
	@echo "🔒 Running security audit..."
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit; \
	fi
	cargo audit

# Update dependencies
update-deps:
	@echo "📦 Updating dependencies..."
	cargo update

# Generate shell completions
completions:
	@echo "🚀 Generating shell completions..."
	@mkdir -p completions
	@cargo run -- --generate-completions bash > completions/evaleds.bash
	@cargo run -- --generate-completions zsh > completions/_evaleds
	@cargo run -- --generate-completions fish > completions/evaleds.fish
	@echo "✅ Shell completions generated in completions/"

# Docker build (if Dockerfile exists)
docker-build:
	@echo "🐳 Building Docker image..."
	docker build -t evaleds:latest .

docker-run:
	@echo "🐳 Running Docker container..."
	docker run -it --rm evaleds:latest

# CI helpers
ci-test: test check audit
	@echo "✅ All CI checks passed"

# Show version info
version:
	@cargo pkgid | cut -d# -f2

# Show detailed help
help-detailed:
	@echo "EvalEds Makefile - Detailed Help"
	@echo "================================"
	@echo ""
	@echo "Development Workflow:"
	@echo "  1. make setup-dev    # Setup development environment"
	@echo "  2. make build        # Build the project"
	@echo "  3. make test         # Run tests"
	@echo "  4. make dev          # Test CLI locally"
	@echo ""
	@echo "Installation:"
	@echo "  make install         # Install locally using install.sh"
	@echo "  make uninstall       # Remove installation"
	@echo ""
	@echo "Release Process:"
	@echo "  make release         # Build single-platform release"
	@echo "  make release-all     # Build cross-platform releases"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean           # Clean build artifacts"
	@echo "  make update-deps     # Update Cargo dependencies"
	@echo "  make audit           # Security audit"
	@echo ""