#!/usr/bin/env nu

use std assert
use blend_util [expand_package_dir, is_git_ignored, log]

cd $env.FILE_PWD

def describe (title, block) {
    log info -i $"($title)"
    do $block
}
def it (title, block) {
    log info $"  ($title)"
    do $block
}

describe "expand_package_dir()" {
    it "Expand to directory when multiple files are present" {
        assert equal (expand_package_dir "vscode") "packages/vscode/User"
    }
    it "Expand to file when only one file is present" {
        assert equal (expand_package_dir "sway") "packages/sway/sway/config"
    }
    it "Support filtering non git files" {
        assert equal (expand_package_dir "pueue") "packages/pueue/pueue/pueue.yml"
    }
}

describe "is_git_ignored()" {
    it "Ignore .DS_Store" {
        assert equal (is_git_ignored ".DS_Store") true
    }
    it "Don't ignore pueue.yml" {
        assert equal (is_git_ignored "packages/pueue/pueue/pueue.yml") false
    }
}

describe "blend readme" {
    it "Benchmark" {
        hyperfine --warmup 3 $"./blend readme"
    }
}
