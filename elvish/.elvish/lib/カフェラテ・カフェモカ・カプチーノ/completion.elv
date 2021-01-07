use str

# git
fn g_completion [@args]{
    # 0     1     2
    # g     OP    "STUFF TO BE COMPLETED"
    if (!= (count $args) 3) {
        # this is not our case, use default filename completion
        put (edit:complete-filename $args[-1])
        return
    }

    op = $args[1]

    if (eq $op 'ck') {
        # git checkout
        # completes recent branches
        each [line]{
            index candidate message = (str:split "\t" $line)

            put (edit:complex-candidate $candidate &display=$index" "$message)

            # FIXME:
            # --sort not working yet since elvish always sort candidates...
            # using nl to workaround this issue...

            # MEMO:
            # git for-each-ref --no-contains=(git rev-parse HEAD)
            # omits all branch with same ref.
            #
            # I only want to omit current branch.
            # using `rg` to remove current branch from candidates
      } [(git for-each-ref 'refs/heads/' ^
          --sort="-committerdate" ^
          --format="%(refname:short)\t%(objectname:short) %(refname:short) %(authorname) %(committerdate:relative)" ^
          | rg -v (g cb) | nl -nrz -w3)]
    } else {
        # default completion
        put (edit:complete-filename $args[-1])
    }
}

edit:completion:arg-completer[g] = $g_completion~
