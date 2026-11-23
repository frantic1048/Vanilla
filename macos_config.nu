#!/usr/bin/env nu

# macOS defaults, applied separately from Blend.
# References: https://github.com/mathiasbynens/dotfiles/blob/main/.macos
#
# Each entry is written with `defaults write`; the value type picks the flag
# (bool → -bool, int → -int, string → -string). `host: true` writes with
# `-currentHost`.
const SETTINGS = [
  # MEMO: macOS debug menu
  {domain: NSGlobalDomain key: _NS_4445425547 value: true}

  # Input devices
  # fast keyboard repeat rate
  {domain: NSGlobalDomain key: KeyRepeat value: 1}
  {domain: NSGlobalDomain key: InitialKeyRepeat value: 10}
  # disable "natural" (Lion-style) scrolling
  {domain: NSGlobalDomain key: com.apple.swipescrolldirection value: false}
  # trackpad: tap to click
  {domain: com.apple.driver.AppleBluetoothMultitouch.trackpad key: Clicking value: true}
  {domain: NSGlobalDomain key: com.apple.mouse.tapBehavior value: 1 host: true}

  # Finder
  {domain: com.apple.finder key: ShowStatusBar value: true}
  {domain: com.apple.finder key: ShowPathbar value: true}
  {domain: NSGlobalDomain key: AppleShowAllExtensions value: true}
  # keep folders on top when sorting by name
  {domain: com.apple.finder key: _FXSortFoldersFirst value: true}
  # no warning when changing a file extension
  {domain: com.apple.finder key: FXEnableExtensionChangeWarning value: false}
  # avoid creating .DS_Store files on network or USB volumes
  {domain: com.apple.desktopservices key: DSDontWriteNetworkStores value: true}
  {domain: com.apple.desktopservices key: DSDontWriteUSBStores value: true}

  # Safari: Develop menu and Web Inspector
  {domain: com.apple.Safari key: IncludeDevelopMenu value: true}
  {domain: com.apple.Safari key: WebKitDeveloperExtrasEnabledPreferenceKey value: true}
  {domain: com.apple.Safari key: com.apple.Safari.ContentPageGroupIdentifier.WebKit2DeveloperExtrasEnabled value: true}
  # context menu item for showing the Web Inspector in web views
  {domain: NSGlobalDomain key: WebKitDeveloperExtras value: true}
]

const ACTIVATE_SETTINGS = "/System/Library/PrivateFrameworks/SystemAdministration.framework/Resources/activateSettings"

# Apply macOS defaults. With --dry-run, only compare current and desired values.
def main [--dry-run] {
  if $nu.os-info.name != "macos" {
    error make { msg: "macos_config.nu only runs on macOS" }
  }

  if $dry_run {
    $SETTINGS | each {|s| {
      domain: $s.domain
      key: $s.key
      host: ($s.host? | default false)
      current: (read-default $s)
      desired: (display-value $s.value)
    } } | print
    return
  }

  # Quit System Settings so it does not override the values written below
  if (ps | where name == "System Settings" | is-not-empty) {
    ^osascript -e 'quit app "System Settings"'
  }

  # Keep going past failures: sandboxed domains such as com.apple.Safari
  # reject writes unless the terminal has Full Disk Access
  let failed = $SETTINGS | each {|s|
    let result = ^defaults ...(host-args $s) write $s.domain $s.key (type-flag $s.value) (write-value $s.value) | complete
    if $result.exit_code != 0 {
      {domain: $s.domain key: $s.key error: ($result.stderr | str trim)}
    }
  } | compact

  if ($failed | is-not-empty) {
    print "Some defaults could not be written (sandboxed domains need Full Disk Access for the terminal):"
    print ($failed | table --expand)
  }

  # Apply written defaults without logging out
  ^defaults write com.apple.activatesettings log true
  if ($ACTIVATE_SETTINGS | path exists) {
    ^$ACTIVATE_SETTINGS -u
  } else {
    print "activateSettings not found; log out and back in to apply all settings"
  }
}

def host-args [s: record]: nothing -> list<string> {
  if ($s.host? | default false) { ["-currentHost"] } else { [] }
}

def type-flag [value]: nothing -> string {
  match ($value | describe) {
    "bool" => "-bool"
    "int" => "-int"
    "string" => "-string"
    $t => { error make { msg: $"unsupported defaults value type: ($t)" } }
  }
}

# `defaults write -bool` takes true/false/yes/no (not 1/0)
def write-value [value]: nothing -> string {
  $value | into string
}

# `defaults read` prints bools as 1/0
def display-value [value]: nothing -> string {
  match ($value | describe) {
    "bool" => (if $value { "1" } else { "0" })
    _ => ($value | into string)
  }
}

def read-default [s: record]: nothing -> string {
  let result = ^defaults ...(host-args $s) read $s.domain $s.key | complete
  if $result.exit_code == 0 { $result.stdout | str trim } else { "<unset>" }
}
