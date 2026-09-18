#!/usr/bin/env nu

const cleaner = path self clean-elvish-directory-history.elv

# Using the cleaner as an RC file gives it access to Elvish's interactive-only
# store: module. The marker makes the child exit before showing a nested prompt.
with-env { CLEAN_ELVISH_DIRECTORY_HISTORY_ONESHOT: "1" } {
    ^elvish -rc $cleaner
}
