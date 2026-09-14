# blend

Blend is a cross-platform dotfile manager for composing configuration in
[Nickel](https://nickel-lang.org/) and reconciling it with files used by
applications.

An `order.ncl` file declares one or more target files. Blend can copy literal
files and directories, or render structured Nickel values as TOML, JSON, JSONC,
YAML, and simple line-oriented formats. It compares structured files by value,
supports target-aware declarative policies, and falls back to interactive
resolution when the declaration does not decide a difference.

## Why Blend

- Keep platform logic in Nickel instead of embedding template syntax in target
  file formats.
- Mix structured `from_config` entries with faithful `from_file` copies.
- Describe ownership, defaults, enforcement, absence, and assertions close to
  the fields they affect.
- Preview changes and reconcile in either direction, with snapshot-backed
  Source/Target/Base context.
- Leave unresolved or application-owned state to an explicit interactive or
  unmanaged path.

Blend aims to satisfy declared constraints at best effort: automatically when
the declaration is authoritative, interactively when either side may be right,
and with a clear failure when an assertion or input cannot be satisfied safely.

## Install

### Homebrew

```sh
brew install frantic1048/tap/blend
```

### Installer

Release archives are published from
[frantic1048/Vanilla](https://github.com/frantic1048/Vanilla/releases). The
installer selects the current platform archive, verifies its embedded SHA256
checksum, and installs to `~/.local/bin` by default:

```sh
curl -fsSLO https://github.com/frantic1048/Vanilla/releases/latest/download/blend-installer.sh
sh blend-installer.sh
```

Use `sh blend-installer.sh --dir /path/to/bin` to choose another directory.
Published targets are:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`

### Container

```sh
docker run --rm ghcr.io/frantic1048/blend:latest --version
docker run --rm \
  -v "$PWD:/workspace" \
  -w /workspace \
  ghcr.io/frantic1048/blend:latest check
```

The image entrypoint is `blend`. Stable releases publish both a version tag and
`latest`; prereleases publish only their version tag.

### From source

```sh
git clone https://github.com/frantic1048/Vanilla.git
cd Vanilla/blend
cargo build --release
```

The workspace writes the binary to `../target/release/blend`.

## Quick start

Create a directory for Blend Source and initialize it:

```sh
mkdir dotfiles
cd dotfiles
blend init
```

`blend init` creates the generated Order contract, metadata defaults, and a
starter Blend configuration. Import an existing target file or directory as a
literal source entry:

```sh
blend add git ~/.config/git
```

For a structured configuration, create an order and edit its `order.ncl`:

```sh
blend create editor
```

A small structured order looks like this:

```nickel
let { Order, .. } = import "../order.contract.ncl" in
{
  blend = {
    prefix = ["~/.config/example/"],
    files = [
      {
        name = "settings.toml",
        from_config =
          {
            theme = "dark",
            timeout =
              fun { target } =>
                if target == 'Absent then 30 else target,
          }
          |> blend.with_target_only 'Unmanaged,
      },
    ],
  },
} | Order
```

Here `theme` uses ordinary interactive reconciliation when it differs,
`timeout` defaults to `30` only when absent, and undeclared target fields are
outside Blend ownership.

Validate, preview, and reconcile:

```sh
blend check
blend view
blend sync
```

Running `blend` without a subcommand shows deployment status. Dry-run output is
command-specific: `blend sync --dry-run` reports prospective reconciliation,
while `blend init --dry-run` only checks that generated Order files are current
and does not preview replacement content.

## Documentation

- [GUIDE.md](docs/GUIDE.md) — Orders, formats, resolution semantics, workflows,
  safety boundaries, and troubleshooting.
- [DEVELOPMENT.md](docs/DEVELOPMENT.md) — architecture, source map, tests, CI,
  and release maintenance.
- [DESIGN.md](docs/DESIGN.md) — durable design rationale, trade-offs, and scope.
- [CHANGELOG.md](CHANGELOG.md) — release history.

Blend is currently developed and released from the Vanilla repository. Vanilla
is also its primary real-world consumer and integration corpus.
