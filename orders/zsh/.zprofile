# macOS /etc/zprofile runs path_helper after ~/.zshenv.
# Re-apply centralized env so user-managed toolchains stay before system paths.
eval "$(~/.local/bin/van/shellenv)"
