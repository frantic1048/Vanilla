#!/usr/bin/env nu

let blend_bin = [$env.FILE_PWD "blend" "target" "release" "blend"] | path join

let content = $"# 𝒱𝒶𝓃𝒾𝓁𝓁𝒶

frantic1048's daily configs, scripts etc.

## Screenshots

_Sway_:

![toastx900]\(screenshots/toastx900_2021-07-30_13-00.png)
![toastx900]\(screenshots/toastx900_2021-07-30_13-14.png)

_i3_:

![amausaan]\(screenshots/amausaan_2022-04-05-232523.png)

_macOS_:

![macos_screenshot]\(screenshots/macbook_2024-12-19.png)

## Config orders

Configs are defined as Nickel DSL orders in `orders/` and deployed via `blend`.

(run-external $blend_bin "table" | str trim)

## Usage

### All in one

#### macOS / Arch Linux

Fresh macOS needs Xcode Command Line Tools before the repository can be cloned
and built. The bootstrap script then installs Homebrew when missing, runs the
repo `Brewfile`, installs proto plus a baseline set of proto-managed toolchains,
builds `blend`, and deploys the matching dotfile orders.

Require following dependencies in `PATH` before running `./bootstrap.sh`:

1. `git`: https://git-scm.com/
2. `bash`: https://www.gnu.org/software/bash/
3. `curl`: https://curl.se/

```sh
# macOS only; wait for the installer to finish before cloning.
sudo xcode-select --install

git clone https://github.com/frantic1048/Vanilla.git
cd Vanilla

./bootstrap.sh
```

On macOS, `./bootstrap.sh` may ask for interactive confirmation during Homebrew
and cask installation. After it finishes, follow the printed checklist for
account-level setup such as the default shell, git credentials, Raycast, and
macOS system preferences.

### Sync standalone config order\(s)

Require following dependencies in `PATH`:

1. `git`: https://git-scm.com/
2. `just`: https://just.systems/
3. Rust/Cargo \(for building `blend`\)

```sh
just build

./bin/blend sync [order1] [order2] ...

# sync all available orders
./bin/blend sync
```
"

$content | save --force README.md
