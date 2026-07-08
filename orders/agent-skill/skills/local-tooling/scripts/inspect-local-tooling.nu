#!/usr/bin/env nu

let tools = [
  rg
  fd
  blend
  proto
  cargo
  rustc
  node
  npm
  pnpm
  deno
  bun
  uv
  python
  gh
  glab
  twg
  pup
  sentry
  nu
  elvish
  bash
  zsh
  sh
]

let tool_rows = ($tools | each {|tool|
  let resolved = (which $tool | first | default null)
  {
    tool: $tool
    path: (if $resolved == null { null } else { $resolved.path })
  }
})

let proto_file = ($nu.home-dir | path join ".proto" ".prototools")
let vanilla_proto_file = ($nu.home-dir | path join "Vanilla" "orders" "proto" ".proto" ".prototools")
let shellenv = ($nu.home-dir | path join ".local" "bin" "van" "shellenv")
let shellenv_nu = ($nu.home-dir | path join ".local" "bin" "van" "shellenv.nu")
let vanilla_bin = ($nu.home-dir | path join "Vanilla" "bin")
let vanilla_bin_source = ($nu.home-dir | path join "Vanilla" "orders" "bin" "bin")

{
  os: $nu.os-info
  vanilla: {
    path: ($nu.home-dir | path join "Vanilla")
    exists: (($nu.home-dir | path join "Vanilla") | path exists)
    bin_entrypoint: $vanilla_bin
    bin_entrypoint_exists: ($vanilla_bin | path exists)
    bin_source: $vanilla_bin_source
    bin_source_exists: ($vanilla_bin_source | path exists)
  }
  shellenv: {
    wrapper: $shellenv
    wrapper_exists: ($shellenv | path exists)
    nushell_module: $shellenv_nu
    nushell_module_exists: ($shellenv_nu | path exists)
  }
  proto: {
    home_file: $proto_file
    home_file_exists: ($proto_file | path exists)
    vanilla_file: $vanilla_proto_file
    vanilla_file_exists: ($vanilla_proto_file | path exists)
  }
  tools: $tool_rows
}
