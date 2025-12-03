#!/usr/bin/env bash
# Build all IDE extensions for toon-lsp
# Usage: ./scripts/build-all-extensions.sh [--release]

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

RELEASE_MODE=""
if [[ "${1:-}" == "--release" ]]; then
    RELEASE_MODE="--release"
fi

# Build toon-lsp binary first
log_info "Building toon-lsp binary..."
cd "$ROOT_DIR"
cargo build $RELEASE_MODE

# Bundle binaries to extensions
log_info "Bundling binaries to extensions..."
"$SCRIPT_DIR/bundle-binaries.sh" $RELEASE_MODE

# Build VSCode extension
log_info "Building VSCode extension..."
if [[ -d "$ROOT_DIR/editors/vscode" ]] && [[ -f "$ROOT_DIR/editors/vscode/package.json" ]]; then
    cd "$ROOT_DIR/editors/vscode"
    if command -v npm &> /dev/null; then
        npm install
        npm run compile
        log_info "VSCode extension built successfully"
    else
        log_warn "npm not found, skipping VSCode extension build"
    fi
else
    log_warn "VSCode extension not initialized, skipping"
fi

# Build JetBrains plugin
log_info "Building JetBrains plugin..."
if [[ -d "$ROOT_DIR/editors/jetbrains" ]] && [[ -f "$ROOT_DIR/editors/jetbrains/build.gradle.kts" ]]; then
    cd "$ROOT_DIR/editors/jetbrains"
    if command -v gradle &> /dev/null || [[ -f "./gradlew" ]]; then
        ./gradlew buildPlugin
        log_info "JetBrains plugin built successfully"
    else
        log_warn "Gradle not found, skipping JetBrains plugin build"
    fi
else
    log_warn "JetBrains plugin not initialized, skipping"
fi

# Build tree-sitter grammar
log_info "Building tree-sitter grammar..."
if [[ -d "$ROOT_DIR/editors/shared/tree-sitter-toon" ]]; then
    cd "$ROOT_DIR/editors/shared/tree-sitter-toon"
    if command -v tree-sitter &> /dev/null; then
        tree-sitter generate
        log_info "Tree-sitter grammar built successfully"
    elif command -v npx &> /dev/null; then
        npx tree-sitter-cli generate
        log_info "Tree-sitter grammar built via npx"
    else
        log_warn "tree-sitter CLI not found, skipping grammar build"
    fi
else
    log_warn "tree-sitter grammar not found, skipping"
fi

log_info "Build complete!"
