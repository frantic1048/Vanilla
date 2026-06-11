//! E2E tests for blend sync using real fixtures and temporary home directories.
//!
//! These tests exercise the full sync flow (build → diff → forced source-to-target
//! and target-to-source modes) without interactive prompts.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn blend_binary() -> PathBuf {
    // Use the debug binary from cargo build
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("blend");
    path
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Run blend with given args, using a temp home and the given blend directory.
fn run_blend(home: &Path, blend_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(blend_binary())
        .args(args)
        .arg("--home")
        .arg(home)
        .arg("--blend-dir")
        .arg(blend_dir)
        .output()
        .expect("Failed to execute blend")
}

fn run_blend_in_cwd(home: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(blend_binary())
        .args(args)
        .arg("--home")
        .arg(home)
        .current_dir(cwd)
        .output()
        .expect("Failed to execute blend")
}

fn run_blend_with_env(
    home: &Path,
    blend_dir: &Path,
    args: &[&str],
    envs: &[(&str, &str)],
) -> std::process::Output {
    let mut command = Command::new(blend_binary());
    command
        .args(args)
        .arg("--home")
        .arg(home)
        .arg("--blend-dir")
        .arg(blend_dir);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("Failed to execute blend")
}

fn orders_dir(blend_dir: &Path) -> PathBuf {
    blend_dir.join("orders")
}

/// Copy a single fixture order to a temporary blend directory, along with
/// the shared `order.contract.ncl` and `metadata.ncl` files that every order
/// implicitly depends on for evaluation.
/// Returns the TempDir (which owns the temp path) — the orders dir is at
/// temp.path()/orders.
/// Needed for forced target-to-source tests that modify source .ncl files.
fn copy_fixture(order_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();
    let orders_src = fixtures_dir().join("orders");
    let orders_dst = orders_dir(temp.path());
    std::fs::create_dir_all(&orders_dst).unwrap();

    // Copy the order itself.
    let order_src = orders_src.join(order_name);
    let order_dst = orders_dst.join(order_name);
    copy_dir_recursive(&order_src, &order_dst);

    // Copy the two blend-owned schema files so reader commands don't fail
    // their freshness check and so metadata-importing fixtures resolve.
    for shared in ["order.contract.ncl", "metadata.ncl"] {
        std::fs::copy(orders_src.join(shared), orders_dst.join(shared)).unwrap();
    }

    temp
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

#[test]
fn test_sandbox_never_ignores_debug_probe() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend_with_env(
        home.path(),
        &orders,
        &["--sandbox", "never", "view", "--short", "toml-basic"],
        &[("BLEND_SANDBOX_PROBE", "exec")],
    );

    assert!(
        output.status.success(),
        "--sandbox never should skip sandbox probe\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn test_sandbox_force_exec_probe() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend_with_env(
        home.path(),
        &orders,
        &["--sandbox", "force", "view", "--short", "toml-basic"],
        &[("BLEND_SANDBOX_PROBE", "exec")],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        return;
    }

    assert!(
        stderr.contains("failed to enable process sandbox"),
        "force mode should either enforce the exec probe or fail before work when sandbox is unavailable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
}

#[test]
fn test_check_order_success() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(home.path(), &orders, &["check", "toml-basic"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend check failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Checked 1 order(s)"));
}

#[test]
fn test_check_order_fails_when_from_file_is_missing() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("plaintext-single");
    std::fs::remove_file(orders_dir(blend_dir.path()).join("plaintext-single/config.txt")).unwrap();

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["check", "plaintext-single"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "blend check should fail for missing from_file\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stderr.contains("source file not found"),
        "missing from_file error should mention the missing source\nstdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn test_format_check_order_success() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(home.path(), &orders, &["format", "--check", "toml-basic"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend format --check failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Checked formatting for 1 order.ncl file(s)"));
}

#[test]
fn test_format_order_writes_changes() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("toml-basic");
    let order_path = orders_dir(blend_dir.path()).join("toml-basic/order.ncl");
    let compact = r#"{ blend = { prefix = ["~/.config/toml-basic/"], files = [{ name = "config.toml", from_config = { key = "value", number = 42, nested = { inner = true, }, }, }], }, }"#;
    std::fs::write(&order_path, compact).unwrap();

    let output = run_blend(home.path(), blend_dir.path(), &["format", "toml-basic"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend format failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Formatted 1 order.ncl file(s)"));

    let formatted = std::fs::read_to_string(&order_path).unwrap();
    assert_ne!(formatted, compact);
    assert!(formatted.contains("files = ["));

    let check = run_blend(
        home.path(),
        blend_dir.path(),
        &["--sandbox", "never", "format", "--check", "toml-basic"],
    );
    assert!(
        check.status.success(),
        "formatted order should pass format --check\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn test_sync_force_source_to_target_plain_data_new_file() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Config file doesn't exist yet — sync --force-source-to-target should create it
    let target = home.path().join(".config/toml-basic/config.toml");
    assert!(!target.exists());

    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend sync --force-source-to-target failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // File should now exist
    assert!(target.exists(), "Config file should have been created");

    let content = std::fs::read_to_string(&target).unwrap();
    assert!(content.contains("key"), "Config should contain 'key'");
    assert!(content.contains("42"), "Config should contain '42'");
}

#[test]
fn test_sync_force_source_to_target_then_no_changes() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // First forced source-to-target
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );

    // Second sync should show no changes (nothing to do)
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "-v", "toml-basic"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("in sync") || stdout.contains("0 Source -> Target"),
        "Should be in sync after forced source-to-target, got: {stdout}"
    );
}

#[test]
fn test_sync_dry_run_no_changes() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Dry run should not create any files
    let target = home.path().join(".config/toml-basic/config.toml");
    let output = run_blend(home.path(), &orders, &["sync", "-n", "toml-basic"]);

    assert!(output.status.success());
    assert!(!target.exists(), "Dry run should not create files");
}

