#!/usr/bin/env nu


const date_relative = "--date=relative"
const ff = "--ff-only"
const oneline = "--pretty=oneline"

const ref_format = ''
  + ' --format=%(HEAD)'
  + ' %(color:dim red)%(objectname:short)%(color:reset)'
  + ' %(color:bold italic brightblue)%(refname:short)%(color:reset)'
  + ' %(color:dim)-%(color:reset)'
  + ' %(authorname)'
  + ' %(color:dim)[%(color:reset)%(color:brightmagenta)%(committerdate:relative)%(color:reset)%(color:dim)]%(color:reset)'


export def main [] {
  git status -u
}