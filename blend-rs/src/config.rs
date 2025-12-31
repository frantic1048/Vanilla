use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub packages: HashMap<String, Vec<PathBuf>>,
}

pub fn get_profile() -> Profile {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let profile_name = match (os, arch) {
        ("macos", "aarch64") => "macos-aarch64",
        ("macos", "x86_64") => "macos-x86_64",
        ("linux", "x86_64") => "linux-x86_64",
        _ => {
            eprintln!("Unsupported platform: {}-{}", os, arch);
            std::process::exit(1);
        }
    };

    all_profiles()
        .into_iter()
        .find(|p| p.name == profile_name)
        .unwrap_or_else(|| {
            eprintln!("Profile not found: {}", profile_name);
            std::process::exit(1);
        })
}

pub fn all_profiles() -> Vec<Profile> {
    vec![
        macos_profile("macos-aarch64"),
        macos_profile("macos-x86_64"),
        linux_profile(),
    ]
}

fn macos_profile(name: &str) -> Profile {
    let home = "~";
    let xdg_config = "~/.config";
    let app_support = "~/Library/Application Support";
    let preferences = "~/Library/Preferences";

    let mut packages: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Shells and friends
    packages.insert("elvish".into(), vec![xdg_config.into()]);
    packages.insert("zsh".into(), vec![home.into()]);
    packages.insert("bash".into(), vec![home.into()]);
    packages.insert("nushell".into(), vec![app_support.into()]);
    packages.insert("starship".into(), vec![xdg_config.into()]);
    packages.insert("pueue".into(), vec![app_support.into()]);
    packages.insert("neovim".into(), vec![xdg_config.into()]);

    // Terminal
    packages.insert("alacritty-macos".into(), vec![xdg_config.into()]);
    packages.insert("kitty".into(), vec![xdg_config.into()]);
    packages.insert("wezterm".into(), vec![xdg_config.into()]);
    packages.insert("ghostty".into(), vec![xdg_config.into()]);
    packages.insert("zellij-macos".into(), vec![xdg_config.into()]);
    packages.insert("tmux".into(), vec![home.into()]);

    // Desktop
    packages.insert("aerospace".into(), vec![xdg_config.into()]);
    packages.insert("yabai".into(), vec![xdg_config.into()]);
    packages.insert("skhd".into(), vec![xdg_config.into()]);
    packages.insert("sketchybar".into(), vec![xdg_config.into()]);

    // App
    packages.insert("bin".into(), vec![home.into()]);
    packages.insert("git".into(), vec![xdg_config.into()]);
    packages.insert("sapling".into(), vec![preferences.into()]);
    packages.insert("proto".into(), vec![home.into()]);
    packages.insert("mise".into(), vec![xdg_config.into()]);
    packages.insert(
        "vscode".into(),
        vec![
            format!("{}/Code", app_support).into(),
            format!("{}/code-oss-dev", app_support).into(),
            format!("{}/Cursor", app_support).into(),
        ],
    );
    packages.insert("fastfetch".into(), vec![xdg_config.into()]);
    packages.insert("ncdu".into(), vec![xdg_config.into()]);
    packages.insert("tealdeer".into(), vec![app_support.into()]);
    packages.insert("bat".into(), vec![xdg_config.into()]);
    packages.insert("gpg".into(), vec![home.into()]);

    Profile {
        name: name.into(),
        packages,
    }
}

fn linux_profile() -> Profile {
    let home = "~";
    let xdg_config = "~/.config";
    let local_share = "~/.local/share";

    let mut packages: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Shells and friends
    packages.insert("elvish".into(), vec![xdg_config.into()]);
    packages.insert("zsh".into(), vec![home.into()]);
    packages.insert("bash".into(), vec![home.into()]);
    packages.insert("nushell".into(), vec![xdg_config.into()]);
    packages.insert("starship".into(), vec![xdg_config.into()]);
    packages.insert("pueue".into(), vec![xdg_config.into()]);
    packages.insert("neovim".into(), vec![xdg_config.into()]);

    // Terminal
    packages.insert("alacritty".into(), vec![xdg_config.into()]);
    packages.insert("tmux".into(), vec![home.into()]);
    packages.insert("sakura".into(), vec![xdg_config.into()]);
    packages.insert("ghostty".into(), vec![xdg_config.into()]);

    // Desktop
    packages.insert("alsa".into(), vec![home.into()]);
    packages.insert("pipewire".into(), vec![xdg_config.into()]);
    packages.insert("pulseaudio".into(), vec![xdg_config.into()]);
    packages.insert("fontconfig".into(), vec![xdg_config.into()]);
    packages.insert("pam_env".into(), vec![home.into()]);

    // Desktop: X11
    packages.insert("X".into(), vec![home.into()]);
    packages.insert("i3wm".into(), vec![xdg_config.into()]);
    packages.insert("picom".into(), vec![xdg_config.into()]);
    packages.insert("tint2".into(), vec![xdg_config.into()]);

    // Desktop: Wayland
    packages.insert("sway".into(), vec![xdg_config.into()]);
    packages.insert("waybar".into(), vec![xdg_config.into()]);
    packages.insert("rofi".into(), vec![xdg_config.into()]);
    packages.insert("mako".into(), vec![xdg_config.into()]);

    // App
    packages.insert("bin".into(), vec![home.into()]);
    packages.insert("git".into(), vec![xdg_config.into()]);
    packages.insert("proto".into(), vec![home.into()]);
    packages.insert("sapling".into(), vec![xdg_config.into()]);
    packages.insert(
        "vscode".into(),
        vec![
            format!("{}/Code", xdg_config).into(),
            format!("{}/Code - OSS", xdg_config).into(),
        ],
    );
    packages.insert("htop".into(), vec![xdg_config.into()]);
    packages.insert("nano".into(), vec![xdg_config.into()]);
    packages.insert("sxiv".into(), vec![xdg_config.into()]);
    packages.insert("swayshot".into(), vec![xdg_config.into()]);
    packages.insert("fcitx".into(), vec![xdg_config.into()]);
    packages.insert("npm".into(), vec![home.into()]);
    packages.insert("makepkg".into(), vec![home.into()]);
    packages.insert("bat".into(), vec![xdg_config.into()]);
    packages.insert("gpg".into(), vec![home.into()]);
    packages.insert("neofetch".into(), vec![xdg_config.into()]);
    packages.insert("ncdu".into(), vec![xdg_config.into()]);
    packages.insert("tealdeer".into(), vec![xdg_config.into()]);
    packages.insert("color".into(), vec![local_share.into()]);

    Profile {
        name: "linux-x86_64".into(),
        packages,
    }
}
