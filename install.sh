#!/bin/sh
# aitop installer — https://github.com/bugkill3r/aitop
# Usage: curl -fsSL https://raw.githubusercontent.com/bugkill3r/aitop/master/install.sh | sh
set -e

REPO="bugkill3r/aitop"
INSTALL_DIR="${AITOP_INSTALL_DIR:-/usr/local/bin}"

main() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) os_target="apple-darwin" ;;
        Linux)  os_target="unknown-linux-gnu" ;;
        *)
            echo "Error: unsupported OS: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch_target="x86_64" ;;
        aarch64|arm64)   arch_target="aarch64" ;;
        *)
            echo "Error: unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    target="${arch_target}-${os_target}"

    echo "Detecting platform: ${os} ${arch} -> ${target}"

    # Fetch latest release tag
    tag="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | cut -d'"' -f4)"

    if [ -z "$tag" ]; then
        echo "Error: could not determine latest release" >&2
        exit 1
    fi

    echo "Latest release: ${tag}"

    tarball="aitop-${tag}-${target}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${tag}/${tarball}"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "Downloading ${url}..."
    curl -fsSL "$url" -o "${tmpdir}/${tarball}"

    echo "Extracting..."
    tar xzf "${tmpdir}/${tarball}" -C "$tmpdir"

    # Install binary
    if [ -w "$INSTALL_DIR" ]; then
        mv "${tmpdir}/aitop" "${INSTALL_DIR}/aitop"
    else
        echo "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "${tmpdir}/aitop" "${INSTALL_DIR}/aitop"
    fi

    chmod +x "${INSTALL_DIR}/aitop"

    echo ""
    echo "aitop ${tag} installed to ${INSTALL_DIR}/aitop"
    echo "Run 'aitop' to get started."
}

main
