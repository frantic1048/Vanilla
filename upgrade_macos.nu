#!/usr/bin/env nu

let self_dir = ($env.FILE_PWD)

do {
  cd $self_dir
  brew bundle install
  proto upgrade
  proto clean --yes --days 60
  rye self update
  ./blend install
}
