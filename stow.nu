#!/usr/bin/env nu

const not_stow_list = [
  "darwin-system" # nix-darwin
  "root" # linux system config, not managed by stow
  "screenshots" # screenshots
]

let all_stow_list: list<string> = (git ls-tree --name-only --full-name -d HEAD :/ 
  | lines
  | where not $it in $not_stow_list
)

let home_path = $nu.home_path
let macos_application_path = path join $home_path "Library" "Application Support"
let xdg_config_home_path = path join $home_path ".config"

const macos_stow_list: list<string> = [
  "kitty"
]
const linux_stow_list: list<string> = []

const stow_list_table: table<name: string, list: list<string>> = [
  [name list];
  [macOS-x86_64 $macos_stow_list]
  [linux-x86_64 $linux_stow_list]
]

def get_stow_list [] {
  let os_name = $nu | get os-info.name
  let os_arch = $nu | get os-info.arch
  echo $"profile: ($os_name)-($os_arch)"
}

def assert_stow_list_only_contains_known_items [stow_list: list<string>] {
  let unknown_items = $stow_list | where not $it in $all_stow_list
  if ($unknown_items | length) > 0 {
    echo $"Unknown items in stow list: ($unknown_items)"
    exit 1
  }
}

def verify [] {
  echo "Verifying stow list..."
  $stow_list_table | each {|row|
    echo $"Verifying stow list for ($row.name)..."
    assert_stow_list_only_contains_known_items $row.list
  }
  echo "Stow list verified."
}

def init [] {
  echo $"All stow list: ($all_stow_list)"
  verify
  get_stow_list
}

def install [] {

}

init