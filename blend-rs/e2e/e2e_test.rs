use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the blend binary path
fn blend_binary() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("target")
        .join("debug")
        .join("blend")
}

/// Create a test fixture with packages directory
struct TestFixture {
    temp_dir: TempDir,
    home_dir: PathBuf,
    packages_dir: PathBuf,
}

impl TestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let home_dir = temp_dir.path().join("home");
        let packages_dir = temp_dir.path().join("packages");

        fs::create_dir_all(&home_dir).unwrap();
        fs::create_dir_all(&packages_dir).unwrap();

        // Create stow directory with global ignore
        let stow_dir = packages_dir.join("stow");
        fs::create_dir_all(&stow_dir).unwrap();
        fs::write(
            stow_dir.join(".stow-global-ignore"),
            "# Global ignore\n\\.git\n",
        )
        .unwrap();

        Self {
            temp_dir,
            home_dir,
            packages_dir,
        }
    }

    /// Create a package with files
    fn create_package(&self, name: &str, files: &[(&str, &str)]) {
        let pkg_dir = self.packages_dir.join(name);
        for (path, content) in files {
            let file_path = pkg_dir.join(path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&file_path, content).unwrap();
        }
    }

    /// Run blend command with packages dir
    fn run_blend(&self, args: &[&str]) -> std::process::Output {
        Command::new(blend_binary())
            .current_dir(self.temp_dir.path())
            .args(["--home", self.home_dir.to_str().unwrap()])
            .args(["--packages-dir", self.packages_dir.to_str().unwrap()])
            .args(args)
            .output()
            .expect("Failed to run blend")
    }

    /// Run blend install with explicit target
    fn run_install(&self, packages: &[&str], target: &str) -> std::process::Output {
        let target_path = if target.is_empty() {
            self.home_dir.clone()
        } else {
            self.home_dir.join(target)
        };

        Command::new(blend_binary())
            .current_dir(self.temp_dir.path())
            .args(["--home", self.home_dir.to_str().unwrap()])
            .args(["--packages-dir", self.packages_dir.to_str().unwrap()])
            .arg("install")
            .args(packages)
            .args(["--target", target_path.to_str().unwrap()])
            .output()
            .expect("Failed to run blend")
    }

    /// Run blend uninstall with explicit target
    fn run_uninstall(&self, packages: &[&str], target: &str) -> std::process::Output {
        let target_path = if target.is_empty() {
            self.home_dir.clone()
        } else {
            self.home_dir.join(target)
        };

        Command::new(blend_binary())
            .current_dir(self.temp_dir.path())
            .args(["--home", self.home_dir.to_str().unwrap()])
            .args(["--packages-dir", self.packages_dir.to_str().unwrap()])
            .arg("uninstall")
            .args(packages)
            .args(["--target", target_path.to_str().unwrap()])
            .output()
            .expect("Failed to run blend")
    }

    /// Check if path is a symlink pointing to expected target
    fn assert_symlink(&self, path: &str, expected_target_contains: &str) {
        let full_path = self.home_dir.join(path);
        assert!(
            full_path.is_symlink(),
            "Expected {} to be a symlink",
            full_path.display()
        );
        let target = fs::read_link(&full_path).unwrap();
        assert!(
            target.to_string_lossy().contains(expected_target_contains),
            "Expected symlink {} to point to something containing '{}', but got '{}'",
            full_path.display(),
            expected_target_contains,
            target.display()
        );
    }

    /// Check if path exists and is a regular file
    fn assert_file(&self, path: &str, expected_content: &str) {
        let full_path = self.home_dir.join(path);
        assert!(full_path.exists(), "Expected {} to exist", full_path.display());
        let content = fs::read_to_string(&full_path).unwrap();
        assert_eq!(
            content, expected_content,
            "File content mismatch for {}",
            full_path.display()
        );
    }

    /// Check if path does not exist
    fn assert_not_exists(&self, path: &str) {
        let full_path = self.home_dir.join(path);
        assert!(
            !full_path.exists() && !full_path.is_symlink(),
            "Expected {} to not exist",
            full_path.display()
        );
    }

    /// Check if path exists as a directory (not symlink)
    fn assert_dir(&self, path: &str) {
        let full_path = self.home_dir.join(path);
        assert!(
            full_path.is_dir() && !full_path.is_symlink(),
            "Expected {} to be a real directory",
            full_path.display()
        );
    }
}

