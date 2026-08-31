//! E2E tests for blend sync using real fixtures and temporary home directories.
//!
//! These tests exercise the full sync flow (build → diff → forced source-to-target
//! and target-to-source modes) without interactive prompts.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

fn run_blend_with_stdin(
    home: &Path,
    blend_dir: &Path,
    args: &[&str],
    input: &str,
) -> std::process::Output {
    let mut child = Command::new(blend_binary())
        .args(args)
        .arg("--home")
        .arg(home)
        .arg("--blend-dir")
        .arg(blend_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute blend");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
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

    copy_shared_order_files(temp.path());

    temp
}

fn copy_shared_order_files(blend_dir: &Path) {
    let orders_src = fixtures_dir().join("orders");
    let orders_dst = orders_dir(blend_dir);
    std::fs::create_dir_all(&orders_dst).unwrap();

    // Copy the two blend-owned schema files so reader commands don't fail
    // their freshness check and so metadata-importing fixtures resolve.
    for shared in ["order.contract.ncl", "metadata.ncl"] {
        std::fs::copy(orders_src.join(shared), orders_dst.join(shared)).unwrap();
    }
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
fn test_create_order_scaffolds_empty_order() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let output = run_blend(home.path(), blend_dir.path(), &["create", "kitty"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend create failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Created order 'kitty'"));

    let order_path = orders_dir(blend_dir.path()).join("kitty/order.ncl");
    let source = std::fs::read_to_string(&order_path).unwrap();
    assert!(source.contains("files = []"));

    let check = run_blend(home.path(), blend_dir.path(), &["check", "kitty"]);
    assert!(
        check.status.success(),
        "created order should check:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr),
    );
}

#[test]
fn test_add_target_path_defaults_to_home_prefix() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let target = home.path().join(".config/kitty/kitty.conf");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "font_size 13\n").unwrap();

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["add", "kitty", "~/.config/kitty/kitty.conf"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend add failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Created order 'kitty'"));
    assert!(stdout.contains("Added"));

    let copied = orders_dir(blend_dir.path()).join("kitty/.config/kitty/kitty.conf");
    assert_eq!(std::fs::read_to_string(copied).unwrap(), "font_size 13\n");

    let order_source =
        std::fs::read_to_string(orders_dir(blend_dir.path()).join("kitty/order.ncl")).unwrap();
    assert!(order_source.contains(r#"prefix = ["~"]"#));
    assert!(order_source.contains(r#"from_file = ".config/kitty/kitty.conf""#));
    assert!(
        order_source.find("prefix =").unwrap() < order_source.find("files =").unwrap(),
        "promoted order prefix should be inserted before files:\n{order_source}"
    );

    let check = run_blend(home.path(), blend_dir.path(), &["check", "kitty"]);
    assert!(
        check.status.success(),
        "added order should check:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr),
    );
}

#[test]
fn test_add_target_path_with_explicit_prefix_strips_prefix() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let target = home.path().join(".config/kitty/kitty.conf");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "font_size 13\n").unwrap();

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &[
            "add",
            "kitty",
            "--prefix",
            "~/.config/kitty",
            "~/.config/kitty/kitty.conf",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend add --prefix failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Created order 'kitty'"));

    let copied = orders_dir(blend_dir.path()).join("kitty/kitty.conf");
    assert_eq!(std::fs::read_to_string(copied).unwrap(), "font_size 13\n");

    let order_source =
        std::fs::read_to_string(orders_dir(blend_dir.path()).join("kitty/order.ncl")).unwrap();
    assert!(order_source.contains(r#"prefix = ["~/.config/kitty"]"#));
    assert!(order_source.contains(r#"from_file = "kitty.conf""#));
    assert!(
        order_source.find("prefix =").unwrap() < order_source.find("files =").unwrap(),
        "promoted order prefix should be inserted before files:\n{order_source}"
    );
}

#[test]
fn test_add_failure_does_not_leave_new_order() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["add", "kitty", "~/.config/kitty/missing.conf"],
    );

    assert!(
        !output.status.success(),
        "blend add should reject a missing Target"
    );
    assert!(
        !orders_dir(blend_dir.path()).join("kitty").exists(),
        "a failed add must not leave a new order"
    );
}

#[test]
fn test_add_dry_run_previews_new_order_without_creating_it() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let target = home.path().join(".config/kitty/kitty.conf");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "font_size 13\n").unwrap();

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["--dry-run", "add", "kitty", "~/.config/kitty/kitty.conf"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend add --dry-run failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("Dry run: would create order 'kitty'"));
    assert!(stdout.contains(r#"prefix = ["~"]"#));
    assert!(stdout.contains(r#"from_file = ".config/kitty/kitty.conf""#));
    assert!(
        !orders_dir(blend_dir.path()).join("kitty").exists(),
        "dry-run must not create the order"
    );
}

#[test]
fn test_add_promoted_prefix_goes_before_comments_when_files_is_absent() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let order_dir = orders_dir(blend_dir.path()).join("kitty");
    std::fs::create_dir_all(&order_dir).unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        r#"let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    # no files yet
  },
} | Order
"#,
    )
    .unwrap();

    let target = home.path().join(".config/kitty/kitty.conf");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, "font_size 13\n").unwrap();

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["add", "kitty", "~/.config/kitty/kitty.conf"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend add failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    let order_source =
        std::fs::read_to_string(orders_dir(blend_dir.path()).join("kitty/order.ncl")).unwrap();
    let prefix_pos = order_source.find("prefix =").unwrap();
    assert!(
        prefix_pos < order_source.find("# no files yet").unwrap(),
        "promoted order prefix should be inserted before existing comments:\n{order_source}"
    );
    assert!(
        prefix_pos < order_source.find("files =").unwrap(),
        "promoted order prefix should be inserted before created files:\n{order_source}"
    );
}

#[test]
fn test_add_symlink_target_requires_explicit_policy() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());
    run_blend(home.path(), blend_dir.path(), &["create", "kitty"]);

    let real = home.path().join(".config/kitty/real.conf");
    let link = home.path().join(".config/kitty/kitty.conf");
    std::fs::create_dir_all(real.parent().unwrap()).unwrap();
    std::fs::write(&real, "font_size 13\n").unwrap();
    create_symlink(&real, &link);

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["add", "kitty", "~/.config/kitty/kitty.conf"],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "blend add should reject symlink without policy"
    );
    assert!(stderr.contains("--symlink follow") && stderr.contains("--symlink preserve"));
}

#[test]
fn test_check_rejects_from_file_that_escapes_order_dir() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());

    let order_dir = orders_dir(blend_dir.path()).join("bad");
    std::fs::create_dir_all(&order_dir).unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        r#"let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~"],
    files = [
      {
        from_file = "../outside",
      },
    ],
  },
} | Order
"#,
    )
    .unwrap();

    let output = run_blend(home.path(), blend_dir.path(), &["check", "bad"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "blend check should reject escaping from_file"
    );
    assert!(
        stderr.contains("source path escapes the order directory"),
        "stderr should explain escaping path:\n{stderr}"
    );
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
fn test_view_marks_missing_structured_target_not_deployed() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    for args in [
        &["view", "toml-basic"][..],
        &["view", "--short", "toml-basic"][..],
    ] {
        let output = run_blend(home.path(), &orders, args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "view failed:\nstdout: {stdout}\nstderr: {stderr}"
        );
        assert!(
            stdout.contains("(not deployed)"),
            "missing structured target should be reported as not deployed:\n{stdout}"
        );
        assert!(
            !stdout.contains("(no changes)") && !stdout.contains("All orders are up to date"),
            "missing structured target must not be reported as up to date:\n{stdout}"
        );
    }
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
fn test_status_subcommand_shows_orders() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(home.path(), &orders, &["status"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("toml-basic"),
        "explicit status should list orders:\n{stdout}"
    );
}

#[test]
fn test_help_groups_commands_and_documents_blend_dir_resolution() {
    let output = Command::new(blend_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute blend");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend --help failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Inspect:"),
        "missing Inspect group:\n{stdout}"
    );
    assert!(
        stdout.contains("Maintain:"),
        "missing Maintain group:\n{stdout}"
    );
    assert!(
        stdout.contains("status  [read] Show order deployment status (default)"),
        "missing explicit status command:\n{stdout}"
    );
    assert!(
        stdout.contains("sync    [source, target] Reconcile Source orders and Target files"),
        "missing sync safety tag:\n{stdout}"
    );
    assert!(
        stdout.contains("nearest ancestor with orders/, then remembered state"),
        "missing blend-dir resolution docs:\n{stdout}"
    );
}

