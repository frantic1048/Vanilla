#!/usr/bin/env nu

const date_relative = "--date=relative"
const ff = "--ff-only"
const oneline = "--pretty=oneline"

const ref_format = [
  '--format=%(HEAD)'
  '%(color:dim red)%(objectname:short)%(color:reset)'
  '%(color:bold italic brightblue)%(refname:short)%(color:reset)'
  '%(color:dim)-%(color:reset)'
  '%(authorname)'
  '%(color:dim)[%(color:reset)%(color:brightmagenta)%(committerdate:relative)%(color:reset)%(color:dim)]%(color:reset)'
] | str join ' '

export def --wrapped g [...rest: string] {
  if ($rest | length) == 0 {
    g b | tail -n5
    git status -s
  } else {
    git ...$rest
  }
}

# git branch
export def --wrapped 'g b' [...rest: string] {
  if ($rest | length) == 0 {
    git for-each-ref --sort=committerdate --color=always 'refs/heads/' $ref_format
  } else {
    git branch ...$rest
  }
}