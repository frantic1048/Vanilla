#!/usr/bin/env nu

let self_dir = ($env.FILE_PWD)

do {
  cd $self_dir
  proto upgrade
  rye self update
  paru -Sy archlinux-keyring archlinuxcn-keyring
  paru -Syuw
  paru -Syu
  paru -Rns (paru -Qdtq)
}
