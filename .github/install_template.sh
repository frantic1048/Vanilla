#!/bin/sh
# shellcheck shell=dash
# shellcheck disable=SC2039  # local is non-POSIX but universally available
set -eu

# blend installer
# Template lives in-repo; CI generates the release version with embedded
# checksums and version number. Do not edit the __PLACEHOLDER__ values by hand.
# https://github.com/frantic1048/Vanilla

APP_NAME="blend"
APP_VERSION="__APP_VERSION__"
REPO="frantic1048/Vanilla"

# SHA256 checksums embedded at CI generation time
CHECKSUM_AARCH64_APPLE="__CHECKSUM_AARCH64_APPLE_DARWIN__"
CHECKSUM_X86_64_APPLE="__CHECKSUM_X86_64_APPLE_DARWIN__"
CHECKSUM_X86_64_LINUX="__CHECKSUM_X86_64_UNKNOWN_LINUX_GNU__"

QUIET=0
INSTALL_DIR="${HOME}/.local/bin"

usage() {
    cat <<EOF
blend-installer: install blend to ~/.local/bin

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    --dir <DIR>     Install to a custom directory instead of ~/.local/bin
    --quiet         Suppress non-error output
    --help          Show this help message
EOF
}

main() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --dir)
                shift
                [ $# -gt 0 ] || err "--dir requires a directory argument"
                INSTALL_DIR="$1"
                ;;
            --quiet)
                QUIET=1
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                err "unknown option: $1"
                ;;
        esac
        shift
    done

    local _os _arch _target _archive _checksum _url

    detect_platform
    _os="$RETVAL_OS"
    _arch="$RETVAL_ARCH"
    _target="${_arch}-${_os}"

    _archive="${APP_NAME}-${_target}.tar.xz"
    _url="https://github.com/${REPO}/releases/download/blend-v${APP_VERSION}/${_archive}"

    case "$_target" in
        aarch64-apple-darwin)   _checksum="$CHECKSUM_AARCH64_APPLE" ;;
        x86_64-apple-darwin)    _checksum="$CHECKSUM_X86_64_APPLE" ;;
        x86_64-unknown-linux-gnu) _checksum="$CHECKSUM_X86_64_LINUX" ;;
        *)                      err "unsupported target: $_target" ;;
    esac

    ensure_cmd curl
    ensure_cmd tar

    local _tmpdir
    _tmpdir="$(mktemp -d)" || err "failed to create temp directory"
    # shellcheck disable=SC2064
    trap "rm -rf '$_tmpdir'" EXIT

    say "downloading blend v${APP_VERSION} for ${_target}..."
    download "$_url" "$_tmpdir/$_archive"

    say "verifying checksum..."
    verify_sha256 "$_tmpdir/$_archive" "$_checksum"

    say "extracting..."
    tar xf "$_tmpdir/$_archive" -C "$_tmpdir"

    mkdir -p "$INSTALL_DIR"
    install -m 755 "$_tmpdir/${APP_NAME}-${_target}/${APP_NAME}" "$INSTALL_DIR/${APP_NAME}"

    say "installed blend to ${INSTALL_DIR}/${APP_NAME}"
    say ""
    say "to uninstall: rm ${INSTALL_DIR}/${APP_NAME}"

    check_path
}

detect_platform() {
    local _uname_os _uname_arch

    _uname_os="$(uname -s)"
    case "$_uname_os" in
        Linux)  RETVAL_OS="unknown-linux-gnu" ;;
        Darwin) RETVAL_OS="apple-darwin" ;;
        *)      err "unsupported OS: $_uname_os (only Linux and macOS are supported)" ;;
    esac

    _uname_arch="$(uname -m)"
    case "$_uname_arch" in
        x86_64 | amd64)    RETVAL_ARCH="x86_64" ;;
        aarch64 | arm64)   RETVAL_ARCH="aarch64" ;;
        *)                 err "unsupported architecture: $_uname_arch" ;;
    esac
}

download() {
    local _url="$1" _output="$2"
    if ! curl --proto '=https' --tlsv1.2 -fsSL "$_url" -o "$_output"; then
        err "failed to download ${_url}"
    fi
}

verify_sha256() {
    local _file="$1" _expected="$2" _actual

    if check_cmd sha256sum; then
        _actual="$(sha256sum "$_file" | cut -d' ' -f1)"
    elif check_cmd shasum; then
        _actual="$(shasum -a 256 "$_file" | cut -d' ' -f1)"
    else
        warn "neither sha256sum nor shasum found — skipping checksum verification"
        return
    fi

    if [ "$_actual" != "$_expected" ]; then
        err "checksum mismatch!
  expected: $_expected
  actual:   $_actual
This may indicate a corrupted download or a tampered file."
    fi

    say "checksum verified."
}

check_path() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            return
            ;;
    esac

    local _shell_name _rc_file _line
    _shell_name="$(basename "${SHELL:-/bin/sh}")"
    _line="export PATH=\"${INSTALL_DIR}:\$PATH\""

    case "$_shell_name" in
        bash) _rc_file="~/.bashrc" ;;
        zsh)  _rc_file="~/.zshrc" ;;
        fish)
            _line="fish_add_path \"${INSTALL_DIR}\""
            _rc_file="~/.config/fish/config.fish"
            ;;
        *)    _rc_file="your shell's config file" ;;
    esac

    warn "${INSTALL_DIR} is not on your PATH.
  Add it by running:

    echo '${_line}' >> ${_rc_file}

  Then restart your shell."
}

check_cmd() { command -v "$1" >/dev/null 2>&1; }

ensure_cmd() {
    if ! check_cmd "$1"; then
        err "'$1' is required but not found"
    fi
}

say() {
    if [ "$QUIET" = "0" ]; then
        printf 'blend-installer: %s\n' "$1"
    fi
}

warn() { printf 'blend-installer: WARNING: %s\n' "$1" >&2; }
err()  { printf 'blend-installer: ERROR: %s\n' "$1" >&2; exit 1; }

main "$@"
