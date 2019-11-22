# git

fn g_completion [@args]{
    # 0     1     2
    # g     OP    "STUFF TO BE COMPLETED"
    if (!= (count $args) 3) { return }

    op = $args[1]

    if (eq $op 'ck') {
      # checkout branches
      each [line]{
        candidate message = (splits "\t" $line)

        # ABUSE:
        # use empty string as candidate, &code-suffix for real candidate
        # so we can fully contronl candidate display message via &display-suffix
        put (edit:complex-candidate '' &code-suffix=$candidate &display-suffix=$message)

        # FIXME: 
        # --sort not working yet since elvish always sort candidates...
      } [(git for-each-ref 'refs/heads/' \
          --sort="-committerdate" \
          --no-contains=(git rev-parse HEAD) \
          --format="%(refname:short)\t%(objectname:short) %(refname:short) %(authorname) %(committerdate:relative)")]
    }
}

edit:completion:arg-completer[g] = $g_completion~