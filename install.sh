#!/bin/sh
# pack installer — downloads the latest release binary from GitHub.
#
#   curl -fsSL https://raw.githubusercontent.com/maralla/pack/master/install.sh | sh
#
# What this does:
#   1. picks the release target for this system (musl build on a musl
#      userland, gnu build on glibc)
#   2. resolves the latest release version
#   3. on musl systems: installs the runtime libraries pack resolves
#      against (the system crypto stack — that is the point of not
#      vendoring OpenSSL)
#   4. downloads, unpacks, installs the binary
#
# Transport is GitHub TLS; the release carries no separate checksums, so
# this protects the download channel, not the artifact chain.
set -eu

repo=maralla/pack
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# fetch URL [OUTPUT] — curl where available, busybox wget otherwise.
fetch() {
    if command -v curl >/dev/null 2>&1; then
        if [ $# -ge 2 ]; then curl -fsL -o "$2" "$1"; else curl -fsL "$1"; fi
    else
        if [ $# -ge 2 ]; then wget -q -O "$2" "$1"; else wget -qO- "$1"; fi
    fi
}

# ── 1. Pick the release target ───────────────────────────────────────
os=$(uname -s)
arch=$(uname -m)
case "$os/$arch" in
    Darwin/arm64)  target=aarch64-apple-darwin ;;
    Darwin/x86_64) target=x86_64-apple-darwin ;;
    Linux/x86_64)  target=x86_64-unknown-linux-gnu ;;
    Linux/aarch64) target=aarch64-unknown-linux-musl ;;
    *)
        echo "pack: unsupported system: $os/$arch" >&2
        exit 1
        ;;
esac

# A musl userland runs the musl build (its loader is present); glibc keeps
# the gnu build. The musl build resolves against the system's shared
# libraries — step 3 is what makes it run.
if [ "$os" = Linux ] && [ -e "/lib/ld-musl-$arch.so.1" ]; then
    case "$arch" in
        x86_64|aarch64) target=$(echo "$target" | sed 's/-gnu$/-musl/') ;;
    esac
elif [ "$os" = Linux ] && [ "$arch" = aarch64 ]; then
    echo "pack: aarch64 glibc systems get the musl build; it needs a musl" >&2
    echo "      userland (ld-musl-aarch64.so.1) to run" >&2
fi

# ── 2. Resolve the latest release ────────────────────────────────────
version=$(fetch "https://api.github.com/repos/$repo/releases/latest" |
    sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
[ -n "$version" ] || { echo "pack: cannot resolve latest release" >&2; exit 1; }

echo "==> pack $version ($target)"

# ── 3. Runtime libraries on musl systems ─────────────────────────────
# pack links libssl/libcrypto/libz and the unwinder; on Alpine those are
# apk packages (libgcc was the one actually missing in testing).
case "$target" in
*musl*)
    if command -v apk >/dev/null 2>&1; then
        echo "==> installing runtime libraries"
        apk add --no-cache libgcc libssl3 libcrypto3 zlib
    else
        echo "pack: musl build needs libssl/libcrypto/libz/libgcc_s present" >&2
    fi
    ;;
esac

# ── 4. Download, unpack, install ─────────────────────────────────────
url="https://github.com/$repo/releases/download/$version/pack-$version-$target.tar.gz"
fetch "$url" "$tmp/pack.tar.gz"
tar xzf "$tmp/pack.tar.gz" -C "$tmp"

dest=/usr/local/bin
mkdir -p /usr/local/bin 2>/dev/null || true
[ -w /usr/local/bin ] || dest=$HOME/.local/bin
mkdir -p "$dest"
cp "$tmp/pack" "$dest/pack"
chmod 755 "$dest/pack"

echo "==> installed $dest/pack"
[ "$dest" = /usr/local/bin ] ||
    echo "    note: add $dest to PATH" >&2
echo "    shell completions are in the archive's contrib/ directory"

"$dest/pack" --version
