default:
    @just --list

# Validate all orders
check:
    blend --blend-dir . check

# Deploy all configs
deploy:
    blend --blend-dir . sync

# Interactive sync
sync *ARGS:
    blend --blend-dir . sync {{ ARGS }}

# Regenerate README.md from the current Order table
readme:
    nu README.md.nu

# Run the local system maintenance routine
s:
    nu --no-config-file bin/system-maintenance.nu

# Compatibility alias for the old daily routine recipe
upgrade:
    just s

# Full bootstrap (called by bootstrap.sh after dependencies are installed)
bootstrap:
    blend --blend-dir . sync
    @echo "Bootstrap complete. Restart your shell."
