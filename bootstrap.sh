#!/usr/bin/env bash
set -euo pipefail

# Vanilla bootstrap
# Installs the minimal runtime this repository needs (nushell), then hands off
# to provision.nu for the full machine provisioning.

self_dir="$(cd "$(dirname "$0")" && pwd)"

case "$(uname -s)" in
  Darwin)
    # Xcode CLT is a prerequisite (needed for git to clone this repo)
    if ! xcode-select -p &>/dev/null; then
      echo "Please install Xcode Command Line Tools first:"
      echo "  xcode-select --install"
      exit 1
    fi

    if ! command -v brew &>/dev/null; then
      /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    if [[ "$(uname -m)" == "arm64" ]]; then
      eval "$(/opt/homebrew/bin/brew shellenv)"
    else
      eval "$(/usr/local/bin/brew shellenv)"
    fi

    brew install nushell
    ;;
  Linux)
    # Arch Linux; nushell is in the official repositories
    sudo pacman -S --needed nushell
    ;;
  *)
    echo "Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

exec nu --no-config-file "$self_dir/provision.nu"
