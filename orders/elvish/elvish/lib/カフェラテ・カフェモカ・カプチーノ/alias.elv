use str
use path


# https://github.com/arethetypeswrong/arethetypeswrong.github.io/tree/main/packages/cli
fn attw {|@args|
  npx --package=@arethetypeswrong/cli attw $@args
}
fn b {|@args| e:bat $@args }
fn c { e:clear }
fn ip {|@args| e:ip -c $@args }
fn e {|@args| e:eza $@args }
fn ee {|@args| e:eza -l $@args }
fn tree {|@args| e:eza -T $@args }
fn l {|@args| e:ls --color $@args }
fn p {|@args| e:paru $@args }
fn p-rm-orphan { e:paru -Rns (e:paru -Qtdq) }
fn pping {|@args| e:prettyping $@args }
fn atom {|@args| e:env PYTHON=python2 atom --enable-transparent-visuals --disable-gpu $@args & }
fn code {|@args|
  # workaround for macOS 26
  # https://github.com/microsoft/vscode/pull/267724#issuecomment-3316457267
  env CHROME_HEADLESS=1 code $@args &
}
fn aria {|@args| e:aria2c --conf-path={~}/bkped/aria2c.conf }
fn s {|@args| e:systemctl $@args }
fn f {|@args| e:fd $@args }
fn r {|@args| e:rg $@args }
fn rs {|@args| e:rsync @args }
fn t {|@args| e:ydcv -s $@args }
fn tt {|@args| e:ydcv $@args }
fn i {|@args| e:time $@args }
fn d {|@args| e:docker $@args }
fn q {|@args| e:qalc $@args }
fn y {|@args| e:yarn $@args }
fn yrst { e:rm -rf ./node_modules/;y }

fn rua {|@args| e:rustup $@args }
fn j {|@args| e:jj $@args }
fn g {|@args|
  e:alias-g $@args
}

fn gcm {|@args|
  e:git credential-manager $@args
}

fn cz {|@args|
  e:npx cz $@args
}

fn br {
  git for-each-ref 'refs/heads' --format="%(color:cyan)%(refname:short)"
}

# TODO: better place for this
fn gsign_on {
  if (path:is-dir .sl) {
    e:sl config --local gpg.enabled true
  }
  if (path:is-dir .git) {
    e:git config commit.gpgsign true
  }
}
fn gsign_off {
  if (path:is-dir .sl) {
    e:sl config --local gpg.enabled false
  }
  if (path:is-dir .git) {
    e:git config commit.gpgsign false
  }
}

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
fn phantomjs { e:env QT_QPA_PLATFORM='' phantomjs }

# disable annoying auto word wrap...
fn nano {|@args| e:nano -w $@args }

fn bat {|@args| e:bat --theme="TwoDark" $@args }

fn neofetch {|@args| e:fastfetch $@args }

fn prpr {|@args| e:proxychains $@args }
fn prprme { e:proxychains elvish }

# simple py http server
fn pyserv { e:python -m http.server }

# test sddm theme
# sddm-test-theme PATH/TO/THEME
fn sddm-test-theme {|@args| e:sddm-greeter --test-mode --theme $@args }

# browser-sync
fn serve { e:browser-sync start --server }

# count files/matches of folder
fn count-file {|@args| e:find $@args -type f | wc -l }
fn count-match {|pattern| + (e:rg -ci --no-filename $pattern )}

# start hefur bittorrent tracker
fn tracker { e:hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970 }

# get WAN IP address
fn ipwan { e:dig +short myip.opendns.com @resolver1.opendns.com }
# get ipinfo(need prpr configured)
fn ipinfo {|@args| prpr curl --silent ipinfo.io/$@args }
fn ipwaninfo { ipinfo (ipwan) }

fn renewip {
  # macOS only
  e:networksetup -setbootp Ethernet
  e:networksetup -setdhcp Ethernet
}

fn kitty-reload {
  # https://sw.kovidgoyal.net/kitty/conf/#kitty-conf
  e:killall -SIGUSR1 kitty
}

fn less-watch-file {|@args| e:less -R +F $@args }
