#!/usr/bin/env bash
set -euo pipefail

# Vanilla bootstrap — Stage 1
# Installs system dependencies, then hands off to `just bootstrap` for blend build + config deploy.

self_dir="$(cd "$(dirname "$0")" && pwd)"
cd "$self_dir"

os="$(uname -s)"

# ─── macOS ──────────────────────────────────────────────────────────────────────

bootstrap_darwin() {
  # Xcode CLT is a prerequisite (needed for git to clone this repo)
  if ! xcode-select -p &>/dev/null; then
    echo "Please install Xcode Command Line Tools first:"
    echo "  xcode-select --install"
    exit 1
  fi

  # Install Homebrew
  if ! command -v brew &>/dev/null; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  fi

  # Set up Homebrew environment
  if [[ "$(uname -m)" == "arm64" ]]; then
    eval "$(/opt/homebrew/bin/brew shellenv)"
    # Rosetta 2 for x86_64 binaries
    softwareupdate --install-rosetta --agree-to-license || true
  else
    eval "$(/usr/local/bin/brew shellenv)"
  fi

  brew analytics off

  # Install system packages (just, elvish, CLI tools, fonts, desktop apps, etc.)
  brew bundle install
}

# ─── Arch Linux ─────────────────────────────────────────────────────────────────

bootstrap_linux() {
  if ! command -v paru &>/dev/null; then
    echo "Please install an AUR helper (paru) first."
    exit 1
  fi

  local aur_helper
  aur_helper="$(command -v paru)"

  # Minimal packages needed for Stage 2 (Rust comes from proto, not pacman)
  "$aur_helper" -S --needed just elvish
}

# ─── Stage 1: System dependencies ──────────────────────────────────────────────

case "$os" in
  Darwin) bootstrap_darwin ;;
  Linux)  bootstrap_linux ;;
  *)
    echo "Unsupported OS: $os"
    exit 1
    ;;
esac

# ─── Install proto & dev toolchains ─────────────────────────────────────────────

# Proto is self-managed — use the official installer
# https://moonrepo.dev/docs/proto/install
if ! command -v proto &>/dev/null; then
  curl -fsSL https://moonrepo.dev/install/proto.sh | bash -s -- --yes --no-profile
fi

export PROTO_HOME="$HOME/.proto"
export PATH="$PROTO_HOME/shims:$PROTO_HOME/bin:$PATH"

# Install essential toolchains and generate shims
proto install rust node npm pnpm deno bun go python uv

# ─── Stage 2: Build & deploy via just ───────────────────────────────────────────

just bootstrap

# ─── Post-bootstrap checklist ───────────────────────────────────────────────────

echo ""
echo "=== Bootstrap complete ==="
echo ""
echo "Remaining manual steps:"
echo ""
echo "  1. Change default shell to elvish:"
echo "     which elvish | sudo tee -a /etc/shells"
echo "     chsh -s \"\$(which elvish)\""
echo ""
echo "  2. Configure git credentials:"
echo "     git credential-manager configure"
echo ""
if [[ "$os" == "Darwin" ]]; then
  echo "  3. Apply macOS system preferences:"
  echo "     ./macos_config.sh"
  echo ""
  echo "  4. Configure Raycast (https://manual.raycast.com/hotkey)"
  echo "     Optionally disable Spotlight indexing: sudo mdutil -a -i off"
  echo ""
fi
echo "Restart your shell to pick up the new configuration."
