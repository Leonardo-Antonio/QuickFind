#!/usr/bin/env bash
set -Eeuo pipefail

APP_NAME="QuickFind"
BIN_NAME="quickfind"
DEFAULT_REPO="tu-usuario/QuickFind"

GITHUB_REPO="${QUICKFIND_REPO:-$DEFAULT_REPO}"
VERSION="${QUICKFIND_VERSION:-latest}"
PREFIX="${PREFIX:-/usr/local}"
BINDIR="${BINDIR:-$PREFIX/bin}"
DESKTOPDIR="${DESKTOPDIR:-$PREFIX/share/applications}"
DOWNLOAD_URL="${QUICKFIND_URL:-}"
LOCAL_BINARY=""
INSTALL_DEPS=1
UNINSTALL=0
PACKAGE=0
TMP_DIRS=()

cleanup() {
    local dir
    for dir in "${TMP_DIRS[@]:-}"; do
        rm -rf "$dir"
    done
}
trap cleanup EXIT

usage() {
    cat <<EOF
QuickFind Linux installer

Usage:
  ./install.sh [options]

Install from GitHub Releases:
  ./install.sh --repo owner/QuickFind
  ./install.sh --repo owner/QuickFind --version v1.0.0

Install a local binary:
  ./install.sh --local ./target/release/quickfind

Create the release tarball:
  cargo build --release --locked
  ./install.sh --package

Options:
  --repo owner/repo       GitHub repository that hosts releases.
                          Default: $DEFAULT_REPO
  --version tag           Release tag to install. Default: latest
  --url url               Download a release tarball from a custom URL.
  --local path            Install an already-built quickfind binary.
  --prefix path           Installation prefix. Default: /usr/local
  --no-deps               Do not try to install runtime packages.
  --uninstall             Remove installed files.
  --package               Create dist/quickfind-linux-ARCH.tar.gz.
  -h, --help              Show this help.

Environment:
  QUICKFIND_REPO=owner/repo
  QUICKFIND_VERSION=v1.0.0
  QUICKFIND_URL=https://example.com/quickfind-linux-x86_64.tar.gz
  PREFIX=/usr/local
EOF
}

log() {
    printf '\033[1;34m==>\033[0m %s\n' "$*"
}

warn() {
    printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2
}

die() {
    printf '\033[1;31merror:\033[0m %s\n' "$*" >&2
    exit 1
}

have() {
    command -v "$1" >/dev/null 2>&1
}

as_root() {
    if [ "${EUID:-$(id -u)}" -eq 0 ]; then
        "$@"
    elif "$@" >/dev/null 2>&1; then
        return 0
    elif have sudo; then
        sudo "$@"
    elif have doas; then
        doas "$@"
    else
        die "need sudo/doas to install into $PREFIX"
    fi
}

make_tmpdir() {
    local dir
    dir="$(mktemp -d)"
    TMP_DIRS+=("$dir")
    printf '%s' "$dir"
}

parse_args() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --repo)
                GITHUB_REPO="${2:-}"
                shift 2
                ;;
            --version)
                VERSION="${2:-}"
                shift 2
                ;;
            --url)
                DOWNLOAD_URL="${2:-}"
                shift 2
                ;;
            --local)
                LOCAL_BINARY="${2:-}"
                shift 2
                ;;
            --prefix)
                PREFIX="${2:-}"
                BINDIR="$PREFIX/bin"
                DESKTOPDIR="$PREFIX/share/applications"
                shift 2
                ;;
            --no-deps)
                INSTALL_DEPS=0
                shift
                ;;
            --uninstall)
                UNINSTALL=1
                shift
                ;;
            --package)
                PACKAGE=1
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                die "unknown option: $1"
                ;;
        esac
    done
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) printf 'x86_64' ;;
        aarch64|arm64) printf 'aarch64' ;;
        *) die "unsupported architecture: $(uname -m). Build QuickFind locally and install with ./install.sh --local ./target/release/quickfind" ;;
    esac
}

