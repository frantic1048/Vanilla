fn b {|@args| e:bat --theme="TwoDark" $@args }
fn c { e:clear }
fn e {|@args| e:exa $@args }
fn ee {|@args| e:exa -l $@args }
fn l {|@args| e:ls --color $@args }
fn p {|@args| e:pikaur $@args }
fn p-rm-orphan { e:pikaur -Rns (e:pikaur -Qtdq) }
fn pping {|@args| e:prettyping $@args }
fn atom {|@args| e:env PYTHON=python2 atom --enable-transparent-visuals --disable-gpu $@args & }
fn code {|@args| e:code --disable-gpu & }
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
fn g {|@args|
  var g--rela = '--date=relative'
  var g--ff = '--ff-only'
  var g--ol = '--pretty=oneline'
  var g--ref-formatter = '--format=%(HEAD) %(color:#FEA090)%(objectname:short)%(color:reset) %(color:#89FE9F)%(refname:short)%(color:reset) - %(authorname) (%(color:#FEACD6)%(committerdate:relative)%(color:reset))'

  if (== (count $args) 0) {
    g b | tail -n5
    g tu -s
    return
  }

  var op @rest = $@args

  if (==s $op 'a') { g add $@rest; return }
  if (==s $op 'b') {
    if (== (count $rest) 0) {
      git for-each-ref --sort=committerdate --color=always 'refs/heads/' $g--ref-formatter
    } else {
      git branch $@rest
    }
    return
  }
  if (==s $op 'ap') { g apply $@rest; return }
  if (==s $op 'bl') { g blame $@rest; return }

  # Commit
  if (==s $op 'c') { g commit $@rest; return }
  if (==s $op 'cnm') { g c -n -m $@rest; return }
  if (==s $op 'cnmw') { g a .; g c -n --allow-empty -m '[skip ci] wip'; return }
  if (==s $op 'f') { g c -n --fixup ':/'$@rest; return }
  if (==s $op 'ff') { g c -n --fixup $@rest; return }

  # ChecKout
  if (==s $op 'ck') { g checkout $@rest; return }
  if (==s $op 'ckb') { g checkout -b $@rest; return }
  if (==s $op 'ckm') { g checkout master; return }

  if (==s $op 'cb') { g rev-parse --abbrev-ref HEAD; return }

  # CherryPick
  if (==s $op 'cp') { g cherry-pick $@rest; return }
  if (==s $op 'cpc') { g cherry-pick --continue $@rest; return }
  if (==s $op 'cpa') { g cherry-pick --abort $@rest; return }

  # Diff HEAD
  if (==s $op 'dh') { g diff HEAD $@rest; return }
  if (==s $op 'dhc') { g diff HEAD --cached $@rest; return }

  if (==s $op 'fe') { g fetch $@rest; return }
  if (==s $op 'g') { g gui $@rest &; return }
  if (==s $op 'k') { gitk $@rest &; return }
  if (==s $op 'ka') { gitk --all $@rest &; return }
  if (==s $op 'loc') { e:cloc $@args (g ls-files) $@rest; return }

  # Push
  if (==s $op 'p') { g push $@rest; return }
  if (==s $op 'P') { g push --force $@rest; return }
  if (==s $op 'pu') { g push -u origin (g cb); return }
  if (==s $op 'pl') { g pull $@rest; return }

  # Merge
  if (==s $op 'm') { g merge $@rest; return }
  ## ff?(use / as ? alternate) : can current branch fast forward with $@rest
  if (==s $op 'mf/') { put (==s (g merge-base HEAD $@rest) (g rev-parse HEAD)); return }
  # do merge when ff is possible, and create a merge commit, aka semi-linear history merging strategy
  if (==s $op 'mf') { if (g mf/ $@rest) { g merge --no-ff $@rest } else { echo 'outdated branch !' }; return}

  # ReBase
  if (==s $op 'rb') { g rebase $@rest; return }
  if (==s $op 'rbo') { g rebase --onto $@rest; return }
  if (==s $op 'rbi') { g rebase -i $@rest; return }
  # extra "b" means rebase more stuff -> rebase merges
  if (==s $op 'rbbi') { g rebase --rebase-merges -i $@rest; return }
  if (==s $op 'rbm') { g rebase $@rest master; return }
  if (==s $op 'rbim') { g rebase -i $@rest master; return }

  # ReBase mode commands
  if (==s $op 'rba') { g rebase --abort; return }
  if (==s $op 'rbc') { g rebase --continue; return }
  if (==s $op 'rbs') { g rebase --skip; return }

  if (==s $op 'rp') { g rev-parse --show-toplevel; return }

  # ReSet
  if (==s $op 'rs') { g reset $@rest; return }
  if (==s $op 'rs1') { g reset "HEAD~1"; return }

  # Show Oneline <ref>
  # generating rebase commands
  if (==s $op 'so') { g show -q $g--ol $@rest; return }

  if (==s $op 'tu') { g status $@rest; return }
  if (==s $op 'ta') { g stash $@rest; return }
  if (==s $op 'rl') { g reflog $@rest; return }

  # TRee log
  if (==s $op 'tr') { g log --graph --abbrev-commit $g--rela --decorate=short --single-worktree $@rest; return }
  if (==s $op 'trr') { g log --graph --abbrev-commit $g--rela --decorate=short --all $@rest; return }
  if (==s $op 'to') { g tr $g--ol --single-worktree $@rest; return }
  if (==s $op 'too') { g tr $g--ol --all $@rest; return }

  if (==s $op 'wt') { g worktree $@rest; return }
  if (==s $op 'wc') { g whatchanged -p $@rest; return }

  e:git $@args
}

fn br {
  git for-each-ref 'refs/heads' --format="%(color:cyan)%(refname:short)"
}

# TODO: merge into g
fn gsign_on { e:git config commit.gpgsign true }
fn gsign_off { e:git config commit.gpgsign false }

# FIX phantomjs crash issue
# https://github.com/ariya/phantomjs/issues/14061
fn phantomjs { e:env QT_QPA_PLATFORM='' phantomjs }

# disable annoying auto word wrap...
fn nano {|@args| e:nano -w $@args }

fn bat {|@args| e:bat --theme="TwoDark" $@args }

fn neofetch {|@args| e:neofetch --shell_version off $@args }

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
