# TOON Language Support for JetBrains IDEs

Full language support for [TOON](https://github.com/toon-format/spec) files in JetBrains IDEs.

## Supported IDEs

- IntelliJ IDEA (Community & Ultimate)
- WebStorm
- PyCharm
- PhpStorm
- RubyMine
- CLion
- GoLand
- Rider
- DataGrip

## Features

- **Syntax Highlighting**: Full token-based highlighting
- **Diagnostics**: Real-time error detection
- **Hover Information**: View key and value details
- **Go to Definition**: Navigate to duplicate key definitions
- **Find References**: Find all usages of a key
- **Rename Symbol**: Rename keys across the document
- **Document Formatting**: Format with configurable indentation
- **Document Symbols**: Structure view for navigation

## Installation

### JetBrains Marketplace (Recommended)

1. Open your JetBrains IDE
2. Go to `Settings` → `Plugins` → `Marketplace`
3. Search for "TOON Language"
4. Click **Install**
5. Restart IDE

### Manual Installation

1. Download `toon-lsp-X.X.X.jar` from [GitHub Releases](https://github.com/toon-format/toon-lsp/releases)
2. `Settings` → `Plugins` → `⚙️` → `Install Plugin from Disk...`
3. Select the downloaded JAR file
4. Restart IDE

## Requirements

- JetBrains IDE 2023.1 or later
- LSP4IJ plugin (installed automatically as dependency)

## Configuration

The plugin uses sensible defaults. Configuration options:

- **toon-lsp path**: By default uses bundled binary. Configure in `Settings` → `Languages & Frameworks` → `TOON`
- **Formatting**: Uses 2-space indentation by default

## Development

### Building from Source

```bash
cd editors/jetbrains
./gradlew buildPlugin
```

The plugin ZIP will be in `build/distributions/`.

### Running in Development IDE

```bash
./gradlew runIde
```

### Running Tests

```bash
./gradlew test
```

## Troubleshooting

### Plugin not activating

1. Ensure LSP4IJ is installed (should install automatically)
2. Verify file has `.toon` extension
3. Check `Help` → `Diagnostic Tools` → `Debug Log Settings`

### LSP server errors

1. Check `View` → `Tool Windows` → `Language Server Protocol` for logs
2. Verify toon-lsp works standalone: `toon-lsp --version`

## License

AGPL-3.0-only

## Links

- [TOON Specification](https://github.com/toon-format/spec)
- [toon-lsp Repository](https://github.com/toon-format/toon-lsp)
- [Report Issues](https://github.com/toon-format/toon-lsp/issues)
