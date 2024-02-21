# Vanilla

frantic1048's daily configs, scripts etc.

## Screenshots

_Sway_:

![toastx900](screenshots/toastx900_2021-07-30_13-00.png)
![toastx900](screenshots/toastx900_2021-07-30_13-14.png)

_i3_:

![amausaan](screenshots/amausaan_2022-04-05-232523.png)

## Config packages

| package                             | profiles                       |
| ----------------------------------- | ------------------------------ |
| [alacritty](alacritty)              | `macos-x86_64`, `linux-x86_64` |
| [bin](bin/bin)                      | `macos-x86_64`, `linux-x86_64` |
| [elvish](elvish/elvish)             | `macos-x86_64`, `linux-x86_64` |
| [git](git/git)                      | `macos-x86_64`, `linux-x86_64` |
| [jj](jj/jj)                         | `macos-x86_64`, `linux-x86_64` |
| [neofetch](neofetch)                | `macos-x86_64`, `linux-x86_64` |
| [nushell](nushell/nushell)          | `macos-x86_64`, `linux-x86_64` |
| [pueue](pueue/pueue)                | `macos-x86_64`, `linux-x86_64` |
| [sapling](sapling/sapling)          | `macos-x86_64`, `linux-x86_64` |
| [starship](starship)                | `macos-x86_64`, `linux-x86_64` |
| [vscode](vscode/User)               | `macos-x86_64`, `linux-x86_64` |
| [kitty](kitty/kitty)                | `macos-x86_64`                 |
| [proto](proto)                      | `macos-x86_64`                 |
| [sketchybar](sketchybar/sketchybar) | `macos-x86_64`                 |
| [skhd](skhd/skhd)                   | `macos-x86_64`                 |
| [wezterm](wezterm)                  | `macos-x86_64`                 |
| [yabai](yabai/yabai)                | `macos-x86_64`                 |
| [X](X)                              | `linux-x86_64`                 |
| [alsa](alsa)                        | `linux-x86_64`                 |
| [fcitx](fcitx/fcitx)                | `linux-x86_64`                 |
| [fontconfig](fontconfig/fontconfig) | `linux-x86_64`                 |
| [htop](htop/htop)                   | `linux-x86_64`                 |
| [i3wm](i3wm/i3)                     | `linux-x86_64`                 |
| [makepkg](makepkg)                  | `linux-x86_64`                 |
| [mako](mako/mako)                   | `linux-x86_64`                 |
| [nano](nano/nano)                   | `linux-x86_64`                 |
| [npm](npm)                          | `linux-x86_64`                 |
| [pam_env](pam_env)                  | `linux-x86_64`                 |
| [picom](picom/picom)                | `linux-x86_64`                 |
| [pipewire](pipewire/pipewire)       | `linux-x86_64`                 |
| [pulseaudio](pulseaudio/pulse)      | `linux-x86_64`                 |
| [rofi](rofi/rofi)                   | `linux-x86_64`                 |
| [sway](sway/sway)                   | `linux-x86_64`                 |
| [swayshot](swayshot)                | `linux-x86_64`                 |
| [sxiv](sxiv/sxiv/exec)              | `linux-x86_64`                 |
| [tint2](tint2/tint2)                | `linux-x86_64`                 |
| [tmux](tmux)                        | `linux-x86_64`                 |
| [waybar](waybar/waybar)             | `linux-x86_64`                 |
| [atom](atom)                        |                                |
| [bash](bash)                        |                                |
| [color](color)                      |                                |
| [commitizen](commitizen)            |                                |
| [conky](conky)                      |                                |
| [krita](krita)                      |                                |
| [psd](psd)                          |                                |
| [sakura](sakura)                    |                                |
| [stow](stow)                        |                                |

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
