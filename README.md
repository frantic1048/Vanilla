# Vanilla

frantic1048's daily configs, scripts etc.

## Screenshots

_Sway_:

![toastx900](screenshots/toastx900_2021-07-30_13-00.png)
![toastx900](screenshots/toastx900_2021-07-30_13-14.png)

_i3_:

![amausaan](screenshots/amausaan_2022-04-05-232523.png)

## Config packages

| package                  | profiles                       |
| ------------------------ | ------------------------------ |
| [bin](bin)               | `macos-x86_64`, `linux-x86_64` |
| [elvish](elvish)         | `macos-x86_64`, `linux-x86_64` |
| [git](git)               | `macos-x86_64`, `linux-x86_64` |
| [alacritty](alacritty)   | `macos-x86_64`                 |
| [kitty](kitty)           | `macos-x86_64`                 |
| [nushell](nushell)       | `macos-x86_64`                 |
| [proto](proto)           | `macos-x86_64`                 |
| [sketchybar](sketchybar) | `macos-x86_64`                 |
| [skhd](skhd)             | `macos-x86_64`                 |
| [starship](starship)     | `macos-x86_64`                 |
| [vscode](vscode)         | `macos-x86_64`                 |
| [wezterm](wezterm)       | `macos-x86_64`                 |
| [yabai](yabai)           | `macos-x86_64`                 |
| [X](X)                   |                                |
| [alsa](alsa)             |                                |
| [atom](atom)             |                                |
| [bash](bash)             |                                |
| [code-oss](code-oss)     |                                |
| [color](color)           |                                |
| [commitizen](commitizen) |                                |
| [conky](conky)           |                                |
| [fcitx](fcitx)           |                                |
| [fontconfig](fontconfig) |                                |
| [htop](htop)             |                                |
| [i3wm](i3wm)             |                                |
| [krita](krita)           |                                |
| [makepkg](makepkg)       |                                |
| [mako](mako)             |                                |
| [nano](nano)             |                                |
| [neofetch](neofetch)     |                                |
| [npm](npm)               |                                |
| [pam_env](pam_env)       |                                |
| [picom](picom)           |                                |
| [pipewire](pipewire)     |                                |
| [psd](psd)               |                                |
| [pulseaudio](pulseaudio) |                                |
| [rofi](rofi)             |                                |
| [sakura](sakura)         |                                |
| [stow](stow)             |                                |
| [sway](sway)             |                                |
| [swayshot](swayshot)     |                                |
| [sxiv](sxiv)             |                                |
| [tint2](tint2)           |                                |
| [tmux](tmux)             |                                |
| [waybar](waybar)         |                                |

## Usage

### All in one

#### macOS

Require following dependencies in `PATH`:

1. `bash`: https://www.gnu.org/software/bash/
2. `curl`: https://curl.se/

```sh
./bootstrap_macos.sh
```

### Install standalone config package(s)

Require following dependencies in `PATH`:

1. `git`: https://git-scm.com/
2. `stow`: https://www.gnu.org/software/stow/
3. `nu`: https://www.nushell.sh/

```sh
./blend install [package1] [package2] ...

# install all available packages
./blend install
```