#[test]
fn test_status_help_succeeds() {
    let output = Command::new(blend_binary())
        .args(["status", "-h"])
        .output()
        .expect("Failed to execute blend");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "blend status -h failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[read] Show order deployment status (default)"),
        "status help should describe the command:\n{stdout}"
    );
}

#[test]
fn test_view_content_only_conflicts_with_all() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(home.path(), &orders, &["view", "--content-only", "--all"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "conflicting view flags should fail:\nstdout: {}\nstderr: {stderr}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stderr.contains("--content-only") && stderr.contains("--all"),
        "error should name both conflicting flags:\n{stderr}"
    );
}

#[test]
fn test_sync_force_directions_conflict() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let output = run_blend(
        home.path(),
        &orders,
        &[
            "sync",
            "--force-source-to-target",
            "--force-target-to-source",
        ],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "conflicting sync force flags should fail:\nstdout: {}\nstderr: {stderr}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stderr.contains("--force-source-to-target") && stderr.contains("--force-target-to-source"),
        "error should name both conflicting flags:\n{stderr}"
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
// YAML format tests
// ---------------------------------------------------------------------------

#[test]
fn test_sync_force_source_to_target_yaml_format() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    let target = home.path().join(".config/yaml-format/settings.yaml");
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "yaml-format"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "YAML source-to-target sync failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(target).unwrap(),
        "enabled: true\nmessage: hello\nnested:\n  count: 2\nports:\n  - 8080\n  - 8443\n"
    );
}

