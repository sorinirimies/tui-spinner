# tui-spinner — justfile
# Install just: cargo install just
# Usage: just <task>

default:
    @just --list

# Build the project
build:
    cargo build

# Build release version
build-release:
    cargo build --release

# Run the main spinner demo example
run:
    cargo run --example spinner

# Run tests
test:
    cargo test

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt --check

# Lint
clippy:
    cargo clippy -- -D warnings

# Run all checks
check-all: fmt-check clippy test
    @echo "✅ All checks passed!"

# Generate documentation (opens in browser)
doc:
    cargo doc --no-deps --open

# Clean build artifacts
clean:
    cargo clean

# Show current version
version:
    @grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'

# Publish (dry run)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# Update dependencies
update:
    cargo update

# Git: push to GitHub
push:
    git push origin main

# Git: push tags
push-tags:
    git push origin --tags

# Bump version and tag (usage: just bump 0.2.0)
bump version: check-all
    @sed -i 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    @git add Cargo.toml
    @git commit -m "chore: bump version to {{version}}"
    @git tag v{{version}}
    @echo "✅ Bumped to v{{version}}"
