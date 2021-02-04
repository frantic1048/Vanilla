#!/bin/elvish

use カフェラテ・カフェモカ・カプチーノ/env

# "load aliases"
# injecting variables in alias module to current scope
#
# https://elv.sh/ref/builtin.html#use-mod
# https://elv.sh/ref/edit.html#editadd-var
alias = (use-mod カフェラテ・カフェモカ・カプチーノ/alias)
keys $alias | each [f]{ edit:add-var $f $alias[$f] }

# MEMO: emmmmm, not working because add-vars needs a map, not namespace
# edit:add-vars (use-mod カフェラテ・カフェモカ・カプチーノ/alias)


use カフェラテ・カフェモカ・カプチーノ/prompt

# FIXME: not working in elvish 0.15.0 yet
use カフェラテ・カフェモカ・カプチーノ/completion