#[test]
fn test_sync_force_target_to_source_accepts_yaml_anchors_and_merge() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("yaml-format");
    let orders = temp_orders.path();

    let initial = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "yaml-format"],
    );
    assert!(initial.status.success());

    let target = home.path().join(".config/yaml-format/settings.yaml");
    std::fs::write(
        &target,
        "defaults: &defaults\n  count: 3\nenabled: true\nmessage: hello\nnested:\n  <<: *defaults\nports: [8080, 8443]\n",
    )
    .unwrap();

    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-target-to-source", "yaml-format"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "YAML target-to-source sync failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    let source = std::fs::read_to_string(orders_dir(orders).join("yaml-format/order.ncl")).unwrap();
    assert!(
        source.contains("count = 3"),
        "updated YAML was not pulled:\n{source}"
    );
    assert!(
        source.contains("defaults"),
        "merged YAML key was not pulled:\n{source}"
    );
}

#[test]
fn test_sync_interactive_can_restore_malformed_yaml_target_from_source() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("yaml-format");
    let orders = temp_orders.path();

    let initial = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "yaml-format"],
    );
    assert!(initial.status.success());

    let target = home.path().join(".config/yaml-format/settings.yaml");
    let expected = std::fs::read_to_string(&target).unwrap();
    std::fs::write(&target, "enabled: [\n").unwrap();

    let output = run_blend_with_stdin(home.path(), orders, &["sync", "yaml-format"], "s\n");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "interactive YAML repair failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[s]ource -> target"),
        "whole-file repair prompt was not shown:\n{stdout}"
    );
    assert!(
        format!("{stdout}\n{stderr}").contains("showing textual diff"),
        "YAML parse fallback warning was not visible:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(std::fs::read_to_string(target).unwrap(), expected);
}

