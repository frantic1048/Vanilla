#!/bin/elvish

use カフェラテ・カフェモカ・カプチーノ/env

# "load aliases"
# injecting variables in alias module to current scope
#
# https://elv.sh/ref/builtin.html#use-mod
# https://elv.sh/ref/edit.html#edit:add-var
var alias = (use-mod カフェラテ・カフェモカ・カプチーノ/alias)
keys $alias | each {|f| edit:add-var $f $alias[$f] }

use カフェラテ・カフェモカ・カプチーノ/prompt

# FIXME: not working in elvish 0.15.0 yet
use カフェラテ・カフェモカ・カプチーノ/completion

if (has-external starship) {
    eval (starship init elvish)
}

# feeling bad about this　ಠ_ಠ
# https://elv.sh/ref/readline-binding.html
# https://github.com/elves/elvish/blob/master/pkg/mods/readline-binding/readline-binding.elv
# use readline-binding
# {
#     var bindsym = {|k f| set edit:insert:binding[$k] = $f }
#     # Alt+l for location mode is not working on macOS...
#     # bind Ctrl-Y to location mode
#     $bindsym Ctrl-Y $edit:location:start~
# }

# https://github.com/ghostty-org/ghostty/blob/main/src/shell-integration/README.md#elvish
if (has-env GHOSTTY_RESOURCES_DIR) {
    use kokkoro
    var at-env~ = $kokkoro:at-env~

    at-env &os="darwin" {
        eval (slurp <$E:GHOSTTY_RESOURCES_DIR/shell-integration/elvish/lib/ghostty-integration.elv)
    }
}
