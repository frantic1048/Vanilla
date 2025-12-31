mod config;
mod output;
mod package;
mod stow;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::{Path, PathBuf};

use config::get_profile;
use output::log;
use package::discover_packages;
use stow::{StowAction, execute_actions, stow_package, unstow_package};

#[derive(Parser)]
#[command(name = "blend", version, about = "Cross-platform dotfiles manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show what would be done without making changes
    #[arg(short = 'n', long, global = true)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Override home directory (for testing)
    #[arg(long, global = true)]
    home: Option<PathBuf>,

    /// Override packages directory (for testing)
    #[arg(long, global = true)]
    packages_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install package configurations
    Install {
        /// Packages to install (default: all for current profile)
        packages: Vec<String>,

        /// Override target directory (bypasses profile lookup)
        #[arg(long)]
        target: Option<PathBuf>,
    },
    /// Uninstall package configurations
    Uninstall {
        /// Packages to uninstall
        packages: Vec<String>,

        /// Override target directory (bypasses profile lookup)
        #[arg(long)]
        target: Option<PathBuf>,
    },
    /// Show status of all packages and profiles
    Stat,
    /// Initialize and verify configuration
    Init,
}

/// Runtime context
struct Context {
    home_dir: PathBuf,
    packages_dir: PathBuf,
    dry_run: bool,
    verbose: bool,
}

impl Context {
    fn new(cli: &Cli) -> Self {
        let home_dir = cli
            .home
            .clone()
            .unwrap_or_else(|| dirs::home_dir().expect("Could not determine home directory"));

        let packages_dir = cli.packages_dir.clone().unwrap_or_else(find_packages_dir);

        Self {
            home_dir,
            packages_dir,
            dry_run: cli.dry_run,
            verbose: cli.verbose,
        }
    }

