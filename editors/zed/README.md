# TOON — Zed

TOON language support for Zed.

## Prerequisites

`toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`, or download a
release binary and put it on your `PATH`.

This extension is declarative: it is an `extension.toml` and a grammar, with no Rust
code. Only a Rust extension can download or bundle a language server binary, so Zed
resolves `toon-lsp` from your `PATH`.

## Install

1. Open Zed.
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Linux and Windows) and run
   `zed: extensions`.
3. Search for `TOON` and install it.

## Install from this repository

1. Clone this repository anywhere you like.
2. Run `zed: install dev extension` and select the `editors/zed` directory.

Copying the directory into `~/.config/zed/extensions/` does not register it; Zed loads
dev extensions only through that action.

## Verify

Open a `.toon` file in Zed. Introduce a syntax error and confirm diagnostics appear.

## More Information

See [docs/ide-support.md](../../docs/ide-support.md) for full feature list and usage notes.
