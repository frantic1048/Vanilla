fn b [@args]{ e:bat --theme="TwoDark" $@args }
fn c { e:clear }
fn e [@args]{ e:exa $@args }
fn ee [@args]{ e:exa -l $@args }
fn l [@args]{ e:ls --color $@args }
fn p [@args]{ e:pikaur $@args }
fn p-rm-orphan { e:pikaur -Rns (e:pikaur -Qtdq) }
fn pping [@args]{ e:prettyping $@args }
fn atom [@args]{ e:env PYTHON=python2 atom --enable-transparent-visuals --disable-gpu $@args & }
fn code [@args]{ e:code --disable-gpu & }
fn aria [@args]{ e:aria2c --conf-path={~}/bkped/aria2c.conf }
fn s [@args]{ e:systemctl $@args }
fn f [@args]{ e:fd $@args }
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
  g--rela = '--date=relative'
  g--ff = '--ff-only'
  g--ol = '--pretty=oneline'
  g--ref-formatter = '--format=%(HEAD) %(color:#FEA090)%(objectname:short)%(color:reset) %(color:#89FE9F)%(refname:short)%(color:reset) - %(authorname) (%(color:#FEACD6)%(committerdate:relative)%(color:reset))'

  if (eq (count $args) 0) {
    g b | tail -n5
    g tu -s
    return
  }

  op @rest = $@args

  if (eq $op 'a') { g add $@rest; return }
  if (eq $op 'b') {
    if (eq (count $rest) 0) {
      git for-each-ref --sort=committerdate --color=always 'refs/heads/' $g--ref-formatter
    } else {
      git branch $@rest
    }
    return
  }
  if (eq $op 'ap') { g apply $@rest; return }
  if (eq $op 'bl') { g blame $@rest; return }

  # Commit
  if (eq $op 'c') { g commit $@rest; return }
  if (eq $op 'cnm') { g c -n -m $@rest; return }
  if (eq $op 'cnmw') { g a .; g c -n --allow-empty -m '[skip ci] wip'; return }
  if (eq $op 'f') { g c -n --fixup ':/'$@rest; return }
  if (eq $op 'ff') { g c -n --fixup $@rest; return }

  # ChecKout
  if (eq $op 'ck') { g checkout $@rest; return }
  if (eq $op 'ckb') { g checkout -b $@rest; return }
  if (eq $op 'ckm') { g checkout master; return }

  if (eq $op 'cb') { g rev-parse --abbrev-ref HEAD; return }

  # CherryPick
  if (eq $op 'cp') { g cherry-pick $@rest; return }
  if (eq $op 'cpc') { g cherry-pick --continue $@rest; return }
  if (eq $op 'cpa') { g cherry-pick --abort $@rest; return }

  # Diff HEAD
  if (eq $op 'dh') { g diff HEAD $@rest; return }
  if (eq $op 'dhc') { g diff HEAD --cached $@rest; return }

  if (eq $op 'fe') { g fetch $@rest; return }
  if (eq $op 'g') { g gui $@rest &; return }
  if (eq $op 'k') { gitk $@rest &; return }
  if (eq $op 'ka') { gitk --all $@rest &; return }
  if (eq $op 'm') { g merge $@rest; return }
  if (eq $op 'loc') { e:cloc $@args (g ls-files) $@rest; return }

  # Push
  if (eq $op 'p') { g push $@rest; return }
  if (eq $op 'P') { g push --force $@rest; return }
  if (eq $op 'pu') { g push -u origin (g cb); return }
  if (eq $op 'pl') { g pull $@rest; return }

  # ReBase
  if (eq $op 'rb') { g rebase $@rest; return }
  if (eq $op 'rbo') { g rebase --onto $@rest; return }
  if (eq $op 'rbi') { g rebase -i $@rest; return }
  if (eq $op 'rbm') { g rebase $@rest master; return }
  if (eq $op 'rbim') { g rebase -i $@rest master; return }
  if (eq $op 'rba') { g rebase --abort; return }
  if (eq $op 'rbc') { g rebase --continue; return }
  if (eq $op 'rbs') { g rebase --skip; return }

  if (eq $op 'ro') { g rev-parse --show-toplevel; return }
  if (eq $op 'rs') { g reset $@rest; return }
  if (eq $op 'rs1') { g reset "HEAD~1"; return }

  # Show Oneline <ref>
  # generating rebase commands
  if (eq $op 'so') { g show -q $g--ol $@rest; return }

  if (eq $op 'tu') { g status $@rest; return }
  if (eq $op 'ta') { g stash $@rest; return }
  if (eq $op 'rl') { g reflog $@rest; return }

  # TRee log
  if (eq $op 'tr') { g log --graph --abbrev-commit $g--rela --decorate=short --single-worktree $@rest; return }
  if (eq $op 'trr') { g log --graph --abbrev-commit $g--rela --decorate=short --all $@rest; return }
  if (eq $op 'to') { g tr $g--ol --single-worktree $@rest; return }
  if (eq $op 'too') { g tr $g--ol --all $@rest; return }

  if (eq $op 'wt') { g worktree $@rest; return }
  if (eq $op 'wc') { g whatchanged -p $@rest; return }
  if (eq $op 'RP') {
    g add .
    g commit --amend --no-verify --no-edit
    g push --force
    return
  }
  e:git $@args
}

fn br []{
  git for-each-ref 'refs/heads' --format="%(color:cyan)%(refname:short)"
}

# TODO: merge into g
fn gsign_on { e:git config commit.gpgsign true }
fn gsign_off { e:git config commit.gpgsign false }


# nvm does not want to see a prefix
fn nvm_on { e:npm config delete prefix }
fn nvm_off { e:npm config set prefix {~}/npm-global }

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
fn phantomjs { e:env QT_QPA_PLATFORM='' phantomjs }

# disable annoying auto word wrap...
fn nano [@args]{ e:nano -w $@args }

fn bat [@args]{ e:bat --theme="TwoDark" $@args }

fn neofetch [@args]{ e:neofetch --shell_version off $@args }

fn prpr [@args]{ e:proxychains $@args }
fn prprme { e:proxychains elvish }

# simple py http server
fn pyserv { e:python -m http.server }

# test sddm theme
# sddm-test-theme PATH/TO/THEME
fn sddm-test-theme [@args]{ e:sddm-greeter --test-mode --theme $@args }

# browser-sync
fn serve { e:browser-sync start --server }

# count files/matches of folder
fn count-file [@args]{ e:find $@args -type f | wc -l }
fn count-match [pattern]{ + (e:rg -ci --no-filename $pattern )}

# start hefur bittorrent tracker
fn tracker { e:hefurd -ipv6 -log-color -log-level info -udp-port 6969 -http-port 6969 -https-port 6970 }

# get WAN IP address
fn ipwan { e:dig +short myip.opendns.com @resolver1.opendns.com }
# get ipinfo(need prpr configured)
fn ipinfo [@args]{ prpr curl --silent ipinfo.io/$@args }
fn ipwaninfo { ipinfo (ipwan) }
