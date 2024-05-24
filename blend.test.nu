#!/usr/bin/env nu

use std assert
use std log
use blend_util [expand_package_dir, is_git_ignored]


log info "Running tests for expand_package_dir"
assert equal (expand_package_dir "vscode") "packages/vscode/User" "expand to directory when multiple files are present"
assert equal (expand_package_dir "sway") "packages/sway/sway/config" "expand to file when only one file is present"
assert equal (expand_package_dir "pueue") "packages/pueue/pueue/pueue.yml" "support filtering non git files"

log info "Running tests for is_git_ignored"
assert equal (is_git_ignored ".DS_Store") true "ignore .DS_Store"
assert equal (is_git_ignored "packages/pueue/pueue/pueue.yml") false "don't ignore pueue.yml"