// ============================================================================
// Test: Basic Tree Folding
// When target doesn't exist, symlink entire directory
// ============================================================================

#[test]
fn test_tree_folding_single_dir() {
    let fixture = TestFixture::new();

    // Create package: bat/bat/config
    fixture.create_package("bat", &[("bat/config", "--theme=gruvbox")]);

    // Create .config directory
    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    // Install
    let output = fixture.run_install(&["bat"], ".config");
    assert!(output.status.success(), "blend install failed: {:?}", output);

    // Should create symlink at .config/bat (tree folding)
    fixture.assert_symlink(".config/bat", "packages/bat/bat");

    // Content should be accessible
    fixture.assert_file(".config/bat/config", "--theme=gruvbox");
}

#[test]
fn test_tree_folding_nested_dirs() {
    let fixture = TestFixture::new();

    // Create package with nested structure: neovim/nvim/lua/plugins/init.lua
    fixture.create_package(
        "neovim",
        &[
            ("nvim/init.lua", "-- init"),
            ("nvim/lua/config/options.lua", "-- options"),
            ("nvim/lua/plugins/lazy.lua", "-- lazy"),
        ],
    );

    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    let output = fixture.run_install(&["neovim"], ".config");
    assert!(output.status.success());

    // Should symlink entire nvim directory
    fixture.assert_symlink(".config/nvim", "packages/neovim/nvim");

    // All files accessible
    fixture.assert_file(".config/nvim/init.lua", "-- init");
    fixture.assert_file(".config/nvim/lua/config/options.lua", "-- options");
}

// ============================================================================
// Test: Unfolding
// When target directory exists, create individual symlinks
// ============================================================================

#[test]
fn test_unfold_existing_directory() {
    let fixture = TestFixture::new();

    // Create package
    fixture.create_package(
        "neovim",
        &[
            ("nvim/init.lua", "-- init"),
            ("nvim/lua/plugins.lua", "-- plugins"),
        ],
    );

    // Pre-create target directory with existing file
    let nvim_dir = fixture.home_dir.join(".config/nvim");
    fs::create_dir_all(&nvim_dir).unwrap();
    fs::write(nvim_dir.join("existing.lua"), "-- existing").unwrap();

    let output = fixture.run_install(&["neovim"], ".config");
    assert!(output.status.success());

    // nvim should be a real directory, not symlink
    fixture.assert_dir(".config/nvim");

    // Existing file preserved
    fixture.assert_file(".config/nvim/existing.lua", "-- existing");

    // Package files symlinked individually
    fixture.assert_symlink(".config/nvim/init.lua", "nvim/init.lua");
    fixture.assert_symlink(".config/nvim/lua", "nvim/lua"); // lua dir folded
}

