#!/usr/bin/env bash
set -euo pipefail

function handle_exit {
  echo "Finished"
}
trap handle_exit EXIT

self_path="$(command -v "$0")"
self_dir="$(dirname "$self_path")"

# Install pacman packages

# MEMO:
# At this point, we have nushell and elvish.
# We could move following steps to nushell or elvish scripts
# with better error handling.

# Install dotfiles
"$self_dir/blend"

# Install proto
# https://moonrepo.dev/docs/proto/install
curl -fsSL https://moonrepo.dev/install/proto.sh | bash -s -- --yes --no-profile

# Temporary make proto available in the current shell
export PATH="$HOME/.proto/bin:$PATH"
# Generate shims based on config from ~/.proto/.prototools
proto regen

# Install Rye
# https://rye.astral.sh/
curl -sSf https://rye.astral.sh/get | RYE_INSTALL_OPTION="--yes" bash

# MEMO: VSCode settings sync

# MEMO: Change default shell to elvish
# which elvish | sudo tee -a /etc/shells
chsh -s "$(which elvish)"

# TODO: Configure git for ~/work
# Copy config.user.work.example to config.user.work and edit it

git credential-manager configure

# TODO: chmod +i shell rc files to prevent external modification