detect_pm() {
    if have apt-get; then printf 'apt'; return; fi
    if have dnf; then printf 'dnf'; return; fi
    if have pacman; then printf 'pacman'; return; fi
    if have zypper; then printf 'zypper'; return; fi
    if have xbps-install; then printf 'xbps'; return; fi
    if have apk; then printf 'apk'; return; fi
    printf 'unknown'
}

install_required_packages() {
    [ "$INSTALL_DEPS" -eq 1 ] || return 0

    local pm
    pm="$(detect_pm)"

    log "Installing runtime dependencies for $pm"
    case "$pm" in
        apt)
            as_root apt-get update
            as_root apt-get install -y libgtk-4-1 xdg-utils coreutils bash ca-certificates curl tar
            ;;
        dnf)
            as_root dnf install -y gtk4 xdg-utils coreutils bash ca-certificates curl tar
            ;;
        pacman)
            as_root pacman -Sy --needed --noconfirm gtk4 xdg-utils coreutils bash ca-certificates curl tar
            ;;
        zypper)
            as_root zypper --non-interactive install gtk4 xdg-utils coreutils bash ca-certificates curl tar
            ;;
        xbps)
            as_root xbps-install -Sy gtk4 xdg-utils coreutils bash ca-certificates curl tar
            ;;
        apk)
            warn "Alpine uses musl. The official glibc release binary may not run there."
            as_root apk add --no-cache gtk4.0 xdg-utils coreutils bash ca-certificates curl tar
            ;;
        unknown)
            warn "No supported package manager found. Install GTK 4.12+, xdg-utils, coreutils, bash, curl and tar manually."
            ;;
    esac
}

install_optional_packages_hint() {
    cat <<'EOF'

Optional features:
  - Clipboard history needs cliphist and wl-clipboard.
    Start the history daemon with:
      wl-paste --watch cliphist store
  - Terminal .desktop entries use $TERMINAL, or kitty if $TERMINAL is unset.
    Install a terminal emulator or export TERMINAL=your-terminal.

EOF
}

check_runtime() {
    local missing=0

    for cmd in sh date xdg-open; do
        if ! have "$cmd"; then
            warn "missing runtime command: $cmd"
            missing=1
        fi
    done

    if have pkg-config; then
        if pkg-config --exists "gtk4 >= 4.12"; then
            log "GTK metadata reports $(pkg-config --modversion gtk4)"
        else
            warn "pkg-config did not find gtk4 >= 4.12. QuickFind requires GTK 4.12 or newer."
        fi
    else
        warn "pkg-config is not installed; skipping GTK version check."
    fi

    if ! have cliphist || ! have wl-copy; then
        warn "clipboard history is optional, but needs cliphist and wl-clipboard/wl-copy"
    fi

    return "$missing"
}

release_url() {
    local arch asset base
    arch="$(detect_arch)"
    asset="quickfind-linux-${arch}.tar.gz"

    if [ -n "$DOWNLOAD_URL" ]; then
        printf '%s' "$DOWNLOAD_URL"
        return
    fi

    if [ "$GITHUB_REPO" = "$DEFAULT_REPO" ]; then
        die "set your GitHub repo first: ./install.sh --repo owner/QuickFind"
    fi

    base="https://github.com/${GITHUB_REPO}/releases"
    if [ "$VERSION" = "latest" ]; then
        printf '%s/latest/download/%s' "$base" "$asset"
    else
        printf '%s/download/%s/%s' "$base" "$VERSION" "$asset"
    fi
}

download() {
    local url="$1" out="$2"
    if have curl; then
        curl -fL --retry 3 --connect-timeout 20 -o "$out" "$url"
    elif have wget; then
        wget -O "$out" "$url"
    else
        die "curl or wget is required to download QuickFind"
    fi
}

write_desktop_file() {
    local path="$1"
    cat >"$path" <<'EOF'
[Desktop Entry]
Name=QuickFind
Comment=Fast application launcher
Exec=quickfind
Type=Application
Terminal=false
Categories=System;Utility;
Keywords=launcher;spotlight;rofi;search;applications;
Icon=system-search
EOF
}

find_binary_in_archive() {
    local dir="$1"
    find "$dir" -type f -name "$BIN_NAME" -perm -u+x | head -n 1
}