#[test]
fn test_unfold_nested_existing_directory() {
    let fixture = TestFixture::new();

    // Create package with nested dirs
    fixture.create_package(
        "neovim",
        &[
            ("nvim/lua/config/a.lua", "-- a"),
            ("nvim/lua/config/b.lua", "-- b"),
        ],
    );

    // Pre-create nested directory with file
    let config_dir = fixture.home_dir.join(".config/nvim/lua/config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("existing.lua"), "-- existing").unwrap();

    let output = fixture.run_install(&["neovim"], ".config");
    assert!(output.status.success());

    // All intermediate dirs should be real directories
    fixture.assert_dir(".config/nvim");
    fixture.assert_dir(".config/nvim/lua");
    fixture.assert_dir(".config/nvim/lua/config");

    // Existing file preserved
    fixture.assert_file(".config/nvim/lua/config/existing.lua", "-- existing");

    // Package files symlinked
    fixture.assert_symlink(".config/nvim/lua/config/a.lua", "config/a.lua");
    fixture.assert_symlink(".config/nvim/lua/config/b.lua", "config/b.lua");
}

// ============================================================================
// Test: Conflict Detection
// ============================================================================

#[test]
fn test_conflict_existing_file() {
    let fixture = TestFixture::new();

    // Create package
    fixture.create_package("bat", &[("bat/config", "--theme=gruvbox")]);

    // Pre-create conflicting file
    let bat_dir = fixture.home_dir.join(".config/bat");
    fs::create_dir_all(&bat_dir).unwrap();
    fs::write(bat_dir.join("config"), "-- existing config").unwrap();

    let output = fixture.run_install(&["bat", "-v"], ".config");

    // Should report conflict
    let stderr = String::from_utf8_lossy(&output.stdout);
    assert!(
        stderr.contains("CONFLICT") || stderr.contains("conflict"),
        "Expected conflict warning, got: {}",
        stderr
    );

    // Existing file should be preserved
    fixture.assert_file(".config/bat/config", "-- existing config");
}

#[test]
fn test_conflict_existing_symlink_different_target() {
    let fixture = TestFixture::new();

    // Create package
    fixture.create_package("bat", &[("bat/config", "--theme=gruvbox")]);

    // Pre-create symlink pointing elsewhere
    let bat_dir = fixture.home_dir.join(".config/bat");
    fs::create_dir_all(&bat_dir).unwrap();
    let config_path = bat_dir.join("config");
    fs::write(fixture.temp_dir.path().join("other_config"), "other").unwrap();
    symlink(fixture.temp_dir.path().join("other_config"), &config_path).unwrap();

    let output = fixture.run_install(&["bat", "-v"], ".config");

    // Should report conflict
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CONFLICT") || stdout.contains("conflict"),
        "Expected conflict warning"
    );
}

// ============================================================================
// Test: Idempotent Install
// Running install twice should be safe
// ============================================================================

#[test]
fn test_idempotent_install() {
    let fixture = TestFixture::new();

    fixture.create_package("bat", &[("bat/config", "--theme=gruvbox")]);
    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    // First install
    let output1 = fixture.run_install(&["bat"], ".config");
    assert!(output1.status.success());

    // Second install should also succeed
    let output2 = fixture.run_install(&["bat"], ".config");
    assert!(output2.status.success());

    // Symlink still valid
    fixture.assert_symlink(".config/bat", "packages/bat/bat");
}

// ============================================================================
// Test: Uninstall
// ============================================================================

#[test]
fn test_uninstall_removes_symlink() {
    let fixture = TestFixture::new();

    fixture.create_package("bat", &[("bat/config", "--theme=gruvbox")]);
    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    // Install
    fixture.run_install(&["bat"], ".config");
    fixture.assert_symlink(".config/bat", "packages/bat/bat");

    // Uninstall
    let output = fixture.run_uninstall(&["bat"], ".config");
    assert!(output.status.success());

    // Symlink should be removed
    fixture.assert_not_exists(".config/bat");
}

#[test]
fn test_uninstall_individual_symlinks() {
    let fixture = TestFixture::new();

    fixture.create_package("neovim", &[("nvim/init.lua", "-- init")]);

    // Pre-create nvim dir to force unfolding
    let nvim_dir = fixture.home_dir.join(".config/nvim");
    fs::create_dir_all(&nvim_dir).unwrap();
    fs::write(nvim_dir.join("existing.lua"), "-- existing").unwrap();

    // Install (will unfold)
    fixture.run_install(&["neovim"], ".config");
    fixture.assert_symlink(".config/nvim/init.lua", "init.lua");

    // Uninstall
    let output = fixture.run_uninstall(&["neovim"], ".config");
    assert!(output.status.success());

    // Package symlink removed
    fixture.assert_not_exists(".config/nvim/init.lua");

    // Existing file preserved
    fixture.assert_file(".config/nvim/existing.lua", "-- existing");
}

// ============================================================================
// Test: Ignore Patterns
// ============================================================================

