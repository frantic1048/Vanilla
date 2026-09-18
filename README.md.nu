#!/usr/bin/env nu

let blend_bin = $env.BLEND_BIN? | default "blend"

let content = $"# 𝒱𝒶𝓃𝒾𝓁𝓁𝒶

frantic1048's daily configs, scripts etc. Managed by [blend]\(https://github.com/frantic1048/blend).

## Screenshots

_Sway_:

![toastx900]\(screenshots/toastx900_2021-07-30_13-00.png)
![toastx900]\(screenshots/toastx900_2021-07-30_13-14.png)

_i3_:

![amausaan]\(screenshots/amausaan_2022-04-05-232523.png)

_macOS_:

![macos_screenshot]\(screenshots/macbook_2024-12-19.png)

## Contents

This repository mainly contains configs for various tools and applications shown in the table below.

Configs are defined as [DSL]\(orders/order.contract.ncl) in [Nickel]\(https://github.com/nickel-lang/nickel) language under [orders/]\(orders/). They are deployed with the standalone [blend CLI]\(https://github.com/frantic1048/blend).

(run-external $blend_bin "--sandbox" "never" "--blend-dir" $env.FILE_PWD "table" | str trim)

## Usage

### Using standalone order\(s)

Require a stable `blend` executable in `PATH`. Vanilla does not install or pin
the local CLI; Orders CI independently pins a released container image and
digest.

```sh
# interactively deploy specific order\(s)
just sync [order1] [order2] ...

# interactively sync all available orders
just sync
```

### bootstrap script

> [!CAUTION]
> The bootstrap script is intended for fresh systems only. It will install
> various tools and deploy all orders in this repository.
> Do not run it if you already have a working environment.

#### macOS / Arch Linux

Fresh macOS needs Xcode Command Line Tools before the repository can be cloned.
The bootstrap script then installs Homebrew when missing, runs the
repo `Brewfile`, installs proto plus a baseline set of proto-managed toolchains,
and deploys the matching dotfile orders. The script currently expects `blend`
to be available in `PATH`; installing it is intentionally left to the user.

Require following dependencies in `PATH` before running `./bootstrap.sh`:

1. `git`: https://git-scm.com/
2. `bash`: https://www.gnu.org/software/bash/
3. `curl`: https://curl.se/
4. `blend`: https://github.com/frantic1048/blend

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

## blend repository history

blend versions through 0.2.16 were developed and released from this repository.
Those tags and releases remain available as historical pre-extraction records.
Starting with 0.3.0, [frantic1048/blend]\(https://github.com/frantic1048/blend)
is the only active source and release repository.

## Credits

- [xiaq]\(https://github.com/xiaq): Kindly [improved my elvish code]\(https://github.com/frantic1048/Vanilla/commits?author=xiaq) :)
"

$content | save --force README.md
