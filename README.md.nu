#!/usr/bin/env nu

let content = $"# Vanilla

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

1. `bash`: https://www.gnu.org/software/bash/
2. `curl`: https://curl.se/

```sh
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