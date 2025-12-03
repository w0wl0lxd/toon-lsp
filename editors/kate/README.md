# TOON Language Support for Kate/KDevelop

LSP support for [TOON](https://github.com/toon-format/spec) files in Kate and KDevelop.

## Prerequisites

- Kate 21.08+ or KDevelop 5.7+
- KDE Frameworks 5.80+
- `toon-lsp` binary in PATH

## Installation

### 1. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases
# https://github.com/toon-format/toon-lsp/releases
```

### 2. Configure LSP Client

1. Open Kate/KDevelop
2. Go to **Settings** → **Configure Kate** → **LSP Client**
3. Click **User Server Settings** tab
4. Add the following JSON:

```json
{
  "servers": {
    "toon": {
      "command": ["toon-lsp"],
      "url": "https://github.com/toon-format/toon-lsp",
      "rootIndicationFileNames": [".git"],
      "highlightingModeRegex": "^TOON$"
    }
  }
}
```

5. Click **OK** to save

### 3. Add Syntax Highlighting (Optional)

1. Copy `toon.xml` to Kate's syntax highlighting directory:

**Linux:**
```bash
cp toon.xml ~/.local/share/org.kde.syntax-highlighting/syntax/
```

**Note:** Kate can also use the TextMate grammar from `editors/shared/toon.tmLanguage.json`.

## Features

- Diagnostics (errors and warnings)
- Hover information
- Go to definition
- Find references
- Rename symbol
- Document formatting

## Keybindings

| Action | Shortcut |
|--------|----------|
| Hover | Mouse hover |
| Go to Definition | `Ctrl+Click` or `F12` |
| Find References | `Ctrl+Shift+U` |
| Rename | `Ctrl+Shift+R` |

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is in PATH
2. Check **View** → **Tool Views** → **LSP Client** for errors
3. Ensure file has `.toon` extension

### No syntax highlighting

Without the syntax definition, you'll only get LSP-based highlighting.
Kate may prompt to download highlighting rules for unrecognized file types.

## License

AGPL-3.0-only
