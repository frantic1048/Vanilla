#!/bin/elvish

use re
#use 1048

# env vars
E:PATH = {$E:PATH}:{~}/npm-global/bin:{~}/.gem/ruby/2.2.0/bin:{~root}/.composer/vendor/bin:{~}/bin
E:NODE_PATH = {~}/npm-global/lib/node_modules/:/usr/lib/node_modules/:{$E:NODE_PATH}
E:VISUAL = "nano"

fn emsdk_env {
# based on output of command
# source portable/emsdk-portable/emsdk_env.sh
  E:PATH = {$E:PATH}:{~}/portable/emsdk-portable
  E:PATH = {$E:PATH}:{~}/portable/emsdk-portable/clang/fastcomp/build_incoming_64/bin
  E:PATH = {$E:PATH}:{~}/portable/emsdk-portable/node/4.1.1_64bit/bin
  E:PATH = {$E:PATH}:{~}/portable/emsdk-portable/emscripten/incoming
  E:EMSDK = {~}/portable/emsdk-portable
  E:EM_CONFIG = {~}/.emscripten
  E:EMSCRIPTEN = {~}/portable/emsdk-portable/emscripten/incoming
}

# aliases

# refresh rc.elv
fn rc { -source {~}/.elvish/rc.elv }

fn c { e:clear }
fn l [@args]{ e:ls --color $@args }
fn p [@args]{ e:pacaur $@args }
fn pping [@args]{ e:prettyping $@args }
fn atom [@args]{ e:env PYTHON=python2 atom --enable-transparent-visuals --disable-gpu $@args & }
fn code [@args]{ /opt/visual-studio-code/code --disable-gpu & }
fn aria [@args]{ e:aria2c --conf-path={~}/bkped/aria2c.conf }
fn s [@args]{ e:systemctl $@args }
fn r [@args]{ e:rg $@args }
fn rs [@args]{ e:rsync @args }
fn t [@args]{ e:ydcv -s $@args }
fn tt [@args]{ e:ydcv $@args }
fn i [@args]{ e:time $@args }
fn d [@args]{ e:docker $@args }
fn n [@args]{ e:npm $@args }
fn y [@args]{ e:yarn $@args }
fn yrst { e:rm -rf ./node_modules/;y }

fn rua [@args]{ e:rustup $@args }
fn g [@args]{
  fn ss []{
    # stash staged file
    echo TBD
  }
  fn cbr []{ replaces "## " "" (g status -sb -uno) }
  fn pu []{
    g push -u origin (g cbr)
  }
  fn frp []{
    g add .
    g commit --amend --no-verify --no-edit
    g push --force
  }

  if (eq (count $args) 0) { g --help; return }

  op @rest = $@args

  #if (eq $op 'ss') { ss; return }
  if (eq $op 'b') { g branch $@rest; return }
  if (eq $op 'c') { g commit $@rest; return }
  if (eq $op 'cbr') { cbr; return }
  if (eq $op 'p') { g push $@rest; return }
  if (eq $op 'pu') { pu; return }
  if (eq $op 'frp') { frp; return }
  e:git $@args
}
g--ff = '--ff-only'
g--rela = '--date=relative'
g--ol = '--pretty=oneline'
fn gtree [@args]{ e:git log --graph --abbrev-commit $g--rela --decorate=short --all $@args }
fn gtreeo [@args]{ gtree $g--ol $@args }
fn gdh [@args]{ e:git diff HEAD $@args  }
fn gsign_on { e:git config commit.gpgsign true }
fn gsign_off { e:git config commit.gpgsign false }
fn gloc [@args]{ e:cloc $@args (g ls-files) }

# nvm does not want to see a prefix
fn nvm_on { e:npm config delete prefix }
fn nvm_off { e:npm config set prefix {~}/npm-global }

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
fn phantomjs { e:env QT_QPA_PLATFORM='' phantomjs }

# disable annoying auto word wrap...
fn nano [@args]{ e:nano -w $@args }

fn neofetch { e:neofetch --shell_version off}

fn prpr [@args]{ e:proxychains $@args }
fn prprme { e:proxychains elvish }

# simple py http server
fn pyserv { e:python -m http.server }

# test sddm theme
# sddm-test-theme PATH/TO/THEME
fn sddm-test-theme [@args]{ e:sddm-greeter --test-mode --theme $@args }

# browser-sync
fn serve { e:browser-sync start --server }

# count files of folder
fn count-file [@args]{ e:find $@args -type f | wc -l }

# start hefur bittorrent tracker
fn tracker { e:hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970 }

# get WAN IP address
fn ipwan { e:dig +short myip.opendns.com @resolver1.opendns.com }
# get ipinfo(need prpr configured)
fn ipinfo [@args]{ prpr curl --silent ipinfo.io/$@args }
fn ipwaninfo { ipinfo (ipwan) }
