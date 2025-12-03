# TOON Language Extension for Zed

Full language support for [TOON](https://github.com/toon-format/spec) files in [Zed](https://zed.dev).

## Features

- Syntax highlighting via tree-sitter
- LSP integration with toon-lsp
- Real-time diagnostics
- Hover information
- Go to definition
- Find references
- Rename symbol
- Document formatting

## Installation

### From Zed Extensions (Recommended)

1. Open Zed
2. Open Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
3. Search for "Extensions"
4. Search for "TOON"
5. Click **Install**

### Manual Installation

1. Clone this repository to Zed's extensions directory:
   ```bash
   git clone https://github.com/toon-format/toon-lsp ~/.config/zed/extensions/toon
   ```

2. Install toon-lsp binary:
   ```bash
   cargo install toon-lsp
   ```

3. Restart Zed

## Configuration

Add to your `settings.json` if needed:

```json
{
  "lsp": {
    "toon-lsp": {
      "binary": {
        "path": "/path/to/toon-lsp"
      }
    }
  }
}
```

## Usage

Open any `.toon` file. The extension activates automatically and provides:

- Syntax highlighting
- Error squiggles for invalid syntax
- Hover for key/value information
- `gd` - Go to definition
- `gr` - Find references
- `<leader>r` - Rename symbol
- Format on save (if enabled)

## Development

To test the extension locally:

```bash
# Clone the extension
cd ~/.config/zed/extensions/
git clone https://github.com/toon-format/toon-lsp toon

# Start Zed in development mode
zed --dev-server-token
```

## License

AGPL-3.0-only

## Links

- [TOON Specification](https://github.com/toon-format/spec)
- [toon-lsp Repository](https://github.com/toon-format/toon-lsp)
- [Zed Extensions Guide](https://zed.dev/docs/extensions)
