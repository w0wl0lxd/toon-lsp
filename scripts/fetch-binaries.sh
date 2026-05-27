#!/usr/bin/env bash
# Fetch pre-built toon-lsp binaries for all platforms (development helper)
# Usage: ./scripts/fetch-binaries.sh [version]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Version to fetch (default: latest)
VERSION="${1:-latest}"
REPO="toon-format/toon-lsp"

if [[ "$VERSION" == "latest" ]]; then
    log_info "Fetching latest release version..."
    VERSION=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [[ -z "$VERSION" ]]; then
        log_error "Could not determine latest version"
        exit 1
    fi
fi

log_info "Fetching toon-lsp version: $VERSION"

# Platform targets
declare -A TARGETS=(
    ["win32-x64"]="x86_64-pc-windows-msvc"
    ["darwin-x64"]="x86_64-apple-darwin"
    ["darwin-arm64"]="aarch64-apple-darwin"
    ["linux-x64"]="x86_64-unknown-linux-gnu"
)

# Download and extract for each platform
for platform in "${!TARGETS[@]}"; do
    target="${TARGETS[$platform]}"
    binary_name="toon-lsp"
    archive_ext="tar.gz"

    if [[ "$platform" == "win32-x64" ]]; then
        binary_name="toon-lsp.exe"
        archive_ext="zip"
    fi

    url="https://github.com/$REPO/releases/download/$VERSION/toon-lsp-$VERSION-$target.$archive_ext"
    dest_dir="$ROOT_DIR/editors/vscode/binaries/$platform"

    log_info "Downloading $platform from $url..."
    mkdir -p "$dest_dir"

    if [[ "$archive_ext" == "zip" ]]; then
        curl -sL "$url" -o "/tmp/toon-lsp-$platform.zip" && \
            unzip -o "/tmp/toon-lsp-$platform.zip" -d "$dest_dir" && \
            rm "/tmp/toon-lsp-$platform.zip" || \
            log_warn "Failed to download $platform"
    else
        curl -sL "$url" | tar -xzf - -C "$dest_dir" || \
            log_warn "Failed to download $platform"
    fi

    if [[ -f "$dest_dir/$binary_name" ]]; then
        log_info "Downloaded $platform: $dest_dir/$binary_name"
    fi
done

log_info "Binary fetch complete!"