    /// Expand ~ to home directory
    fn expand_path(&self, path: &Path) -> PathBuf {
        let path_str = path.to_string_lossy();
        if path_str.starts_with("~/") {
            self.home_dir.join(&path_str[2..])
        } else if path_str == "~" {
            self.home_dir.clone()
        } else {
            path.to_path_buf()
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let ctx = Context::new(&cli);

    if ctx.verbose {
        log::info(&format!("Home directory: {}", ctx.home_dir.display()));
        log::info(&format!(
            "Packages directory: {}",
            ctx.packages_dir.display()
        ));
    }

    match cli.command {
        Some(Commands::Install { packages, target }) => {
            cmd_install(&ctx, &packages, target.as_ref());
        }
        Some(Commands::Uninstall { packages, target }) => {
            cmd_uninstall(&ctx, &packages, target.as_ref());
        }
        Some(Commands::Stat) => {
            cmd_stat(&ctx);
        }
        Some(Commands::Init) | None => {
            cmd_init(&ctx);
        }
    }
}

fn find_packages_dir() -> PathBuf {
    // Find packages directory relative to executable or current directory
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    // Try relative to exe first, then current dir
    for base in [exe_dir, Some(std::env::current_dir().unwrap_or_default())]
        .into_iter()
        .flatten()
    {
        let packages_dir = base.join("packages");
        if packages_dir.is_dir() {
            return packages_dir;
        }
        // Also check parent (in case we're in blend-rs/)
        if let Some(parent) = base.parent() {
            let packages_dir = parent.join("packages");
            if packages_dir.is_dir() {
                return packages_dir;
            }
        }
    }

    // Default to current dir
    PathBuf::from("packages")
}

fn cmd_init(ctx: &Context) {
    log::info("Initializing blend...");

    if !ctx.packages_dir.is_dir() {
        log::error(&format!(
            "Packages directory not found: {}",
            ctx.packages_dir.display()
        ));
        std::process::exit(1);
    }

    let packages = discover_packages(&ctx.packages_dir);
    log::success(&format!("Found {} packages", packages.len()));

    let profile = get_profile();
    log::success(&format!("Profile: {}", profile.name));

    // Verify all packages in profile exist
    let mut errors = 0;
    for (pkg, _) in &profile.packages {
        if !packages.contains(pkg) {
            log::warn(&format!(
                "Package '{}' in profile but not found in packages/",
                pkg
            ));
            errors += 1;
        }
    }

    if errors > 0 {
        log::warn(&format!("{} packages in profile not found", errors));
    } else {
        log::success("All profile packages verified");
    }
}

fn cmd_stat(ctx: &Context) {
    let packages = discover_packages(&ctx.packages_dir);
    let profiles = config::all_profiles();

    println!(
        "{:<20} {}",
        "PACKAGE".bold(),
        profiles
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join("  ")
    );
    println!("{}", "-".repeat(60));

    let mut pkg_list: Vec<_> = packages.iter().collect();
    pkg_list.sort();

    for pkg in pkg_list {
        let indicators: Vec<String> = profiles
            .iter()
            .map(|profile| {
                if profile.packages.contains_key(pkg) {
                    format!("{}", "*".green())
                } else {
                    format!("{}", "-".dimmed())
                }
            })
            .collect();

        println!(
            "{:<20} {}",
            pkg.green(),
            indicators.join(&" ".repeat(profile_name_width(&profiles[0].name)))
        );
    }
}

fn profile_name_width(name: &str) -> usize {
    name.len().saturating_sub(1).max(1)
}

fn cmd_install(ctx: &Context, packages: &[String], target_override: Option<&PathBuf>) {
    let all_packages = discover_packages(&ctx.packages_dir);

    // If target is overridden, install specified packages to that target
    if let Some(target) = target_override {
        if packages.is_empty() {
            log::error("Must specify packages when using --target");
            std::process::exit(1);
        }
        for pkg in packages {
            if !all_packages.contains(pkg) {
                log::error(&format!("Package '{}' not found in packages/", pkg));
                continue;
            }
            install_package(ctx, pkg, target);
        }
        return;
    }

    let profile = get_profile();
    log::info(&format!("Installing with profile: {}", profile.name));

    let to_install: Vec<(&String, &Vec<PathBuf>)> = if packages.is_empty() {
        // Install all packages in profile
        profile.packages.iter().collect()
    } else {
        // Install specified packages
        packages
            .iter()
            .filter_map(|pkg| {
                profile.packages.get_key_value(pkg).or_else(|| {
                    log::warn(&format!("Package '{}' not in current profile", pkg));
                    None
                })
            })
            .collect()
    };

    for (pkg, targets) in to_install {
        if !all_packages.contains(pkg) {
            log::error(&format!("Package '{}' not found in packages/", pkg));
            continue;
        }

        for target in targets {
            install_package(ctx, pkg, target);
        }
    }
}

fn install_package(ctx: &Context, pkg: &str, target: &PathBuf) {
    let pkg_dir = ctx.packages_dir.join(pkg);
    let expanded_target = ctx.expand_path(target);

    log::info_important(&format!("Stowing {} -> {}", pkg, expanded_target.display()));

    // Ensure target directory exists
    if !ctx.dry_run && !expanded_target.exists() {
        if let Err(e) = std::fs::create_dir_all(&expanded_target) {
            log::error(&format!(
                "Failed to create target directory {}: {}",
                expanded_target.display(),
                e
            ));
            return;
        }
    }

    let ignore = stow::ignore::load_ignore_patterns(&ctx.packages_dir, pkg);
    let actions = stow_package(&pkg_dir, &expanded_target, &ignore);

    let (conflicts, safe): (Vec<_>, Vec<_>) = actions
        .into_iter()
        .partition(|a| matches!(a, StowAction::Conflict { .. }));

    if ctx.verbose || ctx.dry_run {
        for action in &safe {
            println!("  {}", action);
        }
    }

    if !conflicts.is_empty() {
        for conflict in &conflicts {
            log::warn(&format!("  {}", conflict));
        }
    }

    if !ctx.dry_run {
        execute_actions(safe);
    }
}

fn cmd_uninstall(ctx: &Context, packages: &[String], target_override: Option<&PathBuf>) {
    if packages.is_empty() {
        log::error("No packages specified for uninstall");
        std::process::exit(1);
    }

    // If target is overridden, uninstall from that target
    if let Some(target) = target_override {
        for pkg in packages {
            uninstall_package(ctx, pkg, target);
        }
        return;
    }

    let profile = get_profile();

    for pkg in packages {
        if let Some(targets) = profile.packages.get(pkg) {
            for target in targets {
                uninstall_package(ctx, pkg, target);
            }
        } else {
            log::warn(&format!("Package '{}' not in current profile", pkg));
        }
    }
}

fn uninstall_package(ctx: &Context, pkg: &str, target: &PathBuf) {
    let pkg_dir = ctx.packages_dir.join(pkg);
    let expanded_target = ctx.expand_path(target);

    log::info_important(&format!(
        "Unstowing {} from {}",
        pkg,
        expanded_target.display()
    ));

    let actions = unstow_package(&pkg_dir, &expanded_target);

    if ctx.verbose || ctx.dry_run {
        for action in &actions {
            println!("  {}", action);
        }
    }

    if !ctx.dry_run {
        execute_actions(actions);
    }
}
