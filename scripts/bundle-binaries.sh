#!/usr/bin/env bash
# Bundle toon-lsp binaries to each extension that needs them
# Usage: ./scripts/bundle-binaries.sh [--release]

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

BUILD_DIR="debug"
if [[ "${1:-}" == "--release" ]]; then
    BUILD_DIR="release"
fi

# Detect host platform
detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            case "$arch" in
                arm64) echo "darwin-arm64" ;;
                x86_64) echo "darwin-x64" ;;
                *) echo "unknown" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) echo "linux-x64" ;;
                aarch64) echo "linux-arm64" ;;
                *) echo "unknown" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "win32-x64"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

PLATFORM="$(detect_platform)"
log_info "Detected platform: $PLATFORM"

# Binary name varies by platform
BINARY_NAME="toon-lsp"
if [[ "$PLATFORM" == "win32-x64" ]]; then
    BINARY_NAME="toon-lsp.exe"
fi

SOURCE_BINARY="$ROOT_DIR/target/$BUILD_DIR/$BINARY_NAME"

if [[ ! -f "$SOURCE_BINARY" ]]; then
    log_error "Binary not found: $SOURCE_BINARY"
    log_error "Run 'cargo build' first"
    exit 1
fi

log_info "Source binary: $SOURCE_BINARY"

# Bundle to VSCode extension
bundle_vscode() {
    local dest_dir="$ROOT_DIR/editors/vscode/binaries/$PLATFORM"
    if [[ -d "$ROOT_DIR/editors/vscode" ]]; then
        mkdir -p "$dest_dir"
        cp "$SOURCE_BINARY" "$dest_dir/$BINARY_NAME"
        log_info "Bundled to VSCode: $dest_dir/$BINARY_NAME"
    fi
}

# Bundle to JetBrains plugin
bundle_jetbrains() {
    local dest_dir="$ROOT_DIR/editors/jetbrains/binaries"
    if [[ -d "$ROOT_DIR/editors/jetbrains" ]]; then
        mkdir -p "$dest_dir"
        cp "$SOURCE_BINARY" "$dest_dir/$BINARY_NAME"
        log_info "Bundled to JetBrains: $dest_dir/$BINARY_NAME"
    fi
}

bundle_vscode
bundle_jetbrains

log_info "Binary bundling complete!"
