#!/usr/bin/env nu

use std assert
use std log
use blend_util expand_package_dir


log info "Running tests for expand_package_dir"
assert equal (expand_package_dir "vscode") "vscode/User"
assert equal (expand_package_dir "sway") "sway/sway/config"

# FIXME: support filtering non git files
assert equal (expand_package_dir "pueue") "pueue/pueue/pueue.yml"

