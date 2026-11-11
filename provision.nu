#!/usr/bin/env nu

# Vanilla provision
# Run by bootstrap.sh once nushell is available. Installs packages, proto with
# blend and the dev toolchains, deploys Orders, registers git credential
# helpers, and prints the remaining manual steps.

# Baseline toolchains for a fresh host. After provisioning, each host pins what
# it needs (`proto install --pin global`) and system-maintenance.nu bumps them.
const PROTO_TOOLS = [rust node npm pnpm deno bun go python uv]

def main [] {
  # The repo root .prototools (plugin registration) applies from here
  cd $env.FILE_PWD

  if $nu.os-info.name == "macos" { setup-homebrew }
  setup-proto
  ^blend --blend-dir $env.FILE_PWD sync
  setup-git-credentials
  print-checklist
}

def setup-homebrew [] {
  ^brew analytics off

  if $nu.os-info.arch == "aarch64" {
    # Rosetta 2 for x86_64 binaries; already-installed or declined is fine
    try { ^softwareupdate --install-rosetta --agree-to-license }
  }

  # System packages: CLI tools, fonts, desktop apps, etc.
  ^brew bundle install --file ($env.FILE_PWD | path join Brewfile)
}

def --env setup-proto [] {
  # Proto is self-managed; use the official installer
  # https://moonrepo.dev/docs/proto/install
  if (which proto | is-empty) {
    http get https://moonrepo.dev/install/proto.sh | ^bash -s -- --yes --no-profile
  }

  $env.PROTO_HOME = $env.HOME | path join .proto
  $env.PATH = $env.PATH | prepend [
    ($env.PROTO_HOME | path join shims)
    ($env.PROTO_HOME | path join bin)
  ]

  # proto only loads plugins from trusted configs; trusting the repo enables
  # the blend plugin registered in its .prototools
  ^proto trust $env.FILE_PWD
  ^proto install blend --pin global

  for tool in $PROTO_TOOLS {
    ^proto install $tool --pin global
  }
}

# Register gh/glab as git credential helpers in ~/.gitconfig.
# ~/.gitconfig is host-specific and not managed by Vanilla. Authentication is
# left to the checklist, since credentials may not be available yet.
# Extra GitLab instances: GLAB_CREDENTIAL_HOSTS="gitlab.example.com ..."
def setup-git-credentials [] {
  if (which gh | is-empty) {
    print "gh not found; skipped GitHub credential helper (run: gh auth setup-git --hostname github.com --force)"
  } else {
    # Also configures gist.github.com; --force allows setup before `gh auth login`
    ^gh auth setup-git --hostname github.com --force
  }

  if (which glab | is-empty) {
    print "glab not found; skipped GitLab credential helper"
    return
  }

  let extra_hosts = $env.GLAB_CREDENTIAL_HOSTS?
    | default ""
    | split row --regex '\s+'
    | compact --empty

  for host in (["gitlab.com"] ++ $extra_hosts) {
    let key = $"credential.https://($host).helper"

    # Exit code 5 means the key did not exist yet
    let unset = ^git config --global --unset-all $key | complete
    if $unset.exit_code not-in [0 5] {
      error make { msg: $"failed to reset ($key): ($unset.stderr | str trim)" }
    }

    # Empty helper first resets inherited helpers (e.g. osxkeychain) for this host
    ^git config --global --add $key ""
    ^git config --global --add $key "!glab auth git-credential"
  }
}

def print-checklist [] {
  mut steps = [
    [title commands];
    ["Change default shell to nushell" [
      "which nu | sudo tee -a /etc/shells"
      "chsh -s \"$(which nu)\""
    ]]
    ["Authenticate git hosting CLIs (credential helpers are already registered)" [
      "gh auth login"
      "glab auth login [--hostname <gitlab-host>]"
    ]]
  ]

  if $nu.os-info.name == "macos" {
    $steps = $steps | append [
      [title commands];
      ["Apply macOS system preferences" ["./macos_config.sh"]]
      ["Configure Raycast (https://manual.raycast.com/hotkey)" [
        "Optionally disable Spotlight indexing: sudo mdutil -a -i off"
      ]]
    ]
  }

  print ""
  print "=== Provision complete ==="
  print ""
  print "Remaining manual steps:"
  print ""
  for step in ($steps | enumerate) {
    print $"  ($step.index + 1). ($step.item.title):"
    for cmd in $step.item.commands { print $"     ($cmd)" }
    print ""
  }
  print "Restart your shell to pick up the new configuration."
}