#[test]
fn test_sync_force_source_to_target_from_file() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let target = home.path().join(".config/plaintext-single/config.txt");
    assert!(!target.exists());

    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );
    assert!(output.status.success());
    assert!(target.exists());

    let content = std::fs::read_to_string(&target).unwrap();
    assert!(content.contains("original content from repo"));
}

#[test]
fn test_sync_force_target_to_source_from_file() {
    let home = TempDir::new().unwrap();

    // Copy fixtures to a temp location so we can modify the orders dir
    let temp_orders = TempDir::new().unwrap();
    let blend_dir = temp_orders.path();
    let orders = orders_dir(blend_dir);

    // Copy the test-file fixture
    let order_dir = orders.join("plaintext-single");
    std::fs::create_dir_all(&order_dir).unwrap();
    std::fs::copy(
        fixtures_dir().join("orders/plaintext-single/order.ncl"),
        order_dir.join("order.ncl"),
    )
    .unwrap();
    std::fs::copy(
        fixtures_dir().join("orders/plaintext-single/config.txt"),
        order_dir.join("config.txt"),
    )
    .unwrap();

    // First forced source-to-target to deploy
    run_blend(
        home.path(),
        blend_dir,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    assert!(target.exists());

    // Modify the deployed file
    std::fs::write(&target, "modified by user\nnew line\n").unwrap();

    // Forced target-to-source back
    let output = run_blend(
        home.path(),
        blend_dir,
        &["sync", "--force-target-to-source", "plaintext-single"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Forced target-to-source failed: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Source file in orders should now have the deployed content
    let source_content = std::fs::read_to_string(order_dir.join("config.txt")).unwrap();
    assert_eq!(source_content, "modified by user\nnew line\n");
}

#[test]
fn test_view_shows_diffs() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Forced source-to-target first
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );

    // View should show no changes
    let output = run_blend(home.path(), &orders, &["view", "toml-basic"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("no changes"),
        "Should show no changes: {stdout}"
    );
}

#[test]
fn test_status_shows_orders() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(home.path(), &orders, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("toml-basic"),
        "Should list test-plain order"
    );
    assert!(
        stdout.contains("plaintext-single"),
        "Should list test-file order"
    );
}

#[test]
fn test_status_shows_order_when_first_file_entry_is_skipped() {
    let home = TempDir::new().unwrap();
    let temp = TempDir::new().unwrap();
    let orders = orders_dir(temp.path());
    std::fs::create_dir_all(&orders).unwrap();

    for shared in ["order.contract.ncl", "metadata.ncl"] {
        std::fs::copy(
            fixtures_dir().join("orders").join(shared),
            orders.join(shared),
        )
        .unwrap();
    }

    let order_dir = orders.join("status-first-hidden");
    std::fs::create_dir_all(&order_dir).unwrap();
    std::fs::write(order_dir.join("shown"), "shown\n").unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        r#"let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/status-first-hidden/"],
    files = [
      {
        name = "skipped.toml",
        from_config = { value = 1 },
        when = { os = ["definitely-not-current"] },
      },
      {
        from_file = "shown",
      },
    ],
  },
} | Order
"#,
    )
    .unwrap();

    let output = run_blend(home.path(), temp.path(), &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "status failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("status-first-hidden") && stdout.contains("shown"),
        "status should print the order name on the first visible row:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// Match conditional tests
// ---------------------------------------------------------------------------

#[test]
fn test_sync_force_source_to_target_match_conditional() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let target = home.path().join(".config/os-match/config.toml");
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "os-match"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sync --force-source-to-target test-match failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(target.exists(), "Config file should have been created");

    let content = std::fs::read_to_string(&target).unwrap();

    // Should contain the platform-appropriate font_size
    let expected_font_size = match std::env::consts::OS {
        "macos" => "14",
        "linux" => "12",
        _ => "10",
    };
    assert!(
        content.contains(expected_font_size),
        "Should contain font_size = {expected_font_size} for this platform, got:\n{content}"
    );
    // Static value should always be present
    assert!(
        content.contains("catppuccin"),
        "Should contain theme = catppuccin, got:\n{content}"
    );
}

