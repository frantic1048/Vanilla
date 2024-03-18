#!/usr/bin/env bash
set -euo pipefail

function handle_exit {
  echo "Finished"
}
trap handle_exit EXIT

self_path="$(command -v "$0")"
self_dir="$(dirname "$self_path")"

# FIXME: install Xcode Command Line Tools
# xcode-select --install

# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Add necessary Homebrew environment variables
UNAME_MACHINE="$(uname -m)"
if [[ "${UNAME_MACHINE}" == "arm64" ]]
then
  # On ARM macOS
  eval "$(/opt/homebrew/bin/brew shellenv)"
else
  # On Intel macOS
  eval "$(/usr/local/bin/brew shellenv)"
fi

brew analytics off

# Install Homebrew packages
brew bundle install

# Install proto
# https://moonrepo.dev/docs/proto/install
curl -fsSL https://moonrepo.dev/install/proto.sh | bash -s -- --yes --no-profile

# Install dotfiles
"$self_dir/blend" install

git credential-manager configure

# MEMO: start essential service
# yabai --restart-service
# skhd --restart-service

echo "To configure Raycast"
echo "See https://manual.raycast.com/hotkey"

# MEMO: once
# ./macos_sysctl.sh
