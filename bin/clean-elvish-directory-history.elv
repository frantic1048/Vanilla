#!/usr/bin/env elvish

# clear invalid directories from elvish directory history

# FIXME:
# The store: module provides access to Elvish’s persistent data store. It is only available in interactive mode now.
#
# do this to call the script:
# eval (slurp <~/bin/clean-elvish-directory-history.elv)

use store
use path

put (store:dirs) | each {|dir|
  if (not (path:is-dir $dir[path])) {
    echo found invalid dir: $dir[path]
    store:del-dir $dir[path]
  }
}