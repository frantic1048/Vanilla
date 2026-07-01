# Environment now comes from the centralized shellenv (bin/shellenv).
# rc.elv still does `use カフェラテ・カフェモカ・カプチーノ/env`; this shim applies it.
#
# The explicit path is required: ~/.local/bin/van is not yet on PATH at startup
eval (~/.local/bin/van/shellenv elvish | slurp)