/// Regression test for the vscode `[javascript]` sync-back bug: a nested
/// from_config record whose path segments are all literal keys containing
/// dots/brackets (`"[javascript]"` -> `"editor.codeActionsOnSave"` ->
/// `"source.fixAll.eslint"`). Resolving the conflict in favor of Target must
/// actually rewrite the Source .ncl — blend previously reported success while
/// silently leaving order.ncl untouched, because `json_path_get` cannot
/// resolve the dotted LeafSpan path when non-root segments contain literal
/// dots.
#[test]
fn test_sync_target_to_source_literal_dotted_bracket_keys() {
    let home = TempDir::new().unwrap();
    let temp_orders = copy_fixture("literal-dotted-keys");
    let orders = temp_orders.path();

    // Deploy Source -> Target first
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-source-to-target", "literal-dotted-keys"],
    );
    assert!(output.status.success());

    let target = home
        .path()
        .join(".config/literal-dotted-keys/settings.json");

    // Simulate the user changing the value in the deployed Target file
    // (VS Code writing "always" for the eslint fixAll code action).
    let mut parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&target).unwrap()).unwrap();
    parsed["[javascript]"]["editor.codeActionsOnSave"]["source.fixAll.eslint"] =
        serde_json::json!("always");
    std::fs::write(&target, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();

    // Resolve in favor of Target (same rewrite path as picking [t]arget in
    // the interactive per-key prompt: pull -> surgical_rewrite via segment paths).
    let output = run_blend(
        home.path(),
        orders,
        &["sync", "--force-target-to-source", "literal-dotted-keys"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Forced target-to-source failed:\nstdout: {stdout}\nstderr: {stderr}"
    );

    // The Source .ncl must actually contain the pulled Target value. If blend
    // claimed success but left "explicit" in place, the sync-back was a
    // silent no-op.
    let ncl_content =
        std::fs::read_to_string(orders_dir(orders).join("literal-dotted-keys/order.ncl")).unwrap();
    assert!(
        ncl_content.contains("\"always\""),
        "order.ncl should have '[javascript].editor.codeActionsOnSave.source.fixAll.eslint' \
         rewritten to \"always\", but the Target -> Source write-back was silently dropped:\n\
         stdout: {stdout}\nstderr: {stderr}\norder.ncl:\n{ncl_content}"
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

#[cfg(unix)]
fn create_broken_symlink(link: &Path) {
    create_symlink(Path::new("missing-target"), link);
}

/// Replace an existing file with an unexpected symlink to identical content.
/// The returned TempDir keeps the backing file alive for the test duration.
#[cfg(unix)]
fn replace_with_unexpected_symlink(target: &Path) -> (TempDir, PathBuf, String) {
    let content = std::fs::read_to_string(target).unwrap();
    let backing_dir = TempDir::new().unwrap();
    let backing_file = backing_dir.path().join(target.file_name().unwrap());
    std::fs::write(&backing_file, &content).unwrap();
    std::fs::remove_file(target).unwrap();
    create_symlink(&backing_file, target);
    (backing_dir, backing_file, content)
}

#[test]
#[cfg(unix)]
fn test_sync_force_source_to_target_replaces_symlink_with_real_file() {
    // The target is an unexpected symlink to a file with matching content.
    // `blend sync --force-source-to-target` should replace the symlink with a real file.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // First, create a real file elsewhere with the same content that blend would deploy
    let backing_dir = TempDir::new().unwrap();
    let backing_file = backing_dir.path().join("config.txt");

    // Read the source file content to know what blend would deploy
    let source_content =
        std::fs::read_to_string(fixtures_dir().join("orders/plaintext-single/config.txt")).unwrap();
    std::fs::write(&backing_file, &source_content).unwrap();

    // Create a symlink at the target location pointing to the backing file.
    let target = home.path().join(".config/plaintext-single/config.txt");
    create_symlink(&backing_file, &target);

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

    // Output should report the explicit Source -> Target resolution.
    assert!(
        stdout.contains("Applied Source -> Target"),
        "Should mention Source -> Target in output:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_force_source_to_target_replaces_structured_file_symlink() {
    // Structured from_config entries must obey the same real-file invariant.
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

    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&target);

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
    // When a target is a symlink but the order expects a regular file,
    // `blend view` should report a generic node-type mismatch.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally first
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&target);

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
        stdout.contains("type mismatch: expected file, found symlink"),
        "Should show type mismatch in view output:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_interactive_does_not_replace_symlink_automatically() {
    // An existing node-type mismatch requires an explicit Source -> Target
    // decision even when the symlink referent has matching content.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Deploy normally
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );

    let target = home.path().join(".config/plaintext-single/config.txt");
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&target);

    let output = run_blend_with_stdin(home.path(), &orders, &["sync", "plaintext-single"], "k\n");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "sync failed:\n{}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    // Skip must preserve the exact target symlink.
    assert!(
        target.symlink_metadata().unwrap().file_type().is_symlink(),
        "Target symlink must remain after skipping the conflict"
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
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&target);

    // Dry run should detect but not modify
    let output = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "-n", "plaintext-single"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("type mismatch: expected file, found symlink"),
        "Dry run should mention the type mismatch:\n{stdout}"
    );

    // Should still be a symlink
    assert!(
        target.symlink_metadata().unwrap().file_type().is_symlink(),
        "Dry run should not modify the symlink"
    );
}

