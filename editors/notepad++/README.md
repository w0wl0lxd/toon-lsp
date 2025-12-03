# TOON Syntax Highlighting for Notepad++

Basic syntax highlighting for [TOON](https://github.com/toon-format/spec) files in Notepad++.

## Important Limitation

**Notepad++ does not support LSP (Language Server Protocol).**

This means you will NOT get:
- Error diagnostics
- Hover information
- Go to definition
- Find references
- Rename symbol
- Document formatting

Only basic syntax highlighting is available.

For full LSP support, consider using:
- [VS Code](../vscode/README.md)
- [Sublime Text](../sublime/README.md)
- [Vim](../vim/README.md)

## Installation

### Method 1: Import UDL (Recommended)

1. Download `toon-udl.xml`
2. Open Notepad++
3. Go to **Language** → **User Defined Language** → **Define your language...**
4. Click **Import...**
5. Select `toon-udl.xml`
6. Click **OK**
7. Restart Notepad++

### Method 2: Manual Copy

1. Close Notepad++
2. Copy `toon-udl.xml` to:
   - **Windows**: `%APPDATA%\Notepad++\userDefineLangs\`
3. Restart Notepad++

## Usage

After installation, `.toon` files will automatically use TOON syntax highlighting.

To manually set the language:
1. Open a TOON file
2. Go to **Language** → **TOON**

## Features

- Comment highlighting (`#`)
- Keyword highlighting (`true`, `false`, `null`)
- String highlighting (single and double quoted)
- Number highlighting
- Operator highlighting (`:`, `-`, `|`, `[`, `]`)

## Customization

To customize colors:
1. Go to **Language** → **User Defined Language** → **Define your language...**
2. Select **TOON** from the dropdown
3. Modify colors in the **Styler** tab
4. Click **Save As...** to save changes

## Alternative: External Tools

For LSP-like features in Notepad++, you can use external tools:
- [NppExec](https://github.com/d0vgan/nppexec) - Run toon-lsp commands
- Configure custom commands for validation

## Files

- `toon-udl.xml` - User Defined Language syntax definition

## License

AGPL-3.0-only
