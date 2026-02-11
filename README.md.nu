#!/usr/bin/env nu

let blend_rs = [$env.FILE_PWD "blend-rs" "target" "release" "blend"] | path join

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

## Config packages

Configs are defined as Nickel DSL orders in `orders/` and deployed via `blend-rs`.

(run-external $blend_rs "table" | str trim)

## Usage

### All in one

#### macOS

Require following dependencies in `PATH`:

1. `git`: https://git-scm.com/
2. `bash`: https://www.gnu.org/software/bash/
3. `curl`: https://curl.se/

```sh
# necessary for git
sudo xcode-select --install

git clone https://github.com/frantic1048/Vanilla.git
cd Vanilla

./bootstrap_macos.sh
```

### Install standalone config package\(s)

Require following dependencies in `PATH`:

1. `git`: https://git-scm.com/
2. `nu`: https://www.nushell.sh/

```sh
./blend install [package1] [package2] ...

# install all available packages
./blend install
```
"

$content | save --force README.md
