#!/bin/elvish

# env vars
E:PATH = {$E:PATH}:{~}/npm-global/bin:{~}/.gem/ruby/2.2.0/bin:{~root}/.composer/vendor/bin:{~}/bin
E:NODE_PATH = {~}/npm-global/lib/node_modules/:/usr/lib/node_modules/:{$E:NODE_PATH}
E:VISUAL = "nano"

in_reg = '--registry=https://npm.in.chaitin.com'

fn emsdk_set_env {
  # make sure emsdk being choosed first
  E:PATH = {~}/portable/emsdk-portable:{~}/portable/emsdk-portable/clang/fastcomp/build_incoming_64/bin:{~}/portable/emsdk-portable/node/4.1.1_64bit/bin:{~}/portable/emsdk-portable/emscripten/incoming:{$E:PATH}
  E:EMSDK = {~}/portable/emsdk-portable
  E:EM_CONFIG = {~}/.emscripten
}

emsdk_set_env

# aliases
fn ls { e:ls --color $@ }
fn p { e:pacaur $@ }
fn pping { e:prettyping $@ }
fn atom { e:env PYTHON=python2 atom --enable-transparent-visuals --disable-gpu $@ & }
fn nano { e:nano -w $@ }
fn aria { e:aria2c --conf-path={~}/bkped/aria2c.conf }


fn n { e:npm $@ }
fn y { e:yarn $@ }
fn g { e:git $@ }
fn gdh { e:git diff HEAD $@  }
fn gsign-on { e:git config commit.gpgsign true }
fn gsign-off { e:git config commit.gpgsign false }

# nvm does not want to see a prefix
fn nvm_on { e:npm config delete prefix }
fn nvm_off { e:npm config set prefix /home/chino/npm-global }

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
fn phantomjs { e:env QT_QPA_PLATFORM='' phantomjs }

fn neofetch { e:neofetch --shell_version off }

fn prpr { e:proxychains $@ }
fn prprme { e:proxychains elvish }

# simple py http server
fn pyserv { e:python -m http.server }

# test sddm theme
# sddm-test-theme PATH/TO/THEME
fn sddm-test-theme { e:sddm-greeter --test-mode --theme $@ }

# browser-sync
fn serve { e:browser-sync start --server }

# count files of folder
fn file_count { e:find $@ -type f | wc -l }

# start hefur bittorrent tracker
fn tracker { e:hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970 }

