# Shell
brew "elvish"
brew "nushell"
brew "starship"

# Terminal workflow
brew "tmux"
brew "zellij"
brew "pueue"

# macOS has very old versions of these tools
# builtin rsync@2.6.9, Nov 2006
brew "rsync"
# builtin bash@3.2.57, Nov 2014
brew "bash"
# builtin make@3.81, Apr 2006
brew "make"

# Core CLI
brew "bat"
brew "btop"
brew "coreutils"
brew "curl"
brew "difftastic"
brew "eza"
brew "fastfetch"
brew "fd"
brew "findutils"
brew "fzf"
brew "gawk"
brew "gnu-time"
brew "htop"
brew "hyperfine"
brew "inxi"
brew "jless"
brew "jq"
brew "libqalculate"
brew "mtr"
brew "ncdu"
brew "onefetch"
brew "ripgrep"
brew "scc"
brew "tealdeer"
brew "unar"
brew "wget"

# Editors and language tooling
brew "go-jsonnet"
brew "just"
brew "marksman"
brew "neovim"
brew "nickel"
brew "typos-cli"
brew "watchman"

# Lint and formatting
brew "actionlint"
brew "ast-grep"
brew "shellcheck"
brew "shfmt"
brew "svgo"

# Git and friends
brew "git"
brew "git-lfs"
cask "git-credential-manager"
brew "git-delta"
brew "git-filter-repo"
brew "jj"
brew "jjui"

# Image, video, and document processing
brew "exiv2"
brew "ffmpeg"
brew "graphviz"
brew "imagemagick"
brew "optipng"
brew "sevenzip"
brew "tesseract"
brew "webp"
brew "yt-dlp"

# Network and infrastructure
brew "butane"
brew "caddy"
brew "cloudflared"
brew "ipcalc"
brew "ldns"
brew "mintoolkit"
brew "netbirdio/tap/netbird"
brew "nmap"
brew "s3cmd"
brew "tailscale"
brew "tilt"
brew "hashicorp/tap/terraform"
brew "vectordotdev/brew/vector"
brew "vultr/vultr-cli/vultr-cli"
cask "ngrok"

# Kubernetes
brew "azure/kubelogin/kubelogin"
brew "kubectx"
brew "kubernetes-cli"

# Android
cask "android-platform-tools"

# TLS certificates for local development
# https://github.com/FiloSottile/mkcert
brew "mkcert"
brew "nss"

# Secrets and signing
brew "pinentry-mac"
brew "sops"
brew "ykman"

# Identity
# https://kanidm.github.io/kanidm/stable/installing_client_tools.html
brew "kanidm/kanidm/kanidm"

# Platform CLIs
brew "gh"
brew "glab"
brew "datadog-labs/pack/pup"
brew "getsentry/tools/sentry"

# Atlassian CLI
# upstream changed to installation script ... (°_°)
# https://developer.atlassian.com/cloud/twg-cli/getting-started/installation/
