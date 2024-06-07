#!/usr/bin/env nu

let content = $"# 𝒱𝒶𝓃𝒾𝓁𝓁𝒶

frantic1048's daily configs, scripts etc.

## Screenshots

_Sway_:

![toastx900]\(screenshots/toastx900_2021-07-30_13-00.png)
![toastx900]\(screenshots/toastx900_2021-07-30_13-14.png)

_i3_:

![amausaan]\(screenshots/amausaan_2022-04-05-232523.png)

## Config packages

(./blend stat_markdown)

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
2. `stow`: https://www.gnu.org/software/stow/
3. `nu`: https://www.nushell.sh/

```sh
./blend install [package1] [package2] ...

# install all available packages
./blend install
```
"

$content | save --force README.md