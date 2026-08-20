# TOON — Sublime Text

## Prerequisites

- Sublime Text 4 or later.
- LSP package. Install it via Package Control.
- `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.

## Setup

### 1. Install the LSP package

Open Command Palette, run `Package Control: Install Package`, and select `LSP`.

### 2. Configure the TOON language client and syntax

Add the client config from `LSP-toon.sublime-settings` to your LSP settings. Open `Preferences > Package Settings > LSP > Settings` and merge in the `clients` block:

```json
{
  // TOON Language Server Settings for Sublime Text
  // Add this to your LSP.sublime-settings via Preferences -> Package Settings -> LSP -> Settings

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

Copy `TOON.sublime-syntax` to your `Packages/User/` folder:

- Linux: `~/.config/sublime-text/Packages/User/TOON.sublime-syntax`
- macOS: `~/Library/Application Support/Sublime Text/Packages/User/TOON.sublime-syntax`
- Windows: `%APPDATA%\Sublime Text\Packages\User\TOON.sublime-syntax`

Restart Sublime Text after you add both files.

## Verify

Open a `.toon` file. Check that syntax highlighting appears. Introduce a syntax error and confirm diagnostics appear.

## More Information

See [IDE Support](../../docs/ide-support.md) for all features and usage.
