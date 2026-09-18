#!/usr/bin/env elvish

# Remove paths that no longer exist from Elvish's directory history.
#
# The store: module is only available in interactive mode. Prefer the
# clean-elvish-directory-history.nu wrapper, or run this directly with:
# eval (slurp <~/Vanilla/bin/clean-elvish-directory-history.elv)

use path
use store

var invalid = []
each {|dir|
  if (not (path:is-dir $dir[path])) {
    set invalid = (conj $invalid $dir[path])
  }
} [(store:dirs)]

each {|dir|
  echo removing invalid directory: $dir
  store:del-dir $dir
} $invalid

echo removed (count $invalid) invalid directory history entries

if (has-env CLEAN_ELVISH_DIRECTORY_HISTORY_ONESHOT) {
  exit
}
