# =============================================================================
# justfile - Daily development workflow
# =============================================================================
#
# ascii-term - ASCII art terminal video player
#
# =============================================================================

# Update local main branch
new:
    git switch main && git pull --ff-only

# === Setup ===

# Install system dependencies (FFmpeg, OpenCV, LLVM)
setup:
    cargo x setup

# === Build Commands ===

# Build workspace in debug mode
build:
    cargo build --workspace

# Build ascii-term in release mode (optimized)
build-release:
    cargo build -p ascii-term --release

# === Run Commands ===

# Run ascii-term (pass arguments with `-- <args>`, e.g. `just run -- video.mp4`)
run *args:
    cargo run -p ascii-term -- {{args}}

# Run ascii-term with debug logging
dev *args:
    RUST_LOG=debug cargo run -p ascii-term -- {{args}}

# Run ascii-term in release mode
release *args:
    cargo run -p ascii-term --release -- {{args}}

# === Code Quality ===

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Quick check (format + clippy)
check:
    cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings

# === Testing ===

# Run all tests for all crates
test:
    cargo test --workspace

# Run unit tests: all crates / specific crate / specific test in crate
# Examples:
#   just unit-test                          # All unit tests
#   just unit-test codec               # All unit tests in codec
#   just unit-test codec test_decode   # Specific test
unit-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --lib
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --lib
    else
        cargo test -p {{crate}} --lib {{test}}
    fi

# Run integration tests: all crates / specific crate / specific test in crate
# Examples:
#   just integration-test                          # All integration tests
#   just integration-test codec               # All integration tests in codec
#   just integration-test codec test_pipeline # Specific test
integration-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --tests
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --tests
    else
        cargo test -p {{crate}} --tests {{test}}
    fi

# Run tests sequentially (saves memory)
test-seq:
    cargo test --workspace -- --test-threads=1

# === Clean ===

# Clean build artifacts
clean:
    cargo clean