#[test]
fn test_ignore_ds_store() {
    let fixture = TestFixture::new();

    fixture.create_package(
        "bat",
        &[
            ("bat/config", "--theme=gruvbox"),
            ("bat/.DS_Store", "binary junk"),
        ],
    );

    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    let output = fixture.run_install(&["bat", "-v"], ".config");
    assert!(output.status.success());

    // .DS_Store should not be linked
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(".DS_Store"),
        "Should not link .DS_Store"
    );
}

#[test]
fn test_ignore_stow_local_ignore() {
    let fixture = TestFixture::new();

    fixture.create_package(
        "myapp",
        &[
            ("myapp/config", "config content"),
            ("myapp/.stow-local-ignore", "ignored"),
        ],
    );

    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    let output = fixture.run_install(&["myapp", "-v"], ".config");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(".stow-local-ignore"),
        "Should not link .stow-local-ignore"
    );
}

#[test]
fn test_local_ignore_patterns() {
    let fixture = TestFixture::new();

    // Create package with local ignore file
    fixture.create_package(
        "myapp",
        &[
            ("myapp/config", "config content"),
            ("myapp/cache.txt", "cached data"),
            (".stow-local-ignore", "cache\\.txt"),
        ],
    );

    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    let output = fixture.run_install(&["myapp", "-v"], ".config");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // cache.txt should be ignored by local ignore pattern
    assert!(
        !stdout.contains("cache.txt"),
        "cache.txt should be ignored"
    );
}

// ============================================================================
// Test: Multiple Packages
// ============================================================================

#[test]
fn test_install_multiple_packages() {
    let fixture = TestFixture::new();

    fixture.create_package("bat", &[("bat/config", "bat config")]);
    fixture.create_package("git", &[("git/config", "git config")]);
    fixture.create_package("neovim", &[("nvim/init.lua", "nvim init")]);

    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    let output = fixture.run_install(&["bat", "git", "neovim"], ".config");
    assert!(output.status.success());

    fixture.assert_symlink(".config/bat", "packages/bat/bat");
    fixture.assert_symlink(".config/git", "packages/git/git");
    fixture.assert_symlink(".config/nvim", "packages/neovim/nvim");
}

// ============================================================================
// Test: Dry Run
// ============================================================================

#[test]
fn test_dry_run_no_changes() {
    let fixture = TestFixture::new();

    fixture.create_package("bat", &[("bat/config", "--theme=gruvbox")]);
    fs::create_dir_all(fixture.home_dir.join(".config")).unwrap();

    let output = fixture.run_install(&["bat", "--dry-run", "-v"], ".config");
    assert!(output.status.success());

    // Should show what would be done
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("LINK"), "Should show LINK action");

    // But symlink should NOT be created
    fixture.assert_not_exists(".config/bat");
}

// ============================================================================
// Test: Home Directory Files (dotfiles in home root)
// ============================================================================

#[test]
fn test_dotfiles_in_home_root() {
    let fixture = TestFixture::new();

    // Package that installs to home root (like zsh)
    fixture.create_package(
        "zsh",
        &[
            (".zshrc", "# zshrc content"),
            (".zshenv", "# zshenv content"),
        ],
    );

    // Install to home root (empty string means home_dir itself)
    let output = fixture.run_install(&["zsh"], "");
    assert!(output.status.success());

    fixture.assert_symlink(".zshrc", "packages/zsh/.zshrc");
    fixture.assert_symlink(".zshenv", "packages/zsh/.zshenv");
}

// ============================================================================
// Test: Stat Command
// ============================================================================

#[test]
fn test_stat_command() {
    let fixture = TestFixture::new();

    fixture.create_package("bat", &[("bat/config", "content")]);
    fixture.create_package("git", &[("git/config", "content")]);

    let output = fixture.run_blend(&["stat"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PACKAGE"), "Should have header");
    assert!(stdout.contains("bat"), "Should list bat package");
    assert!(stdout.contains("git"), "Should list git package");
}

// ============================================================================
// Test: Init Command
// ============================================================================

#[test]
fn test_init_command() {
    let fixture = TestFixture::new();

    fixture.create_package("bat", &[("bat/config", "content")]);

    let output = fixture.run_blend(&["init"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Found") && stdout.contains("package"),
        "Should report found packages"
    );
}
