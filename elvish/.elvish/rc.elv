#!/bin/elvish

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
fn l [@]{ e:ls --color $@ }
fn p [@]{ e:pacaur $@ }
fn pping [@]{ e:prettyping $@ }
fn atom [@]{ e:env PYTHON=python2 atom --enable-transparent-visuals --disable-gpu $@ & }
fn nano [@]{ e:nano -w $@ }
fn aria [@]{ e:aria2c --conf-path={~}/bkped/aria2c.conf }
fn s [@]{ e:systemctl $@ }
fn r [@]{ e:rsync $@ }
fn d [@]{ e:docker $@ }
fn n [@]{ e:npm $@ }
fn y [@]{ e:yarn $@ }
fn rua [@]{ e:rustup $@ }

fn g [@]{ e:git $@ }
g--ff = '--ff-only'
g--rela = '--date=relative'
g--ol = '--pretty=oneline'
fn gtree [@]{ e:git log --graph --abbrev-commit $g--rela --decorate=short --all $@ }
fn gtreeo [@]{ gtree $g--ol $@ }
fn gdh [@]{ e:git diff HEAD $@  }
fn gsign_on { e:git config commit.gpgsign true }
fn gsign_off { e:git config commit.gpgsign false }

# nvm does not want to see a prefix
fn nvm_on { e:npm config delete prefix }
fn nvm_off { e:npm config set prefix /home/chino/npm-global }

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
fn phantomjs { e:env QT_QPA_PLATFORM='' phantomjs }

# disable annoying auto word wrap...
fn nano [@]{ e:nano -w $@ }

fn neofetch { e:neofetch --shell_version off}

fn prpr [@]{ e:proxychains $@ }
fn prprme { e:proxychains elvish }

# simple py http server
fn pyserv { e:python -m http.server }

# test sddm theme
# sddm-test-theme PATH/TO/THEME
fn sddm-test-theme [@]{ e:sddm-greeter --test-mode --theme $@ }

# browser-sync
fn serve { e:browser-sync start --server }

# count files of folder
fn count-file [@]{ e:find $@ -type f | wc -l }

# start hefur bittorrent tracker
fn tracker { e:hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970 }

# get WAN IP address
fn ipwan { e:dig +short myip.opendns.com @resolver1.opendns.com }
# get ipinfo(need prpr configured)
fn ipinfo [@]{ prpr curl --silent ipinfo.io/$@ }
fn ipwaninfo { ipinfo (ipwan) }
