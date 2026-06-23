//! Contract versioning and migration helpers.
//!
//! The `order.contract.ncl` file contains a `contract_version` field that
//! tracks breaking and non-breaking schema evolution. When blend detects a
//! user's contract is older than the current version, it consults the
//! [`MIGRATIONS`] manifest to decide whether the upgrade is safe to apply
//! automatically (`sync`) or requires explicit user action (`init --upgrade`).

use crate::output::log;

/// The contract version that this build of blend expects.
pub const CURRENT_CONTRACT_VERSION: u32 = 2;

/// A single migration step between two adjacent contract versions.
struct Migration {
    from: u32,
    to: u32,
    breaking: bool,
    hints: &'static [&'static str],
}

/// Ordered list of all migration steps. Each entry covers `from` → `to`
/// (always `to == from + 1`). When jumping multiple versions, walk the
/// slice sequentially.
const MIGRATIONS: &[Migration] = &[Migration {
    from: 1,
    to: 2,
    breaking: true,
    hints: &[
        "blend_dir removed from config.toml — now stored in state.json",
        "Remove `blend_dir` and `ignore` fields from orders/blend/order.ncl if present",
        "BlendOrder contract now enforces known config keys (sandbox only)",
    ],
}];

/// Outcome of comparing the user's contract version against the current one.
#[derive(Debug, PartialEq, Eq)]
pub enum MigrationCheck {
    /// Already up to date — no action needed.
    UpToDate,
    /// Upgrade path exists and contains only non-breaking changes.
    /// Safe to apply automatically during `sync`.
    NonBreaking { from: u32, to: u32 },
    /// Upgrade path contains at least one breaking change.
    /// Requires explicit `init --upgrade`.
    Breaking { from: u32, to: u32 },
}

/// Check whether a migration is needed and classify it.
pub fn check(user_version: u32) -> MigrationCheck {
    if user_version >= CURRENT_CONTRACT_VERSION {
        return MigrationCheck::UpToDate;
    }

    let has_breaking = MIGRATIONS
        .iter()
        .filter(|m| m.from >= user_version && m.to <= CURRENT_CONTRACT_VERSION)
        .any(|m| m.breaking);

    if has_breaking {
        MigrationCheck::Breaking {
            from: user_version,
            to: CURRENT_CONTRACT_VERSION,
        }
    } else {
        MigrationCheck::NonBreaking {
            from: user_version,
            to: CURRENT_CONTRACT_VERSION,
        }
    }
}

/// Collect all migration hints for the path from `user_version` to current.
pub fn hints_for(user_version: u32) -> Vec<&'static str> {
    MIGRATIONS
        .iter()
        .filter(|m| m.from >= user_version && m.to <= CURRENT_CONTRACT_VERSION)
        .flat_map(|m| m.hints.iter().copied())
        .collect()
}

/// Print migration hints to the user.
pub fn print_migration_hints(user_version: u32) {
    let hints = hints_for(user_version);
    if hints.is_empty() {
        return;
    }
    log::warn(&format!(
        "Contract upgrade v{user_version} → v{CURRENT_CONTRACT_VERSION}:"
    ));
    for hint in &hints {
        log::warn(&format!("  • {hint}"));
    }
}

/// Extract `contract_version` from a contract file's content.
/// Looks for a line matching `contract_version = <integer>,` in the Nickel
/// source. Returns `1` if not found (implicit v1 for files written before
/// versioning was introduced).
pub fn read_contract_version(content: &str) -> u32 {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("contract_version") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(v) = num_str.parse::<u32>() {
                    return v;
                }
            }
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_up_to_date() {
        assert_eq!(check(CURRENT_CONTRACT_VERSION), MigrationCheck::UpToDate);
        assert_eq!(
            check(CURRENT_CONTRACT_VERSION + 1),
            MigrationCheck::UpToDate
        );
    }

    #[test]
    fn check_breaking_from_v1() {
        assert_eq!(
            check(1),
            MigrationCheck::Breaking {
                from: 1,
                to: CURRENT_CONTRACT_VERSION
            }
        );
    }

    #[test]
    fn hints_for_v1_is_nonempty() {
        let hints = hints_for(1);
        assert!(!hints.is_empty());
        assert!(hints.iter().any(|h| h.contains("blend_dir")));
    }

    #[test]
    fn read_version_from_content() {
        let content = "{\n  contract_version = 2,\n  Format = ...\n}";
        assert_eq!(read_contract_version(content), 2);
    }

    #[test]
    fn read_version_missing_defaults_to_1() {
        let content = "{\n  Format = ...\n}";
        assert_eq!(read_contract_version(content), 1);
    }

    #[test]
    fn read_version_handles_whitespace() {
        let content = "{\n  contract_version  =  3 ,\n}";
        assert_eq!(read_contract_version(content), 3);
    }
}