#[test]
fn test_sync_force_target_to_source_from_config_match_branch() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("os-match");
    let orders = temp_orders.path();

    // Forced source-to-target first
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "os-match"],
    );
    assert!(
        output.status.success(),
        "Initial forced source-to-target failed"
    );

    let target = home.path().join(".config/os-match/config.toml");
    assert!(target.exists());

    // Read and modify the deployed file — change font_size to 20
    let content = std::fs::read_to_string(&target).unwrap();
    let modified = content
        .lines()
        .map(|line| {
            if line.starts_with("font_size") {
                "font_size = 20"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&target, &modified).unwrap();

    // Forced target-to-source back
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-target-to-source", "os-match"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Forced target-to-source failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Read the modified order.ncl
    let ncl_content =
        std::fs::read_to_string(orders_dir(orders).join("os-match/order.ncl")).unwrap();

    // The active branch should have been updated to 20
    let active_branch = match std::env::consts::OS {
        "macos" => "\"darwin\" => 20",
        "linux" => "\"linux\" => 20",
        _ => "_ => 20",
    };
    assert!(
        ncl_content.contains(active_branch),
        "Active branch should be updated to 20.\nExpected to find: {active_branch}\nGot:\n{ncl_content}"
    );

    // Other branches should be untouched
    match std::env::consts::OS {
        "macos" => {
            assert!(
                ncl_content.contains("\"linux\" => 12"),
                "Linux branch should be untouched"
            );
            assert!(
                ncl_content.contains("_ => 10"),
                "Wildcard branch should be untouched"
            );
        }
        "linux" => {
            assert!(
                ncl_content.contains("\"darwin\" => 14"),
                "Darwin branch should be untouched"
            );
            assert!(
                ncl_content.contains("_ => 10"),
                "Wildcard branch should be untouched"
            );
        }
        _ => {
            assert!(
                ncl_content.contains("\"darwin\" => 14"),
                "Darwin branch should be untouched"
            );
            assert!(
                ncl_content.contains("\"linux\" => 12"),
                "Linux branch should be untouched"
            );
        }
    }

    // Re-run sync — should show no changes (round-trip correctness)
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "-v", "os-match"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("in sync") || stdout.contains("0 Source -> Target"),
        "Should be in sync after forced target-to-source round-trip, got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// If-then-else conditional tests
// ---------------------------------------------------------------------------

#[test]
fn test_sync_force_source_to_target_if_then_else() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let target = home.path().join(".config/if-then-else/config.toml");
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "if-then-else"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sync --force-source-to-target test-if failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(target.exists());

    let content = std::fs::read_to_string(&target).unwrap();
    let expected_gpu = match std::env::consts::OS {
        "macos" => "true",
        _ => "false",
    };
    assert!(
        content.contains(expected_gpu),
        "Should contain use_gpu = {expected_gpu}, got:\n{content}"
    );
    assert!(content.contains("test"), "Should contain label = test");
}

#[test]
fn test_sync_force_target_to_source_if_then_else_branch() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("if-then-else");
    let orders = temp_orders.path();

    // Forced source-to-target first
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "if-then-else"],
    );
    assert!(
        output.status.success(),
        "Initial forced source-to-target failed"
    );

    let target = home.path().join(".config/if-then-else/config.toml");

    // Flip the boolean value in deployed file
    let content = std::fs::read_to_string(&target).unwrap();
    let modified = content
        .lines()
        .map(|line| {
            if line.starts_with("use_gpu") {
                if line.contains("true") {
                    "use_gpu = false"
                } else {
                    "use_gpu = true"
                }
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&target, &modified).unwrap();

    // Forced target-to-source back
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-target-to-source", "if-then-else"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Forced target-to-source failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Read modified order.ncl — the active branch should have flipped
    let ncl_content =
        std::fs::read_to_string(orders_dir(orders).join("if-then-else/order.ncl")).unwrap();

    match std::env::consts::OS {
        "macos" => {
            // then branch should now be false (was true)
            assert!(
                ncl_content.contains("then false"),
                "then-branch should be flipped to false:\n{ncl_content}"
            );
            // else branch should be untouched
            assert!(
                ncl_content.contains("else false"),
                "else-branch should be untouched:\n{ncl_content}"
            );
        }
        _ => {
            // else branch should now be true (was false)
            assert!(
                ncl_content.contains("else true"),
                "else-branch should be flipped to true:\n{ncl_content}"
            );
            // then branch should be untouched
            assert!(
                ncl_content.contains("then true"),
                "then-branch should be untouched:\n{ncl_content}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Multi-file order tests
// ---------------------------------------------------------------------------

#[test]
fn test_sync_force_source_to_target_multi_file_order() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let toml_target = home.path().join(".config/mixed-entries/config.toml");
    let txt_target = home.path().join(".config/mixed-entries/extra.txt");

    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "mixed-entries"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sync --force-source-to-target test-multi failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    assert!(toml_target.exists(), "config.toml should be deployed");
    assert!(txt_target.exists(), "extra.txt should be deployed");

    let toml_content = std::fs::read_to_string(&toml_target).unwrap();
    assert!(toml_content.contains("dark"), "TOML should contain theme");
    assert!(toml_content.contains("14"), "TOML should contain font_size");

    let txt_content = std::fs::read_to_string(&txt_target).unwrap();
    assert!(
        txt_content.contains("extra file content"),
        "Text file should have original content"
    );
}

#[test]
fn test_sync_force_target_to_source_multi_selective() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("mixed-entries");
    let orders = temp_orders.path();

    // Forced source-to-target both files
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "mixed-entries"],
    );
    assert!(output.status.success());

    let txt_target = home.path().join(".config/mixed-entries/extra.txt");

    // Save original order.ncl for comparison
    let original_ncl =
        std::fs::read_to_string(orders_dir(orders).join("mixed-entries/order.ncl")).unwrap();

    // Modify only the text file
    std::fs::write(&txt_target, "modified extra content\n").unwrap();

    // Forced target-to-source back
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-target-to-source", "mixed-entries"],
    );
    assert!(output.status.success());

    // extra.txt source should be updated
    let pulled_txt =
        std::fs::read_to_string(orders_dir(orders).join("mixed-entries/extra.txt")).unwrap();
    assert_eq!(pulled_txt, "modified extra content\n");

    // order.ncl should be unchanged (only the from_file was modified)
    let current_ncl =
        std::fs::read_to_string(orders_dir(orders).join("mixed-entries/order.ncl")).unwrap();
    assert_eq!(
        current_ncl, original_ncl,
        "order.ncl should not change when only from_file was modified"
    );
}

// ---------------------------------------------------------------------------
// JSON format tests
// ---------------------------------------------------------------------------

#[test]
fn test_sync_force_source_to_target_json_format() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let target = home.path().join(".config/json-format/settings.json");
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "json-format"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sync --force-source-to-target test-json failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(target.exists(), "JSON file should be deployed");

    let content = std::fs::read_to_string(&target).unwrap();
    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Deployed file should be valid JSON");
    assert_eq!(parsed["fontSize"], 14);
    assert_eq!(parsed["tabSize"], 2);
    assert_eq!(parsed["fontFamily"], "JetBrains Mono");
}

#[test]
fn test_sync_force_target_to_source_json_format() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("json-format");
    let orders = temp_orders.path();

    // Forced source-to-target first
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "json-format"],
    );
    assert!(output.status.success());

    let target = home.path().join(".config/json-format/settings.json");

    // Modify deployed JSON
    let mut parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&target).unwrap()).unwrap();
    parsed["fontSize"] = serde_json::json!(16);
    std::fs::write(&target, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();

    // Forced target-to-source back
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-target-to-source", "json-format"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Forced target-to-source failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // order.ncl should have the updated value
    let ncl_content =
        std::fs::read_to_string(orders_dir(orders).join("json-format/order.ncl")).unwrap();
    assert!(
        ncl_content.contains("16"),
        "order.ncl should have updated fontSize to 16:\n{ncl_content}"
    );
}