#[test]
#[cfg(unix)]
fn test_broken_symlink_is_existing_type_mismatch_and_requires_explicit_source() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("plaintext-single");
    let target = home.path().join(".config/plaintext-single/config.txt");
    create_broken_symlink(&target);

    let status = run_blend(home.path(), blend_dir.path(), &[]);
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status.status.success());
    assert!(status_stdout.contains("mismatch"));
    assert!(!status_stdout.contains("plaintext-single     config.txt           pending"));

    let view = run_blend(home.path(), blend_dir.path(), &["view", "plaintext-single"]);
    let view_stdout = String::from_utf8_lossy(&view.stdout);
    assert!(view_stdout.contains("type mismatch: expected file, found symlink"));
    assert!(!view_stdout.contains("not deployed"));

    let source = orders_dir(blend_dir.path()).join("plaintext-single/config.txt");
    let source_before = std::fs::read_to_string(&source).unwrap();
    let pull = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-target-to-source", "plaintext-single"],
    );
    let pull_stdout = String::from_utf8_lossy(&pull.stdout);
    assert!(pull.status.success());
    assert!(pull_stdout.contains("target structure does not match Source"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), source_before);
    assert!(target.symlink_metadata().unwrap().file_type().is_symlink());

    let push = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "plaintext-single"],
    );
    assert!(push.status.success());
    assert!(!target.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), source_before);
}

#[test]
#[cfg(unix)]
fn test_force_target_to_source_does_not_follow_unexpected_symlink() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("plaintext-single");
    let source = orders_dir(blend_dir.path()).join("plaintext-single/config.txt");
    let source_before = std::fs::read_to_string(&source).unwrap();
    let backing = TempDir::new().unwrap();
    let backing_file = backing.path().join("config.txt");
    std::fs::write(&backing_file, "content from outside Blend\n").unwrap();
    let target = home.path().join(".config/plaintext-single/config.txt");
    create_symlink(&backing_file, &target);

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-target-to-source", "plaintext-single"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("target structure does not match Source"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), source_before);
    assert_eq!(
        std::fs::read_to_string(&backing_file).unwrap(),
        "content from outside Blend\n"
    );
    assert!(target.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
#[cfg(unix)]
fn test_intended_symlink_uses_explicit_reconciliation_for_existing_mismatches() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());
    let order_dir = orders_dir(blend_dir.path()).join("symlink-entry");
    std::fs::create_dir_all(&order_dir).unwrap();
    let source = order_dir.join("source.txt");
    std::fs::write(&source, "source content\n").unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        r#"{
  blend = {
    prefix = ["~/.config/symlink-entry/"],
    files = [{ name = "config.txt", from_file = "source.txt", symlink = true }],
  },
}
"#,
    )
    .unwrap();

    let target = home.path().join(".config/symlink-entry/config.txt");
    let initial = run_blend(home.path(), blend_dir.path(), &["sync", "symlink-entry"]);
    assert!(initial.status.success());
    assert_eq!(
        std::fs::read_link(&target).unwrap(),
        source.canonicalize().unwrap()
    );

    std::fs::remove_file(&target).unwrap();
    create_symlink(Path::new("wrong-target"), &target);
    let skipped = run_blend_with_stdin(
        home.path(),
        blend_dir.path(),
        &["sync", "symlink-entry"],
        "k\n",
    );
    assert!(skipped.status.success());
    assert_eq!(
        std::fs::read_link(&target).unwrap(),
        PathBuf::from("wrong-target")
    );

    let pull = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-target-to-source", "symlink-entry"],
    );
    let pull_stdout = String::from_utf8_lossy(&pull.stdout);
    assert!(pull.status.success());
    assert!(pull_stdout.contains("target structure does not match Source"));
    assert_eq!(
        std::fs::read_link(&target).unwrap(),
        PathBuf::from("wrong-target")
    );

    let pushed = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "symlink-entry"],
    );
    assert!(pushed.status.success());
    assert_eq!(
        std::fs::read_link(&target).unwrap(),
        source.canonicalize().unwrap()
    );

    std::fs::remove_file(&target).unwrap();
    std::fs::write(&target, "regular target\n").unwrap();
    let view = run_blend(home.path(), blend_dir.path(), &["view", "symlink-entry"]);
    let view_stdout = String::from_utf8_lossy(&view.stdout);
    assert!(view_stdout.contains("type mismatch: expected symlink, found file"));
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "regular target\n"
    );
}