install_files() {
    local binary="$1" desktop="$2"

    [ -f "$binary" ] || die "binary not found: $binary"
    chmod +x "$binary"

    log "Installing $BIN_NAME to $BINDIR"
    as_root install -Dm755 "$binary" "$BINDIR/$BIN_NAME"

    log "Installing desktop entry to $DESKTOPDIR"
    as_root install -Dm644 "$desktop" "$DESKTOPDIR/$BIN_NAME.desktop"

    if have update-desktop-database; then
        as_root update-desktop-database "$(dirname "$DESKTOPDIR")" >/dev/null 2>&1 || true
    fi

    if have ldd; then
        if ldd "$BINDIR/$BIN_NAME" 2>/dev/null | grep -q "not found"; then
            warn "some shared libraries are missing:"
            ldd "$BINDIR/$BIN_NAME" | grep "not found" >&2 || true
        fi
    fi
}

install_from_local_binary() {
    local tmp desktop
    [ -n "$LOCAL_BINARY" ] || return 1
    [ -f "$LOCAL_BINARY" ] || die "local binary not found: $LOCAL_BINARY"

    tmp="$(make_tmpdir)"
    desktop="$tmp/$BIN_NAME.desktop"
    write_desktop_file "$desktop"
    install_files "$LOCAL_BINARY" "$desktop"
}

install_from_release() {
    local tmp archive url binary desktop
    tmp="$(make_tmpdir)"

    url="$(release_url)"
    archive="$tmp/quickfind.tar.gz"

    log "Downloading $url"
    download "$url" "$archive"

    tar -xzf "$archive" -C "$tmp"
    binary="$(find_binary_in_archive "$tmp")"
    [ -n "$binary" ] || die "release archive does not contain an executable named $BIN_NAME"

    desktop="$(find "$tmp" -type f -name "$BIN_NAME.desktop" | head -n 1)"
    if [ -z "$desktop" ]; then
        desktop="$tmp/$BIN_NAME.desktop"
        write_desktop_file "$desktop"
    fi

    install_files "$binary" "$desktop"
}

package_release() {
    local arch package_dir archive
    arch="$(detect_arch)"
    package_dir="dist/quickfind-linux-${arch}"
    archive="dist/quickfind-linux-${arch}.tar.gz"

    [ -f "target/release/$BIN_NAME" ] || die "missing target/release/$BIN_NAME. Run: cargo build --release --locked"
    [ -f "$BIN_NAME.desktop" ] || die "missing $BIN_NAME.desktop"

    rm -rf "$package_dir" "$archive"
    mkdir -p "$package_dir"
    install -Dm755 "target/release/$BIN_NAME" "$package_dir/$BIN_NAME"
    install -Dm644 "$BIN_NAME.desktop" "$package_dir/$BIN_NAME.desktop"
    tar -C "$package_dir" -czf "$archive" "$BIN_NAME" "$BIN_NAME.desktop"

    log "Created $archive"
    tar -tzf "$archive"
}

uninstall() {
    log "Removing $BINDIR/$BIN_NAME"
    as_root rm -f "$BINDIR/$BIN_NAME"

    log "Removing $DESKTOPDIR/$BIN_NAME.desktop"
    as_root rm -f "$DESKTOPDIR/$BIN_NAME.desktop"

    if have update-desktop-database; then
        as_root update-desktop-database "$(dirname "$DESKTOPDIR")" >/dev/null 2>&1 || true
    fi
}

main() {
    parse_args "$@"

    if [ "$PACKAGE" -eq 1 ]; then
        package_release
        exit 0
    fi

    if [ "$UNINSTALL" -eq 1 ]; then
        uninstall
        exit 0
    fi

    if [ "$(uname -s)" != "Linux" ]; then
        die "$APP_NAME only supports Linux"
    fi

    install_required_packages
    check_runtime || true

    if [ -n "$LOCAL_BINARY" ]; then
        install_from_local_binary
    else
        install_from_release
    fi

    install_optional_packages_hint
    log "$APP_NAME installed. Run: $BIN_NAME"
}

main "$@"
