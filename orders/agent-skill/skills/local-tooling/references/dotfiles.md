# Vanilla Dotfiles And Blend

## Source Of Truth

- The user's dotfiles repo is `~/Vanilla`, hosted as `frantic1048/Vanilla`.
- The repo is managed by the user's own tool `blend`, implemented in `~/Vanilla/blend`.
- Most config packages live under `~/Vanilla/orders/<order-name>/`.
- User scripts are physically sourced from `~/Vanilla/orders/bin/bin`; `~/Vanilla/bin` is a repo-level symlink kept as the stable entrypoint and contains `bin/blend` as a symlink to the built Blend binary.
- Each order's deployment shape is described by its `order.ncl`.
- Read `~/Vanilla/blend/README.md`, `~/Vanilla/NEW_BLEND.md`, and `~/Vanilla/blend/src/` when deeper Blend behavior matters.

## Rendered Versus Symlinked Config

Blend supports both render/copy-to-target and symlink-to-target modes.

- Most orders use render/copy-to-target mode.
- Rendered target files can drift from the source files in `~/Vanilla`.
- Symlinked entries can be identified in `order.ncl` with `symlink = true` or by `blend view` output showing a symlink relationship.
- Do not assume source files and live system config are identical. Verify the order and target relationship first.

## Inspecting Orders

Prefer read-only inspection:

```sh
cd ~/Vanilla
blend view
blend view nushell
```

- Run `blend view` with `PWD=~/Vanilla` when possible. Outside the repo, Blend may use its state directory from the last successful run to locate orders, which is useful but less explicit.
- `blend view` without an order name lists all discovered orders and their source-to-target relationships.
- `blend view <order-name>` focuses one order.
- The output shows target paths, symlink status, no-change status, and source/target diffs.
- In managed sandboxed sessions, `blend view` may print a sandbox initialization warning and still exit successfully with useful output. Treat this as a warning, not a failure, when the command exit code is zero.

## Safety Boundary

Use only `blend view` for routine inspection.

Do not run other Blend commands unless the user explicitly asks, because other commands may write to target config, update state, create snapshots, sync, or otherwise mutate the system.

## Reading `order.ncl`

When `blend view` is unavailable or more detail is needed, read `~/Vanilla/orders/<order-name>/order.ncl`.

Useful fields:

- `prefix`: target directory prefix, often OS-specific.
- `files`: deployed files or directories.
- `from_file`: source file or directory inside the order.
- `from_config`: inline structured config rendered to a target file.
- `name`: target filename override.
- `when`: OS, architecture, hostname, or similar condition.
- `symlink`: whether the target should be a symlink instead of rendered/copied content.
- `exclude`: files skipped when deploying a source directory.