#[test]
#[cfg(unix)]
fn test_directory_backed_symlink_does_not_inspect_children_through_wrong_link() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());
    let order_dir = orders_dir(blend_dir.path()).join("symlink-directory");
    let source = order_dir.join("source");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("managed.txt"), "managed content\n").unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        r#"{
  blend = {
    prefix = ["~/.config/symlink-directory/"],
    files = [{ name = "config", from_file = "source", symlink = true }],
  },
}
"#,
    )
    .unwrap();

    let wrong_target = home.path().join("wrong-target.txt");
    std::fs::write(&wrong_target, "not a directory\n").unwrap();
    let target = home.path().join(".config/symlink-directory/config");
    create_symlink(&wrong_target, &target);

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "symlink-directory"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "sync failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        !stdout.contains("errors"),
        "unexpected sync error:\n{stdout}"
    );
    assert_eq!(
        std::fs::read_link(&target).unwrap(),
        source.canonicalize().unwrap()
    );
}

#[test]
fn test_status_compares_local_overlay_content_from_effective_inventory() {
    let home = TempDir::new().unwrap();
    let blend_dir = TempDir::new().unwrap();
    copy_shared_order_files(blend_dir.path());
    let order_dir = orders_dir(blend_dir.path()).join("local-overlay-status");
    let source = order_dir.join("source");
    let local = order_dir.join("local");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(&local).unwrap();
    std::fs::write(source.join("tracked.txt"), "tracked content\n").unwrap();
    std::fs::write(local.join("local.txt"), "local content\n").unwrap();
    std::fs::write(
        order_dir.join("order.ncl"),
        r#"{
  blend = {
    prefix = ["~/.config/local-overlay-status/"],
    files = [{ name = "config", from_file = "source", local = "local" }],
  },
}
"#,
    )
    .unwrap();

    let deploy = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "local-overlay-status"],
    );
    assert!(deploy.status.success());

    let local_target = home
        .path()
        .join(".config/local-overlay-status/config/local.txt");
    std::fs::write(&local_target, "drifted local content\n").unwrap();

    let status = run_blend(home.path(), blend_dir.path(), &["status"]);
    let stdout = String::from_utf8_lossy(&status.stdout);
    let stderr = String::from_utf8_lossy(&status.stderr);
    assert!(
        status.status.success(),
        "status failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("deployed"));
    assert!(
        stdout.contains("\u{2260}"),
        "local overlay drift must be reported:\n{stdout}"
    );
}

#[test]
fn test_regular_file_and_directory_type_mismatches_share_reconciliation() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("plaintext-single");
    let file_target = home.path().join(".config/plaintext-single/config.txt");
    std::fs::create_dir_all(&file_target).unwrap();
    std::fs::write(
        file_target.join("unmanaged.txt"),
        "keep until explicit push",
    )
    .unwrap();

    let view = run_blend(home.path(), blend_dir.path(), &["view", "plaintext-single"]);
    assert!(
        String::from_utf8_lossy(&view.stdout)
            .contains("type mismatch: expected file, found directory")
    );

    let skipped = run_blend_with_stdin(
        home.path(),
        blend_dir.path(),
        &["sync", "plaintext-single"],
        "k\n",
    );
    assert!(skipped.status.success());
    assert!(file_target.is_dir());

    let pushed = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "plaintext-single"],
    );
    assert!(pushed.status.success());
    assert!(file_target.is_file());

    let dir_blend = copy_fixture("plaintext-dir");
    let dir_target = home.path().join(".config/plaintext-dir/conf");
    std::fs::create_dir_all(dir_target.parent().unwrap()).unwrap();
    std::fs::write(&dir_target, "wrong node type\n").unwrap();
    let dir_view = run_blend(home.path(), dir_blend.path(), &["view", "plaintext-dir"]);
    assert!(
        String::from_utf8_lossy(&dir_view.stdout)
            .contains("type mismatch: expected directory, found file")
    );
    let dir_push = run_blend(
        home.path(),
        dir_blend.path(),
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );
    assert!(dir_push.status.success());
    assert!(dir_target.is_dir());
    assert!(dir_target.join("file1.txt").is_file());
}

