default:
    @just --list

# Build all executables (release by default)
build: build-blend

# Build blend and symlink into bin/
build-blend:
    cd blend && cargo build --release
    ln -sf ../target/release/blend bin/blend

# Build blend in debug mode (for development)
build-debug:
    cd blend && cargo build
    ln -sf ../target/debug/blend bin/blend-debug

# Validate all orders
check:
    bin/blend view --dry-run

# Run rustfmt on the blend crate
fmt:
    cd blend && cargo fmt

# Check formatting without modifying files (CI-equivalent)
fmt-check:
    cd blend && cargo fmt --check

# Run clippy on the blend crate (CI-equivalent)
clippy:
    cd blend && cargo clippy -- -D warnings

# Run the blend test suite
test:
    cd blend && cargo test --release

# Deploy all configs
deploy:
    bin/blend sync

# Interactive sync
sync *ARGS:
    bin/blend sync {{ ARGS }}

# Run the local system maintenance routine
s:
    nu --no-config-file bin/system-maintenance.nu

# Compatibility alias for the old daily routine recipe
upgrade:
    just s

# Full bootstrap (called by bootstrap.sh after deps are installed)
bootstrap:
    just build
    just deploy
    @echo "Bootstrap complete. Restart your shell."
