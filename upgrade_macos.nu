#!/usr/bin/env nu

let self_dir = ($env.FILE_PWD)

do {
  cd $self_dir
  brew bundle install
  proto upgrade
  rye self update
  ./blend install
}
