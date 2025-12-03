# TOON Language Support for Sublime Text

Full language support for [TOON](https://github.com/toon-format/spec) files in Sublime Text.

## Prerequisites

- Sublime Text 4+
- [Package Control](https://packagecontrol.io/)
- [LSP](https://packagecontrol.io/packages/LSP) package
- `toon-lsp` binary in PATH

## Installation

### 1. Install LSP Package

1. Open Command Palette (`Ctrl+Shift+P`)
2. Run "Package Control: Install Package"
3. Search for "LSP" and install

### 2. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases and add to PATH
# https://github.com/toon-format/toon-lsp/releases
```

### 3. Configure LSP

1. Open Command Palette → "Preferences: LSP Settings"
2. Add to the "clients" object:

```json
{
  "clients": {
    "toon-lsp": {
      "enabled": true,
      "command": ["toon-lsp"],
      "selector": "source.toon"
    }
  }
}
```

### 4. Install Syntax Highlighting

Copy `TOON.sublime-syntax` to your Sublime Text packages:

**Linux/macOS:**
```bash
cp TOON.sublime-syntax ~/.config/sublime-text/Packages/User/
```

**Windows:**
```powershell
copy TOON.sublime-syntax $env:APPDATA\Sublime Text\Packages\User\
```

## Features

| Feature | Shortcut |
|---------|----------|
| Hover | `Ctrl+Alt+H` or mouse hover |
| Go to Definition | `F12` |
| Find References | `Shift+F12` |
| Rename Symbol | `F2` |
| Format Document | `Ctrl+Shift+P` → "LSP: Format Document" |

## Configuration Options

```json
{
  "clients": {
    "toon-lsp": {
      "enabled": true,
      "command": ["toon-lsp"],
      "selector": "source.toon",
      "initializationOptions": {},
      "settings": {
        "formatting": {
          "tabSize": 2,
          "useTabs": false
        }
      }
    }
  }
}
```

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is in PATH: `which toon-lsp`
2. Check LSP logs: `Ctrl+`\` → type "LSP: Toggle Log Panel"
3. Ensure file has `.toon` extension

### No syntax highlighting

1. Verify TOON.sublime-syntax is in Packages/User
2. Restart Sublime Text
3. Check View → Syntax is set to "TOON"

## Files

- `LSP-toon.sublime-settings` - LSP client configuration
- `TOON.sublime-syntax` - Syntax highlighting rules

## License

AGPL-3.0-only
