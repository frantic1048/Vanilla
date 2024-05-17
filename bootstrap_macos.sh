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

# Install Google Chrome
curl -fsSL https://dl.google.com/chrome/mac/universal/stable/GGRO/googlechrome.dmg -o /tmp/googlechrome.dmg
hdiutil attach /tmp/googlechrome.dmg
cp -R '/Volumes/Google Chrome/Google Chrome.app' /Applications/
hdiutil detach '/Volumes/Google Chrome'
rm -vf /tmp/googlechrome.dmg

# TODO: configure zsh PATH to include Homebrew binaries path

# TODO: install gpg(GnuPG for OS X)
# https://sourceforge.net/p/gpgosx/docu/Download/

# MEMO: VSCode settings sync

# Install dotfiles
"$self_dir/blend"

# MEMO: Change default shell to elvish
which elvish | sudo tee -a /etc/shells
chsh -s "$(which elvish)"

# TODO: Configure git for ~/work
# Copy config.user.work.example to config.user.work and edit it

git credential-manager configure

# TODO: Pointer size: very large
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

# TODO: better place for these config

# FIXME: not working on macOS 14.4.1 (Sonoma)
# defaults write -g NSWindowShouldDragOnGesture -bool true

# TODO: chmod +i shell rc files to prevent external modification

# MEMO: macOS debug menu
# defaults write -g _NS_4445425547 -bool true