#!/usr/bin/env elvish

# Generate essential environment variables for zsh and bash

use カフェラテ・カフェモカ・カプチーノ/env

each {|env_name|
  if (has-env $env_name) {
    # WARN: add escape for '
    printf "export %s='%s'\n" $env_name (get-env $env_name)
  }
} [
  'PATH'
  'PROTO_HOME'
  'RUSTUP_HOME'
  'CARGO_HOME'
  'NIX_SSL_CERT_FILE'
  'NIX_PROFILES'
  'GOPATH'
  'GOBIN'
  'SSH_AUTH_SOCK'
  'GPG_TTY'
  'VISUAL'
]