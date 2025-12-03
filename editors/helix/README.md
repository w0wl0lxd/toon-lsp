# TOON Language Support for Helix

Full LSP support for [TOON](https://github.com/toon-format/spec) files in [Helix](https://helix-editor.com).

## Prerequisites

- Helix 23.05+
- `toon-lsp` binary in PATH

## Installation

### 1. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases
# https://github.com/toon-format/toon-lsp/releases
```

### 2. Configure Helix

Edit `~/.config/helix/languages.toml`:

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

### 3. Add Syntax Highlighting (Optional)

Copy the tree-sitter queries:

```bash
mkdir -p ~/.config/helix/runtime/queries/toon
cp highlights.scm ~/.config/helix/runtime/queries/toon/
```

### 4. Restart Helix

Open a `.toon` file to verify LSP is working.

## Features

| Feature | Keybinding | Description |
|---------|------------|-------------|
| Hover | `K` | Show key/value information |
| Go to Definition | `gd` | Jump to first occurrence |
| References | `gr` | Find all usages |
| Rename | `<space>r` | Rename symbol |
| Format | `:format` | Format document |
| Diagnostics | `:lsp-diagnostic` | Show errors |

## Configuration Options

```toml
[language-server.toon-lsp]
command = "toon-lsp"
# Uncomment to use explicit path:
# command = "/path/to/toon-lsp"
```

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is in PATH: `which toon-lsp`
2. Check `:log-open` for errors
3. Ensure file has `.toon` extension

### No syntax highlighting

Without tree-sitter grammar, you get LSP-based highlighting only.
Copy the queries to `~/.config/helix/runtime/queries/toon/` for full highlighting.

## Files

- `languages.toml` - Language and LSP configuration
- `runtime/queries/toon/highlights.scm` - Syntax highlighting queries

## License

AGPL-3.0-only
