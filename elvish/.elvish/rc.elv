#!/bin/elvish

# env vars
E:PATH = $E:PATH{~}/npm-global/bin/:{~}/.gem/ruby/2.2.0/bin/:{~root}/.composer/vendor/bin:{~}/bin/

# aliases

fn ls { e:ls --color $@ }
