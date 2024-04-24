# Vanilla

frantic1048's daily configs, scripts etc.

## Screenshots

_Sway_:

![toastx900](screenshots/toastx900_2021-07-30_13-00.png)
![toastx900](screenshots/toastx900_2021-07-30_13-14.png)

_i3_:

![amausaan](screenshots/amausaan_2022-04-05-232523.png)

## Config packages

| package                                         | profiles                                        |
| ----------------------------------------------- | ----------------------------------------------- |
| [alacritty](alacritty)                          | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [bin](bin/bin)                                  | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [elvish](elvish/elvish)                         | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [git](git/git)                                  | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [ncdu](ncdu/ncdu/config)                        | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [neofetch](neofetch/neofetch/config.conf)       | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [nushell](nushell/nushell)                      | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [pueue](pueue/pueue/pueue.yml)                  | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [sapling](sapling/sapling/sapling.conf)         | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [starship](starship/starship.toml)              | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [tealdeer](tealdeer/tealdeer/config.toml)       | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [vscode](vscode/User)                           | `macos-x86_64`, `macos-aarch64`, `linux-x86_64` |
| [aerospace](aerospace/aerospace/aerospace.toml) | `macos-x86_64`, `macos-aarch64`                 |
| [kitty](kitty/kitty)                            | `macos-x86_64`, `macos-aarch64`                 |
| [proto](proto)                                  | `macos-x86_64`, `macos-aarch64`                 |
| [sketchybar](sketchybar/sketchybar)             | `macos-x86_64`, `macos-aarch64`                 |
| [skhd](skhd/skhd/skhdrc)                        | `macos-x86_64`, `macos-aarch64`                 |
| [wezterm](wezterm)                              | `macos-x86_64`, `macos-aarch64`                 |
| [yabai](yabai/yabai)                            | `macos-x86_64`, `macos-aarch64`                 |
| [X](X)                                          | `linux-x86_64`                                  |
| [alsa](alsa)                                    | `linux-x86_64`                                  |
| [fcitx](fcitx/fcitx)                            | `linux-x86_64`                                  |
| [fontconfig](fontconfig/fontconfig)             | `linux-x86_64`                                  |
| [htop](htop/htop/htoprc)                        | `linux-x86_64`                                  |
| [i3wm](i3wm/i3/config)                          | `linux-x86_64`                                  |
| [makepkg](makepkg)                              | `linux-x86_64`                                  |
| [mako](mako/mako/config)                        | `linux-x86_64`                                  |
| [nano](nano/nano/nanorc)                        | `linux-x86_64`                                  |
| [npm](npm)                                      | `linux-x86_64`                                  |
| [pam_env](pam_env)                              | `linux-x86_64`                                  |
| [picom](picom/picom/picom.conf)                 | `linux-x86_64`                                  |
| [pipewire](pipewire/pipewire)                   | `linux-x86_64`                                  |
| [pulseaudio](pulseaudio/pulse)                  | `linux-x86_64`                                  |
| [rofi](rofi/rofi/config.rasi)                   | `linux-x86_64`                                  |
| [sakura](sakura/sakura/sakura.conf)             | `linux-x86_64`                                  |
| [sway](sway/sway/config)                        | `linux-x86_64`                                  |
| [swayshot](swayshot/swayshot.sh)                | `linux-x86_64`                                  |
| [sxiv](sxiv/sxiv/exec/image-info)               | `linux-x86_64`                                  |
| [tint2](tint2/tint2/tint2rc)                    | `linux-x86_64`                                  |
| [tmux](tmux)                                    | `linux-x86_64`                                  |
| [waybar](waybar/waybar)                         | `linux-x86_64`                                  |
| [atom](atom)                                    |                                                 |
| [bash](bash)                                    |                                                 |
| [color](color)                                  |                                                 |
| [commitizen](commitizen)                        |                                                 |
| [conky](conky)                                  |                                                 |
| [krita](krita)                                  |                                                 |
| [psd](psd)                                      |                                                 |
| [stow](stow)                                    |                                                 |

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
