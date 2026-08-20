# TOON — Kate

TOON support in Kate uses `toon-lsp` for diagnostics, formatting, hover, and more.

## Prerequisites

* Kate 21.08 or later or KDevelop 5.7 or later.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.

## Setup

1. Open Settings → Configure Kate → LSP Client → User Server Settings.
2. Add the JSON below. This file lives at `toon.json` in this directory.

```json
{
  "servers": {
    "toon": {
      "command": ["toon-lsp"],
      "url": "https://github.com/toon-format/toon-lsp",
      "rootIndicationFileNames": [".git", ".toon"],
      "highlightingModeRegex": "^TOON$"
    }
  }
}
```

3. Save the settings and restart Kate.

## Optional: syntax highlighting

Copy `toon.xml` to your Kate syntax highlighting directory if you want syntax highlighting. Restart Kate after you copy the file.

## Verify

Open any `.toon` file. Confirm that diagnostics and hover work.

## More info

See [docs/ide-support.md](../../docs/ide-support.md) for all features and usage.
