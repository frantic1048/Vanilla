#!/usr/bin/env nu
use std log

const not_stow_list = [
  "darwin-system" # nix-darwin
  "root" # linux system config, not managed by stow
  "screenshots" # screenshots
]

let all_stow_list: list<string> = (git ls-tree --name-only --full-name -d HEAD :/
  | lines
  | where not $it in $not_stow_list
)

let home_path = $nu.home-path
let macos_application_path = [$home_path "Library" "Application Support"] | path join
let xdg_config_home_path = [$home_path ".config"] | path join
def make_xdg_config_subpath [subpath: string] {
  [$xdg_config_home_path $subpath] | path join
}
let stow_dir = $env.FILE_PWD

let macos_stow_table: table<package: string, prefix: string> = [
  [package prefix];
  ["git" $xdg_config_home_path]
  ["yabai" $xdg_config_home_path]
  ["skhd" $xdg_config_home_path]
]
let linux_stow_table: table<package: string, prefix: string> = [
  [package prefix];
  ["git" $xdg_config_home_path]
]

let stow_profiles: table<name: string, table: table<package: string, prefix: string>> = [
  [name table];
  [macos-x86_64 $macos_stow_table]
  [linux-x86_64 $linux_stow_table]
]

def assert_stow_profile_exist [profile: string] {
  if not $profile in ($stow_profiles | get name) {
    log error $"Error: profile ($profile) does not exist."
    exit 1
  }
}

def get_stow_profile [] -> table<name: string, table: table<package: string, prefix: string>> {
  log info "Getting stow profile..."
  let os_name = $nu | get os-info.name
  let os_arch = $nu | get os-info.arch
  let profile_name = $"($os_name)-($os_arch)"
  log info $"Expected profile: ($profile_name)"
  assert_stow_profile_exist $profile_name
  $stow_profiles | where name == $profile_name | first
}

def assert_stow_profile_only_contains_known_items [profile: table<name: string, table: table<package: string, prefix: string>>] {
  let unknown_items = $profile | get table | where not $it.package in $all_stow_list
  if ($unknown_items | length) > 0 {
    log error $"Error: in profile ($profile.name) found unknown items: ($unknown_items)"
    exit 1
  }
}

def verify [] {
  log info "Verifying stow list..."
  $stow_profiles | each {|row|
    log info $"Verifying stow list for ($row.name)..."
    assert_stow_profile_only_contains_known_items $row
  }
  log info "Stow list verified."
}

def mk_stow_rc [] {

}

def provision_essential_paths [] {
  log info "Provisioning essential paths..."
  let paths = [$xdg_config_home_path]
  $paths | each {|it|
    if not ("dir" == ($it | path type)) {
      log info $"Creating path ($it)..."
      mkdir $it
    }
  }
  log info $"Provisioned essential paths."
}

def init [] {
  provision_essential_paths
  log info $"All stow list: ($all_stow_list)"
  verify
  log info "Initializing stow..."
  let profile = (get_stow_profile)
  log info $"Using profile: ($profile.name)"
  $profile.table | each {|row|
    log info $"Stowing ($row.package) to ($row.prefix)"
    log info $"stow --dir ($stow_dir) --target ($row.prefix) -S ($row.package)"
    stow --dir ($stow_dir) --target ($row.prefix) -S ($row.package)
  }
}

def install [] {

}

# Install package configs
def "main install" [] {
  init
}

# init stow
def "main init" [] {
  init
}

def main [] {
}