/// A directory entry owns its managed children, so each child uses the same
/// generic node-type comparison as a top-level target.
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
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&inner);
    assert!(inner.symlink_metadata().unwrap().file_type().is_symlink());

    let output = run_blend(home.path(), &orders, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("mismatch"),
        "Status must show a mismatch for an inner file symlink:\n{stdout}"
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
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&inner);

    let output = run_blend(home.path(), &orders, &["view", "plaintext-dir"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("type mismatch: expected file, found symlink"),
        "View must annotate the inner-file type mismatch:\n{stdout}"
    );
    assert!(
        stdout.contains("file1.txt"),
        "View must name the offending file:\n{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn test_sync_interactive_does_not_replace_inner_file_symlink_automatically() {
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();
    run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );

    let inner = home.path().join(".config/plaintext-dir/conf/file1.txt");
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&inner);

    let output = run_blend_with_stdin(home.path(), &orders, &["sync", "plaintext-dir"], "k\n");
    assert!(output.status.success());
    assert!(
        inner.symlink_metadata().unwrap().file_type().is_symlink(),
        "Inner-file symlink must remain after skipping the type mismatch"
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
    let (_backing_dir, backing_file, content) = replace_with_unexpected_symlink(&inner);
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
    // The backing file must remain untouched: sync must not write through the symlink.
    assert_eq!(std::fs::read_to_string(&backing_file).unwrap(), content);
}

#[test]
#[cfg(unix)]
fn test_force_target_to_source_does_not_follow_inner_file_symlink() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("plaintext-dir");
    run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );

    let source = orders_dir(blend_dir.path()).join("plaintext-dir/conf/file1.txt");
    let source_before = std::fs::read_to_string(&source).unwrap();
    let inner = home.path().join(".config/plaintext-dir/conf/file1.txt");
    let backing = TempDir::new().unwrap();
    let backing_file = backing.path().join("file1.txt");
    std::fs::write(&backing_file, "external content\n").unwrap();
    std::fs::remove_file(&inner).unwrap();
    create_symlink(&backing_file, &inner);

    let output = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-target-to-source", "plaintext-dir"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("target structure does not match Source"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), source_before);
    assert_eq!(
        std::fs::read_to_string(&backing_file).unwrap(),
        "external content\n"
    );
    assert!(inner.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
#[cfg(unix)]
fn test_managed_child_directory_symlink_uses_generic_type_reconciliation() {
    let home = TempDir::new().unwrap();
    let blend_dir = copy_fixture("plaintext-dir");
    let source_nested = orders_dir(blend_dir.path()).join("plaintext-dir/conf/nested/config.txt");
    std::fs::create_dir_all(source_nested.parent().unwrap()).unwrap();
    std::fs::write(&source_nested, "managed nested content\n").unwrap();
    run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );

    let target_nested = home.path().join(".config/plaintext-dir/conf/nested");
    std::fs::remove_dir_all(&target_nested).unwrap();
    let backing = TempDir::new().unwrap();
    std::fs::write(
        backing.path().join("config.txt"),
        "managed nested content\n",
    )
    .unwrap();
    create_symlink(backing.path(), &target_nested);

    let view = run_blend(home.path(), blend_dir.path(), &["view", "plaintext-dir"]);
    let view_stdout = String::from_utf8_lossy(&view.stdout);
    assert!(view_stdout.contains("type mismatch: expected directory, found symlink"));

    let skipped = run_blend_with_stdin(
        home.path(),
        blend_dir.path(),
        &["sync", "plaintext-dir"],
        "k\n",
    );
    assert!(skipped.status.success());
    assert!(
        target_nested
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );

    let pushed = run_blend(
        home.path(),
        blend_dir.path(),
        &["sync", "--force-source-to-target", "plaintext-dir"],
    );
    assert!(pushed.status.success());
    assert!(target_nested.symlink_metadata().unwrap().is_dir());
    assert_eq!(
        std::fs::read_to_string(target_nested.join("config.txt")).unwrap(),
        "managed nested content\n"
    );
    assert_eq!(
        std::fs::read_to_string(backing.path().join("config.txt")).unwrap(),
        "managed nested content\n"
    );
}

