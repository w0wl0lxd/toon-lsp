# TOON Language Support for Eclipse

LSP support for [TOON](https://github.com/toon-format/spec) files in Eclipse IDE.

## Prerequisites

- Eclipse 2022-03 or later
- [LSP4E](https://github.com/eclipse/lsp4e) plugin installed
- `toon-lsp` binary in PATH

## Installation

### 1. Install LSP4E

1. Open Eclipse
2. Go to **Help** → **Eclipse Marketplace**
3. Search for "LSP4E"
4. Install "Eclipse LSP4E"
5. Restart Eclipse

### 2. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases and add to PATH
# https://github.com/toon-format/toon-lsp/releases
```

### 3. Configure Language Server

1. Go to **Window** → **Preferences** → **Language Servers**
2. Click **Add...**
3. Configure:
   - **Program**: `toon-lsp` (or full path)
   - **Arguments**: (leave empty)
   - **Content Types**: Click **Add...** and create:
     - Content type: `TOON`
     - File extension: `.toon`
   - **Language ID**: `toon`
4. Click **Apply and Close**

### Alternative: Manual Plugin Installation

1. Download the plugin JAR from releases
2. Place in Eclipse's `dropins/` folder
3. Restart Eclipse

## Features

- Error diagnostics
- Hover information
- Go to definition
- Find references
- Rename symbol
- Document formatting

## Keybindings

| Action | Shortcut |
|--------|----------|
| Hover | Mouse hover or `F2` |
| Go to Definition | `F3` or `Ctrl+Click` |
| Find References | `Ctrl+Shift+G` |
| Rename | `Alt+Shift+R` |
| Format | `Ctrl+Shift+F` |

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is in PATH
2. Check **Window** → **Show View** → **Error Log**
3. Verify content type mapping in Preferences

### No syntax highlighting

Eclipse uses LSP semantic tokens for highlighting.
If not working, check LSP server logs in Error Log view.

## Development

To build the Eclipse plugin from source:

```bash
# Requires Eclipse PDE and Maven/Tycho
mvn clean install
```

## Files

- `plugin.xml` - Plugin descriptor template (reference only)
- Full plugin requires Eclipse PDE build

## License

AGPL-3.0-only