// ---------------------------------------------------------------------------
// Ignore fields test
// ---------------------------------------------------------------------------

#[test]
fn test_sync_ignore_field_not_in_diff() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Forced source-to-target first
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "ignore-keys"],
    );
    assert!(output.status.success());

    let target = home.path().join(".config/ignore-keys/config.toml");

    // Add an ignored field to the deployed file
    let mut content = std::fs::read_to_string(&target).unwrap();
    content.push_str("timestamp = \"2026-01-01\"\n");
    std::fs::write(&target, &content).unwrap();

    // View should not show timestamp as a diff
    let output = run_blend(home.path(), &orders, &["view", "ignore-keys"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        !stdout.contains("timestamp"),
        "Ignored field 'timestamp' should not appear in diff output:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// --no-rewrite flag test
// ---------------------------------------------------------------------------

#[test]
fn test_sync_no_rewrite_flag() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("os-match");
    let orders = temp_orders.path();

    // Forced source-to-target first
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "os-match"],
    );
    assert!(output.status.success());

    let target = home.path().join(".config/os-match/config.toml");

    // Save original order.ncl
    let original_ncl =
        std::fs::read_to_string(orders_dir(orders).join("os-match/order.ncl")).unwrap();

    // Modify deployed file
    let content = std::fs::read_to_string(&target).unwrap();
    let modified = content
        .lines()
        .map(|line| {
            if line.starts_with("font_size") {
                "font_size = 99"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&target, &modified).unwrap();

    // Forced target-to-source with --no-rewrite — should NOT modify order.ncl
    let output = run_blend(
        home.path(),
        orders,
        &[
            "sync",
            "--force-target-to-source",
            "--no-rewrite",
            "os-match",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "sync --force-target-to-source --no-rewrite failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // order.ncl should be unchanged
    let current_ncl =
        std::fs::read_to_string(orders_dir(orders).join("os-match/order.ncl")).unwrap();
    assert_eq!(
        current_ncl, original_ncl,
        "order.ncl should not be modified with --no-rewrite"
    );
}

// ---------------------------------------------------------------------------
// Error handling test
// ---------------------------------------------------------------------------

#[test]
fn test_sync_force_source_to_target_error_malformed_ncl() {
    let home = TempDir::new().unwrap();
    let temp = TempDir::new().unwrap();
    let order_dir = temp.path().join("orders/test-bad");
    std::fs::create_dir_all(&order_dir).unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        "{ this is not valid nickel syntax !!!",
    )
    .unwrap();

    let output = run_blend(
        home.path(),
        temp.path(),
        &["sync", "--force-source-to-target", "test-bad"],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should report an error but not crash
    let has_error = !output.status.success()
        || stdout.contains("error")
        || stdout.contains("Error")
        || stdout.contains("failed")
        || stdout.contains("Failed");
    assert!(
        has_error,
        "Should report error for malformed ncl:\nstdout: {stdout}\nstderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Unexpected symlink detection and replacement tests
// ---------------------------------------------------------------------------

/// Helper: create a symlink at `link` pointing to `target`.
#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[test]
#[cfg(unix)]
fn test_sync_force_source_to_target_replaces_symlink_with_real_file() {
    // Simulate a stow-style symlink: target is a symlink to a file with matching content.
    // `blend sync --force-source-to-target` should replace the symlink with a real file.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // First, create a real file elsewhere with the same content that blend would deploy
    let stow_dir = TempDir::new().unwrap();
    let stow_file = stow_dir.path().join("config.txt");

    // Read the source file content to know what blend would deploy
    let source_content =
        std::fs::read_to_string(fixtures_dir().join("orders/plaintext-single/config.txt")).unwrap();
    std::fs::write(&stow_file, &source_content).unwrap();

    // Create a symlink at the target location pointing to the stow file
    let target = home.path().join(".config/plaintext-single/config.txt");
    create_symlink(&stow_file, &target);

    // Verify it's a symlink with matching content
    assert!(target.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), source_content);

    // Sync --force-source-to-target should detect the symlink mismatch and replace it
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sync --force-source-to-target failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // The target should now be a real file, not a symlink
    assert!(
        !target.symlink_metadata().unwrap().file_type().is_symlink(),
        "Target should no longer be a symlink"
    );

    // Content should still match
    assert_eq!(std::fs::read_to_string(&target).unwrap(), source_content);

    // Output should mention re-deployment
    assert!(
        stdout.contains("Re-deployed") || stdout.contains("replaced symlink"),
        "Should mention re-deployment in output:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_force_source_to_target_replaces_symlinked_directory() {
    // Test that a symlinked directory target also gets replaced with a real directory.
    // We use test-file which has a from_file entry pointing to a single file,
    // but we need to test with a from_config entry too.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // For from_config (structured) entries: test-plain has from_config
    // First, render to know the expected content
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    assert!(output.status.success());

    let target = home.path().join(".config/toml-basic/config.toml");
    let expected_content = std::fs::read_to_string(&target).unwrap();

    // Now remove the target and replace with a symlink to a file with same content
    let stow_dir = TempDir::new().unwrap();
    let stow_file = stow_dir.path().join("config.toml");
    std::fs::write(&stow_file, &expected_content).unwrap();
    std::fs::remove_file(&target).unwrap();
    create_symlink(&stow_file, &target);

    assert!(target.symlink_metadata().unwrap().file_type().is_symlink());

    // Sync --force-source-to-target should replace the symlink
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sync --force-source-to-target failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // Should be a real file now
    assert!(
        !target.symlink_metadata().unwrap().file_type().is_symlink(),
        "Target should no longer be a symlink"
    );
    assert_eq!(std::fs::read_to_string(&target).unwrap(), expected_content);
}

#[test]
#[cfg(unix)]
fn test_view_shows_symlink_annotation() {
    // When a target is a symlink but the order doesn't specify symlink=true,
    // `blend view` should show a "(symlinked, needs re-deploy)" annotation.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally first
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    let content = std::fs::read_to_string(&target).unwrap();

    // Replace with a symlink to a file with same content
    let stow_dir = TempDir::new().unwrap();
    let stow_file = stow_dir.path().join("config.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&target).unwrap();
    create_symlink(&stow_file, &target);

    // View should show the symlink annotation
    let output = run_blend(home.path(), &orders, &["view", "plaintext-single"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "view failed:\n{}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("symlinked, needs re-deploy"),
        "Should show symlink annotation in view output:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_interactive_replaces_symlink_automatically() {
    // In interactive mode with no content changes, unexpected symlinks should
    // be auto-redeployed without prompting.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    let content = std::fs::read_to_string(&target).unwrap();

    // Replace with symlink
    let stow_dir = TempDir::new().unwrap();
    let stow_file = stow_dir.path().join("config.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&target).unwrap();
    create_symlink(&stow_file, &target);

    // Run sync without --force-source-to-target (interactive mode), but since there's no content
    // diff, the symlink replacement should happen automatically without prompting.
    // (No stdin needed because we don't reach the prompt.)
    let output = run_blend(home.path(), &orders, &["sync", "plaintext-single"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "sync failed:\n{}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    // Should now be a real file
    assert!(
        !target.symlink_metadata().unwrap().file_type().is_symlink(),
        "Target should no longer be a symlink after interactive sync"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_dry_run_detects_symlink_but_does_not_replace() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    let content = std::fs::read_to_string(&target).unwrap();

    // Replace with symlink
    let stow_dir = TempDir::new().unwrap();
    let stow_file = stow_dir.path().join("config.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&target).unwrap();
    create_symlink(&stow_file, &target);

    // Dry run should detect but not modify
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "-n", "plaintext-single"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("symlinked") || stdout.contains("re-deploy"),
        "Dry run should mention symlink mismatch:\n{stdout}"
    );

    // Should still be a symlink
    assert!(
        target.symlink_metadata().unwrap().file_type().is_symlink(),
        "Dry run should not modify the symlink"
    );
}

/// Inner-file leftover scenario: the deployed *directory* is a real dir,
/// but one file inside it is still a stow-style symlink. Status, view, and
/// sync must all surface and replace it — these were silent before.
#[test]
#[cfg(unix)]
fn test_status_shows_symlinked_for_inner_file_symlink() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally: real dir with real files
    let deploy = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );
    assert!(deploy.status.success());

    let inner = home.path().join(".config/plaintext-dir/conf/file1.txt");
    let content = std::fs::read_to_string(&inner).unwrap();

    // Replace just file1.txt with a symlink to identical content
    let stow = TempDir::new().unwrap();
    let stow_file = stow.path().join("file1.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&inner).unwrap();
    create_symlink(&stow_file, &inner);
    assert!(inner.symlink_metadata().unwrap().file_type().is_symlink());

    let output = run_blend(home.path(), &orders, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("symlinked"),
        "Status must show 'symlinked' when an inner file in a directory entry is a symlink:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_view_annotates_inner_file_symlink() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );

    let inner = home.path().join(".config/plaintext-dir/conf/file1.txt");
    let content = std::fs::read_to_string(&inner).unwrap();
    let stow = TempDir::new().unwrap();
    let stow_file = stow.path().join("file1.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&inner).unwrap();
    create_symlink(&stow_file, &inner);

    let output = run_blend(home.path(), &orders, &["view", "plaintext-dir"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("unexpected symlink"),
        "View must annotate inner-file symlink:\n{stdout}"
    );
    assert!(
        stdout.contains("file1.txt"),
        "View must name the offending file:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_interactive_auto_replaces_inner_file_symlink() {
    // Pure-symlink-no-content-diff case for an inner file. Interactive sync
    // should auto-redeploy (no prompt), matching the top-level symlink UX.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );

    let inner = home.path().join(".config/plaintext-dir/conf/file1.txt");
    let content = std::fs::read_to_string(&inner).unwrap();
    let stow = TempDir::new().unwrap();
    let stow_file = stow.path().join("file1.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&inner).unwrap();
    create_symlink(&stow_file, &inner);

    // Interactive sync (no --force-source-to-target) — must NOT prompt because there's no
    // content diff, only a structural symlink mismatch.
    let output = run_blend(home.path(), &orders, &["sync", "plaintext-dir"]);
    assert!(output.status.success());
    assert!(
        !inner.symlink_metadata().unwrap().file_type().is_symlink(),
        "Inner-file symlink must be auto-replaced in interactive mode when content matches"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_force_source_to_target_replaces_inner_file_symlink() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );

    let inner = home.path().join(".config/plaintext-dir/conf/file1.txt");
    let content = std::fs::read_to_string(&inner).unwrap();
    let stow = TempDir::new().unwrap();
    let stow_file = stow.path().join("file1.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&inner).unwrap();
    create_symlink(&stow_file, &inner);
    assert!(inner.symlink_metadata().unwrap().file_type().is_symlink());

    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );
    assert!(output.status.success());
    assert!(
        !inner.symlink_metadata().unwrap().file_type().is_symlink(),
        "Inner file must be a real file after forced source-to-target"
    );
    // Stow source must remain untouched (forced source-to-target must not write through the symlink)
    assert_eq!(std::fs::read_to_string(&stow_file).unwrap(), content);
}

#[test]
#[cfg(unix)]
fn test_view_annotates_symlink_when_content_also_differs() {
    // Real-world ncdu shape: parent directory of the target is a symlink
    // to a legacy stow tree, AND the resolved file content differs from
    // the source. View must show BOTH the diff and the symlink annotation,
    // not just the diff (otherwise the user can't tell that a redeploy
    // is needed to restructure the path).
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Build the stow-style tree at a path outside home, then symlink
    // the parent directory in.
    let stow = TempDir::new().unwrap();
    let stow_order = stow.path().join("plaintext-single");
    std::fs::create_dir_all(&stow_order).unwrap();
    std::fs::write(stow_order.join("config.txt"), "old stow content\n").unwrap();

    let parent = home.path().join(".config/plaintext-single");
    std::fs::create_dir_all(parent.parent().unwrap()).unwrap();
    create_symlink(&stow_order, &parent);

    // Sanity: target resolves through the symlink to differing content.
    let target = parent.join("config.txt");
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "old stow content\n"
    );
    assert!(
        parent.symlink_metadata().unwrap().file_type().is_symlink(),
        "parent must be a symlink for this test to be meaningful"
    );

    let output = run_blend(home.path(), &orders, &["view", "plaintext-single"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("symlinked, needs re-deploy"),
        "view must annotate symlink even when content also differs:\n{stdout}"
    );
    // And the content diff must still be shown
    assert!(
        stdout.contains("old stow content"),
        "view must still show the content diff:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_status_shows_symlink_mismatch() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    let content = std::fs::read_to_string(&target).unwrap();

    // Replace with symlink
    let stow_dir = TempDir::new().unwrap();
    let stow_file = stow_dir.path().join("config.txt");
    std::fs::write(&stow_file, &content).unwrap();
    std::fs::remove_file(&target).unwrap();
    create_symlink(&stow_file, &target);

    // Status should show "symlinked" status
    let output = run_blend(home.path(), &orders, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("symlinked"),
        "Status should show 'symlinked' for unexpected symlink target:\n{stdout}"
    );
}

#[test]
fn test_status_errors_when_contract_missing() {
    let home = TempDir::new().unwrap();
    let orders = TempDir::new().unwrap();
    // Create a minimal order so discover_orders has something to find,
    // but DO NOT create order.contract.ncl or metadata.ncl.
    let order_name = orders.path().join("orders/dummy");
    std::fs::create_dir_all(&order_name).unwrap();
    std::fs::write(
        order_name.join("order.ncl"),
        r#"{ blend = { files = [] } }"#,
    )
    .unwrap();

    let output = run_blend(home.path(), orders.path(), &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !output.status.success(),
        "expected non-zero exit when contract is missing\nstdout: {stdout}\nstderr: {stderr}"
    );
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("missing"),
        "expected `missing` in output, got: {combined}"
    );
    assert!(
        combined.contains("blend init"),
        "expected `blend init` hint in output, got: {combined}"
    );
}

#[test]
fn test_init_then_status_succeeds() {
    let home = TempDir::new().unwrap();
    let orders = TempDir::new().unwrap();
    // Empty orders/ — init should still create the two schema files.
    let init_out = run_blend(home.path(), orders.path(), &["init"]);
    assert!(
        init_out.status.success(),
        "blend init should succeed on empty orders dir\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&init_out.stdout),
        String::from_utf8_lossy(&init_out.stderr),
    );
    assert!(orders.path().join("orders/order.contract.ncl").exists());
    assert!(orders.path().join("orders/metadata.ncl").exists());

    let status_out = run_blend(home.path(), orders.path(), &[]);
    assert!(
        status_out.status.success(),
        "blend status should succeed after init\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&status_out.stdout),
        String::from_utf8_lossy(&status_out.stderr),
    );
}

#[test]
fn test_init_uses_cwd_when_blend_dir_is_absent() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();

    let output = run_blend_in_cwd(home.path(), cwd.path(), &["init"]);
    assert!(
        output.status.success(),
        "blend init should bootstrap an empty cwd\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert!(cwd.path().join("orders/order.contract.ncl").exists());
    assert!(cwd.path().join("orders/metadata.ncl").exists());
    assert!(cwd.path().join("orders/blend/order.ncl").exists());
    assert!(home.path().join(".config/blend/config.toml").exists());
}

#[test]
fn test_commands_use_configured_blend_dir_outside_checkout() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();

    let init_output = run_blend_in_cwd(home.path(), blend_dir.path(), &["init"]);
    assert!(
        init_output.status.success(),
        "blend init should bootstrap an empty cwd\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&init_output.stdout),
        String::from_utf8_lossy(&init_output.stderr),
    );

    let view_output = run_blend_in_cwd(home.path(), outside.path(), &["view"]);
    let stdout = String::from_utf8_lossy(&view_output.stdout);
    assert!(
        view_output.status.success(),
        "blend view should use ~/.config/blend/config.toml outside a checkout\nstdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&view_output.stderr),
    );
    assert!(
        stdout.contains("blend") && !stdout.contains("toml-basic"),
        "blend view should use the configured temp checkout, not the executable checkout\nstdout: {stdout}",
    );
}

#[test]
fn test_valid_cwd_refreshes_stale_configured_blend_dir() {
    let home = TempDir::new().unwrap();
    let stale = copy_fixture("plaintext-single");
    let current = copy_fixture("toml-basic");

    let config_dir = home.path().join(".config/blend");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        format!("blend_dir = \"{}\"\n", stale.path().display()),
    )
    .unwrap();

    let output = run_blend_in_cwd(home.path(), current.path(), &["view", "toml-basic"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "blend view should use the current checkout and succeed\nstdout: {stdout}\nstderr: {stderr}",
    );
    assert!(
        stdout.contains("differs from configured blend-dir"),
        "expected mismatch warning\nstdout: {stdout}",
    );

    let config = std::fs::read_to_string(config_dir.join("config.toml")).unwrap();
    assert!(
        config.contains(&current.path().display().to_string()),
        "config should be refreshed to current checkout, got:\n{config}",
    );
    assert!(
        !config.contains(&stale.path().display().to_string()),
        "config should not keep pointing at stale checkout, got:\n{config}",
    );
}

#[test]
fn test_s_alias_runs_sync() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(
        home.path(),
        &orders,
        &["s", "--force-source-to-target", "toml-basic"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend s failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    assert_eq!(
        std::fs::read_to_string(home.path().join(".config/toml-basic/config.toml")).unwrap(),
        "key = \"value\"\nnumber = 42\n\n[nested]\ninner = true\n"
    )
}

#[test]
fn test_status_errors_when_metadata_stale() {
    let home = TempDir::new().unwrap();
    let orders = TempDir::new().unwrap();
    // Init to create the files, then tamper with metadata.ncl.
    run_blend(home.path(), orders.path(), &["init"]);
    std::fs::write(orders.path().join("orders/metadata.ncl"), "tampered\n").unwrap();

    let output = run_blend(home.path(), orders.path(), &[]);
    assert!(
        !output.status.success(),
        "expected non-zero exit on stale metadata"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("out of date"), "got: {combined}");
    assert!(combined.contains("blend init"), "got: {combined}");
}

/// Compute the expected snapshot path inside an XDG_STATE_HOME root, given
/// the order name and absolute deployed target.
fn snapshot_path_for(state_root: &Path, order_name: &str, target: &Path) -> PathBuf {
    let stripped = target.strip_prefix("/").unwrap();
    state_root
        .join("blend")
        .join("snapshots")
        .join(order_name)
        .join(stripped)
}

/// Run blend with explicit XDG_STATE_HOME; returns (output, state_root_tempdir).
fn run_blend_with_state(
    home: &Path,
    orders: &Path,
    args: &[&str],
) -> (std::process::Output, TempDir) {
    let state = TempDir::new().unwrap();
    let output = Command::new(blend_binary())
        .args(args)
        .arg("--home")
        .arg(home)
        .arg("--blend-dir")
        .arg(orders)
        .env("XDG_STATE_HOME", state.path())
        .output()
        .expect("Failed to execute blend");
    (output, state)
}

#[test]
fn test_sync_force_source_to_target_writes_snapshot() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    let target = home.path().join(".config/toml-basic/config.toml");

    let (output, state) = run_blend_with_state(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    assert!(
        output.status.success(),
        "blend sync --force-source-to-target failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let snap = snapshot_path_for(state.path(), "toml-basic", &target);
    assert!(snap.exists(), "snapshot should exist at {}", snap.display());
    let snap_bytes = std::fs::read(&snap).unwrap();
    let deployed_bytes = std::fs::read(&target).unwrap();
    assert_eq!(
        snap_bytes, deployed_bytes,
        "snapshot bytes must match deployed bytes after forced source-to-target"
    );
}

#[test]
fn test_sync_no_op_confirm_writes_snapshot() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    let target = home.path().join(".config/toml-basic/config.toml");

    // First run pushes and writes a snapshot.
    let (out1, state) = run_blend_with_state(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    assert!(out1.status.success());
    let snap = snapshot_path_for(state.path(), "toml-basic", &target);
    assert!(
        snap.exists(),
        "first forced source-to-target should create snapshot"
    );

    // Delete the snapshot to simulate a pre-feature system.
    std::fs::remove_file(&snap).unwrap();
    assert!(!snap.exists());

    // Re-run sync; deployed already matches rendered, so this is a no-op
    // confirm. The eager-write trigger should re-create the snapshot.
    let output = Command::new(blend_binary())
        .args(["sync", "--force-source-to-target", "-v", "toml-basic"])
        .arg("--home")
        .arg(home.path())
        .arg("--blend-dir")
        .arg(&orders)
        .env("XDG_STATE_HOME", state.path())
        .output()
        .expect("Failed to execute blend");
    assert!(output.status.success());
    assert!(
        snap.exists(),
        "no-op confirm must re-create the snapshot: {}",
        snap.display()
    );
}

#[test]
fn test_sync_force_target_to_source_writes_snapshot() {
    // Use the plaintext fixture which supports forced target-to-source.
    let home = TempDir::new().unwrap();
    let orders_temp = copy_fixture("plaintext-single");
    let orders = orders_temp.path();

    // Initial forced source-to-target to create deployed state and a snapshot.
    let (out1, state) = run_blend_with_state(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );
    assert!(out1.status.success());

    // The fixture deploys ~/.config/plaintext-single/config.txt.
    let target = home.path().join(".config/plaintext-single/config.txt");
    assert!(target.exists());

    // Hand-edit the deployed file to create a divergence.
    std::fs::write(&target, b"edited by user\n").unwrap();
    // Delete the snapshot so we can detect a fresh write.
    let snap = snapshot_path_for(state.path(), "plaintext-single", &target);
    std::fs::remove_file(&snap).unwrap();

    // Forced target-to-source deployed → source.
    let output = Command::new(blend_binary())
        .args(["sync", "--force-target-to-source", "plaintext-single"])
        .arg("--home")
        .arg(home.path())
        .arg("--blend-dir")
        .arg(orders)
        .env("XDG_STATE_HOME", state.path())
        .output()
        .expect("Failed to execute blend");
    assert!(
        output.status.success(),
        "forced target-to-source failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        snap.exists(),
        "snapshot should be re-created after forced target-to-source"
    );
    assert_eq!(
        std::fs::read(&snap).unwrap(),
        b"edited by user\n",
        "snapshot bytes should match the pulled (deployed) bytes"
    );
}

#[test]
fn test_sync_dry_run_logs_deployed_deleted_annotation_when_target_missing() {
    // 1. Forced source-to-target to create deployed + snapshot.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    let target = home.path().join(".config/toml-basic/config.toml");
    let (out1, state) = run_blend_with_state(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    assert!(out1.status.success());
    assert!(target.exists());

    // 2. Delete the deployed file but leave the snapshot intact.
    std::fs::remove_file(&target).unwrap();
    let snap = snapshot_path_for(state.path(), "toml-basic", &target);
    assert!(snap.exists());

    // 3. Run a dry-run sync (no flags = interactive, but --dry-run skips the
    //    prompt and logs what would happen). Stdout should mention the
    //    "Target file was deleted" annotation.
    let output = Command::new(blend_binary())
        .args(["sync", "--dry-run", "toml-basic"])
        .arg("--home")
        .arg(home.path())
        .arg("--blend-dir")
        .arg(&orders)
        .env("XDG_STATE_HOME", state.path())
        .output()
        .expect("Failed to execute blend");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("Target file was deleted"),
        "expected 'Target file was deleted' annotation in dry-run output, got:\n{}",
        stdout
    );
}

#[test]
fn test_sync_dry_run_writes_no_snapshots() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    let (output, state) = run_blend_with_state(
        home.path(),
        &orders,
        &[
            "sync",
            "--dry-run",
            "--force-source-to-target",
            "toml-basic",
        ],
    );
    assert!(output.status.success());
    let snapshots_root = state.path().join("blend").join("snapshots");
    if snapshots_root.exists() {
        let entries: Vec<_> = walkdir::WalkDir::new(&snapshots_root)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();
        assert!(
            entries.is_empty(),
            "dry-run must not write any snapshot files; found: {:?}",
            entries
                .iter()
                .map(|e| e.path().to_owned())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_sync_directory_entry_writes_per_leaf_snapshots() {
    // plaintext-dir is a from_file directory entry. After forced source-to-target, every leaf
    // file inside the deployed directory should have a corresponding
    // snapshot mirroring its absolute target path.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    let (output, state) = run_blend_with_state(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );
    assert!(
        output.status.success(),
        "forced source-to-target failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Walk the deployed dir; each file must have a matching snapshot.
    let deploy_root = home.path().join(".config/plaintext-dir/conf");
    let mut leaf_count = 0;
    for entry in walkdir::WalkDir::new(&deploy_root).min_depth(1) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        leaf_count += 1;
        let snap = snapshot_path_for(state.path(), "plaintext-dir", entry.path());
        assert!(
            snap.exists(),
            "missing snapshot for leaf {}: expected at {}",
            entry.path().display(),
            snap.display()
        );
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(&snap).unwrap(),
            "snapshot bytes must equal deployed bytes for {}",
            entry.path().display()
        );
    }
    assert!(
        leaf_count > 0,
        "expected at least one leaf file under {}",
        deploy_root.display()
    );
}

#[test]
fn test_sync_directory_snapshot_ignores_target_only_files() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    let deploy_root = home.path().join(".config/plaintext-dir/conf");
    let target_only = deploy_root.join("target-only/cache.bin");
    std::fs::create_dir_all(target_only.parent().unwrap()).unwrap();
    std::fs::write(&target_only, b"not managed by blend").unwrap();

    let (output, state) = run_blend_with_state(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );
    assert!(
        output.status.success(),
        "forced source-to-target failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        target_only.exists(),
        "forced source-to-target should leave target-only files alone"
    );

    let managed_snapshot = snapshot_path_for(
        state.path(),
        "plaintext-dir",
        &deploy_root.join("file1.txt"),
    );
    assert!(
        managed_snapshot.exists(),
        "managed file should still have a snapshot"
    );

    let target_only_snapshot = snapshot_path_for(state.path(), "plaintext-dir", &target_only);
    assert!(
        !target_only_snapshot.exists(),
        "target-only files must not be snapshotted: {}",
        target_only_snapshot.display()
    );
}

#[test]
fn test_sync_dry_run_logs_source_changed_annotation_in_diff_summary() {
    // Forced source-to-target to bootstrap snapshot, then edit the .ncl source so rendered
    // differs from snapshot but deployed still equals snapshot. Run
    // `blend sync --dry-run` (no --force-source-to-target/--force-target-to-source) and look for the
    // annotation in the dry-run output.
    let home = TempDir::new().unwrap();
    let orders_temp = copy_fixture("toml-basic");
    let orders = orders_temp.path();

    let (out1, state) = run_blend_with_state(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "toml-basic"],
    );
    assert!(out1.status.success());

    // Edit the source: change the value of `number` in toml-basic/order.ncl.
    let order_ncl = orders_dir(orders).join("toml-basic").join("order.ncl");
    let original = std::fs::read_to_string(&order_ncl).unwrap();
    let edited = original.replace("42", "1337");
    assert_ne!(original, edited, "fixture edit must change something");
    std::fs::write(&order_ncl, edited).unwrap();

    // Dry-run interactive sync. The annotation OR the dry-run prompt note
    // should appear in stdout. (--dry-run skips the actual prompt and just
    // logs what would happen.)
    let output = Command::new(blend_binary())
        .args(["sync", "--dry-run", "-v", "toml-basic"])
        .arg("--home")
        .arg(home.path())
        .arg("--blend-dir")
        .arg(orders)
        .env("XDG_STATE_HOME", state.path())
        .output()
        .expect("Failed to execute blend");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    // Either: the source-changed annotation surfaces in interactive dry-run output,
    // OR: the dry-run "would prompt" branch is taken before annotation logic runs.
    // Both indicate the divergence was detected; the spec allows either depending
    // on cmd_sync's dry-run prompt skip strategy.
    assert!(
        stdout.contains("Source changed") || stdout.contains("[dry-run] would prompt"),
        "expected source-changed annotation or dry-run prompt note, got:\n{}",
        stdout
    );
    // Snapshot should be untouched after dry-run.
    let target = home.path().join(".config/toml-basic/config.toml");
    let snap = snapshot_path_for(state.path(), "toml-basic", &target);
    assert!(snap.exists());
    assert_eq!(
        std::fs::read(&snap).unwrap(),
        std::fs::read(&target).unwrap(),
        "dry-run must not mutate snapshot or deployed bytes"
    );
}
