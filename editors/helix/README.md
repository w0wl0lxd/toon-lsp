# TOON — Helix

TOON support in Helix uses `toon-lsp` for diagnostics, formatting, and hover.

## Prerequisites

* Helix 23.05 or later.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.

## Setup

Copy the blocks below into `~/.config/helix/languages.toml`. Create the file if it does not exist. Merge with your existing config if you already have one.

```toml
[[language]]
name = "toon"
scope = "source.toon"
injection-regex = "toon"
file-types = ["toon"]
roots = []
comment-token = "#"
indent = { tab-width = 2, unit = "  " }
language-servers = ["toon-lsp"]
auto-format = true

[language-server.toon-lsp]
command = "toon-lsp"
```

Restart Helix after you save the file.

## Optional: syntax highlighting

For tree-sitter highlighting, copy `runtime/queries/toon/highlights.scm` from this directory into your Helix runtime at `runtime/queries/toon/highlights.scm`.

## Verify

Run `hx --health` and check that `toon-lsp` appears under language servers. Or open any `.toon` file and confirm diagnostics and formatting work.

## More info

See [docs/ide-support.md](../../docs/ide-support.md) for all features and usage.