#[test]
#[cfg(unix)]
fn test_view_ignores_parent_symlink_and_compares_target_content() {
    // Parent components are path-resolution infrastructure. The managed leaf
    // is a regular file, so view should report only its content difference.
    let home = TempDir::new().unwrap();
    let orders = fixtures_dir();

    // Build a backing tree outside home, then symlink
    // the parent directory in.
    let backing = TempDir::new().unwrap();
    let backing_order = backing.path().join("plaintext-single");
    std::fs::create_dir_all(&backing_order).unwrap();
    std::fs::write(backing_order.join("config.txt"), "old backing content\n").unwrap();

    let parent = home.path().join(".config/plaintext-single");
    std::fs::create_dir_all(parent.parent().unwrap()).unwrap();
    create_symlink(&backing_order, &parent);

    // Sanity: target resolves through the symlink to differing content.
    let target = parent.join("config.txt");
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "old backing content\n"
    );
    assert!(
        parent.symlink_metadata().unwrap().file_type().is_symlink(),
        "parent must be a symlink for this test to be meaningful"
    );

    let output = run_blend(home.path(), &orders, &["view", "plaintext-single"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        !stdout.contains("type mismatch"),
        "view must not report the parent symlink as a target mismatch:\n{stdout}"
    );
    // And the content diff must still be shown
    assert!(
        stdout.contains("old backing content"),
        "view must still show the content diff:\n{stdout}"
    );

    let sync = run_blend(
        home.path(),
        &orders,
        &["sync", "--force-source-to-target", "plaintext-single"],
    );
    assert!(sync.status.success());
    assert!(parent.symlink_metadata().unwrap().file_type().is_symlink());
    let source_content =
        std::fs::read_to_string(fixtures_dir().join("orders/plaintext-single/config.txt")).unwrap();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), source_content);
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
    let (_backing_dir, _backing_file, _) = replace_with_unexpected_symlink(&target);

    // Status should show a generic node-type mismatch.
    let output = run_blend(home.path(), &orders, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(
        stdout.contains("mismatch"),
        "Status should show a mismatch for the symlink target:\n{stdout}"
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
fn test_blend_order_rejects_stale_blend_dir_config_field() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();

    let init_output = run_blend_in_cwd(home.path(), cwd.path(), &["init"]);
    assert!(
        init_output.status.success(),
        "blend init should bootstrap an empty cwd\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&init_output.stdout),
        String::from_utf8_lossy(&init_output.stderr),
    );

    let blend_order = cwd.path().join("orders/blend/order.ncl");
    let raw = std::fs::read_to_string(&blend_order).unwrap();
    let stale = raw.replace(
        "from_config = {\n          sandbox = \"prefer\",",
        "from_config = {\n          blend_dir = \"/tmp/stale-dotfiles\",\n          sandbox = \"prefer\",",
    );
    std::fs::write(&blend_order, stale).unwrap();

    let output = run_blend_in_cwd(home.path(), cwd.path(), &["view", "blend"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("extra field `blend_dir`") || stderr.contains("extra field `blend_dir`"),
        "expected Nickel contract error for stale blend_dir\nstdout: {stdout}\nstderr: {stderr}",
    );
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
        "blend view should use remembered blend dir state outside a checkout\nstdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&view_output.stderr),
    );
    assert!(
        stdout.contains("blend") && !stdout.contains("toml-basic"),
        "blend view should use the configured temp checkout, not the executable checkout\nstdout: {stdout}",
    );
}

#[test]
fn test_read_command_uses_current_cwd_without_refreshing_stale_remembered_blend_dir() {
    let home = TempDir::new().unwrap();
    let stale = copy_fixture("plaintext-single");
    let current = copy_fixture("toml-basic");

    // Seed a stale blend dir into state so it differs from the cwd checkout.
    let state_dir = home.path().join(".local/state/blend");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::write(
        state_dir.join("state.json"),
        format!("{{\"blend_dir\":\"{}\"}}", stale.path().display()),
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
        stdout.contains("differs from remembered blend-dir"),
        "expected mismatch warning\nstdout: {stdout}",
    );

    let state = std::fs::read_to_string(state_dir.join("state.json")).unwrap();
    assert!(
        state.contains(&stale.path().display().to_string()),
        "read command should not refresh remembered blend dir, got:\n{state}",
    );
}

#[test]
fn test_mutating_command_refreshes_stale_remembered_blend_dir() {
    let home = TempDir::new().unwrap();
    let stale = copy_fixture("plaintext-single");
    let current = copy_fixture("toml-basic");

    // Seed a stale blend dir into state so it differs from the cwd checkout.
    let state_dir = home.path().join(".local/state/blend");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::write(
        state_dir.join("state.json"),
        format!("{{\"blend_dir\":\"{}\"}}", stale.path().display()),
    )
    .unwrap();

    let output = run_blend_in_cwd(home.path(), current.path(), &["format", "toml-basic"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "blend format should use the current checkout and succeed\nstdout: {stdout}\nstderr: {stderr}",
    );
    assert!(
        stdout.contains("differs from remembered blend-dir"),
        "expected mismatch warning\nstdout: {stdout}",
    );

    let state = std::fs::read_to_string(state_dir.join("state.json")).unwrap();
    assert!(
        state.contains(&current.path().display().to_string()),
        "mutating command should refresh state to current checkout, got:\n{state}",
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
