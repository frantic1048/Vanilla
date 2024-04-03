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

# FIXME: Install Google Chrome
# https://dl.google.com/chrome/mac/universal/stable/GGRO/googlechrome.dmg

# FIXME: configure zsh PATH to include Homebrew binaries path

# FIXME: install gpg(GnuPG for OS X)
# https://sourceforge.net/p/gpgosx/docu/Download/

# FIXME: VSCode settings sync

# TODO: Install ZSA keymapp
# https://www.zsa.io/flash

# Install dotfiles
"$self_dir/blend" install

# FIXME: Change default shell to elvish
# which elvish | sudo tee -a /etc/shells
# chsh -s "$(which elvish)"

# FIXME: Configure git for ~/work
# Copy config.user.work.example to config.user.work and edit it

git credential-manager configure

# TODO: Ponter size: very large
# System Preferences -> Accessibility -> Display -> Pointer

# MEMO: start essential service
# prepare for yabai and skhd
# https://github.com/koekeishiya/yabai/wiki/Disabling-System-Integrity-Protection
# yabai --start-service
# skhd --start-service

echo "To configure Raycast"
echo "See https://manual.raycast.com/hotkey"

# MEMO: once
# ./macos_sysctl.sh
