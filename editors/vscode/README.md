# TOON Language Support for VS Code

Full language support for [TOON](https://github.com/toon-format/spec) (Token-Oriented Object Notation) files.

## Features

- **Syntax Highlighting**: Full grammar support for all TOON constructs
- **Diagnostics**: Real-time error detection and reporting
- **Hover Information**: View key and value details on hover
- **Go to Definition**: Navigate to duplicate key definitions
- **Find References**: Find all usages of a key
- **Rename Symbol**: Rename keys across the document
- **Document Symbols**: Outline view for quick navigation
- **Document Formatting**: Format TOON files with configurable indentation
- **Semantic Tokens**: Enhanced syntax highlighting

## Installation

### VS Code Marketplace (Recommended)

1. Open VS Code
2. Press `Ctrl+Shift+X` to open Extensions
3. Search for "TOON Language"
4. Click **Install**

### Manual Installation

1. Download the `.vsix` file for your platform from [GitHub Releases](https://github.com/toon-format/toon-lsp/releases)
2. In VS Code, press `Ctrl+Shift+P` → "Install from VSIX..."
3. Select the downloaded file

## Configuration

Open Settings (`Ctrl+,`) and search for "toon":

| Setting | Default | Description |
|---------|---------|-------------|
| `toon-lsp.path` | (bundled) | Path to toon-lsp binary |
| `toon-lsp.trace.server` | off | LSP trace level for debugging |
| `toon-lsp.formatting.tabSize` | 2 | Number of spaces for indentation |
| `toon-lsp.formatting.useTabs` | false | Use tabs instead of spaces |

## Usage

Open any `.toon` file to activate the extension. All LSP features work automatically.

### Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Go to Definition | `F12` |
| Find References | `Shift+F12` |
| Rename Symbol | `F2` |
| Format Document | `Shift+Alt+F` |
| Show Hover | `Ctrl+K Ctrl+I` |

## Requirements

- VS Code 1.75 or later
- No additional dependencies (binary is bundled)

## Troubleshooting

### Binary not found

If you see "Could not find toon-lsp binary", you can:

1. Install via cargo: `cargo install toon-lsp`
2. Download from [releases](https://github.com/toon-format/toon-lsp/releases) and configure the path

### Restart Language Server

Use Command Palette (`Ctrl+Shift+P`) → "TOON: Restart Language Server"

### View Logs

Open Output panel (`Ctrl+Shift+U`) → Select "TOON Language Server"

## License

AGPL-3.0-only - See [LICENSE](https://github.com/toon-format/toon-lsp/blob/main/LICENSE)

## Links

- [TOON Specification](https://github.com/toon-format/spec)
- [toon-lsp Repository](https://github.com/toon-format/toon-lsp)
- [Report Issues](https://github.com/toon-format/toon-lsp/issues)
