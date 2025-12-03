# IDE Support

This document provides detailed setup instructions for all supported editors and IDEs.

## Feature Matrix

| Feature | VS Code | Neovim | Zed | Sublime | JetBrains | Helix | Emacs | Vim | Kate | Eclipse |
|---------|---------|--------|-----|---------|-----------|-------|-------|-----|------|---------|
| Diagnostics | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Hover | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Completions | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Go to Definition | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Find References | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Rename Symbol | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Formatting | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Semantic Tokens | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Document Symbols | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |

## Priority 1 Editors (Full Support)

### VS Code

**Install Method:** Marketplace Extension (with bundled binary)

1. Install from VS Marketplace:
   ```bash
   code --install-extension toon-lang.toon-lsp
   ```

2. Or install from VSIX:
   ```bash
   code --install-extension toon-lsp-0.4.0-win32-x64.vsix
   ```

**Features:**
- Bundled toon-lsp binary (no separate installation needed)
- Platform-specific VSIX packages
- TextMate grammar for syntax highlighting
- Full LSP integration

**Configuration:**
```json
{
  "toon-lsp.path": "/custom/path/to/toon-lsp",
  "toon-lsp.trace.server": "verbose"
}
```

See: [editors/vscode/README.md](../editors/vscode/README.md)

---

### Neovim

**Install Method:** nvim-lspconfig

**Prerequisites:**
- Neovim 0.8+
- nvim-lspconfig
- toon-lsp in PATH

**Quick Setup:**
```lua
require('lspconfig').toon_lsp.setup{}
```

**Full Setup with Keybindings:**
```lua
local lspconfig = require('lspconfig')

lspconfig.toon_lsp.setup{
  on_attach = function(client, bufnr)
    local opts = { buffer = bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
  end
}
```

See: [editors/neovim/README.md](../editors/neovim/README.md)

---

### Zed

**Install Method:** Zed Extension

1. Open Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
2. Search for "Extensions: Install Extension"
3. Search for "TOON" and install

**Manual Installation:**
```bash
# Clone to Zed extensions directory
git clone https://github.com/toon-format/toon-lsp-zed ~/.config/zed/extensions/toon
```

**Features:**
- Native tree-sitter highlighting
- Full LSP integration
- Zed-native experience

See: [editors/zed/README.md](../editors/zed/README.md)

---

## Priority 2 Editors

### Sublime Text

**Install Method:** LSP + Syntax Package

**Prerequisites:**
- Package Control
- LSP package
- toon-lsp in PATH

**Setup:**
1. Install LSP package via Package Control
2. Open Preferences > Package Settings > LSP > Settings
3. Add configuration:

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

See: [editors/sublime/README.md](../editors/sublime/README.md)

---

### JetBrains IDEs

**Supported IDEs:** IntelliJ IDEA, WebStorm, PyCharm, GoLand, CLion, Rider, PhpStorm, RubyMine

**Install Method:** JetBrains Marketplace Plugin

1. Open Settings/Preferences
2. Go to Plugins > Marketplace
3. Search for "TOON Language"
4. Install and restart IDE

**Features:**
- LSP4IJ integration
- TextMate grammar highlighting
- Bundled toon-lsp binary
- All JetBrains IDE features

See: [editors/jetbrains/README.md](../editors/jetbrains/README.md)

---

### Helix

**Install Method:** Configuration file

**Prerequisites:**
- Helix 23.10+
- toon-lsp in PATH

**Setup:**
Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "toon"
scope = "source.toon"
file-types = ["toon"]
roots = []
comment-token = "#"
indent = { tab-width = 2, unit = "  " }
language-servers = ["toon-lsp"]

[language-server.toon-lsp]
command = "toon-lsp"
```

See: [editors/helix/README.md](../editors/helix/README.md)

---

## Priority 3 Editors

### Emacs

**Install Method:** Elisp package (lsp-mode or eglot)

**With lsp-mode:**
```elisp
(use-package toon-mode
  :load-path "path/to/editors/emacs"
  :mode "\\.toon\\'"
  :hook (toon-mode . lsp-deferred))

(use-package lsp-mode
  :config
  (add-to-list 'lsp-language-id-configuration '(toon-mode . "toon"))
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection '("toon-lsp"))
    :activation-fn (lsp-activate-on "toon")
    :server-id 'toon-lsp)))
```

**With eglot (Emacs 29+):**
```elisp
(add-to-list 'eglot-server-programs '(toon-mode . ("toon-lsp")))
(add-hook 'toon-mode-hook 'eglot-ensure)
```

See: [editors/emacs/README.md](../editors/emacs/README.md)

---

### Vim

**Install Method:** vim-lsp or coc.nvim

**With vim-lsp:**
```vim
if executable('toon-lsp')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'toon-lsp',
    \ 'cmd': {server_info->['toon-lsp']},
    \ 'allowlist': ['toon'],
    \ })
endif
```

**With coc.nvim:**
Add to `coc-settings.json`:
```json
{
  "languageserver": {
    "toon": {
      "command": "toon-lsp",
      "filetypes": ["toon"],
      "rootPatterns": [".git"]
    }
  }
}
```

See: [editors/vim/README.md](../editors/vim/README.md)

---

## Priority 4 Editors (Basic Support)

### Kate / KDevelop

**Install Method:** LSP Client configuration

**Prerequisites:**
- Kate 21.08+ or KDevelop 5.7+
- toon-lsp in PATH

**Setup:**
1. Go to Settings > Configure Kate > LSP Client
2. Add to User Server Settings:

```json
{
  "servers": {
    "toon": {
      "command": ["toon-lsp"],
      "rootIndicationFileNames": [".git"],
      "highlightingModeRegex": "^TOON$"
    }
  }
}
```

See: [editors/kate/README.md](../editors/kate/README.md)

---

### Eclipse

**Install Method:** LSP4E plugin

**Prerequisites:**
- Eclipse 2022-03+
- LSP4E plugin
- toon-lsp in PATH

**Setup:**
1. Install LSP4E from Eclipse Marketplace
2. Configure language server in Preferences > Language Servers
3. Add toon-lsp for `.toon` files

See: [editors/eclipse/README.md](../editors/eclipse/README.md)

---

### Notepad++

**Note:** Notepad++ does not support LSP. Only syntax highlighting is available.

**Install Method:** User Defined Language

1. Download `toon-udl.xml`
2. Language > User Defined Language > Define your language...
3. Import `toon-udl.xml`
4. Restart Notepad++

**Available Features:**
- Comment highlighting
- String highlighting
- Keyword highlighting (true, false, null)
- Number highlighting

**Not Available:**
- Error diagnostics
- Hover information
- Go to definition
- Find references
- Rename symbol
- Formatting

For full LSP support, consider using a different editor.

See: [editors/notepad++/README.md](../editors/notepad++/README.md)

---

## Installing toon-lsp

All editors (except VS Code and JetBrains with bundled binaries) require toon-lsp in PATH.

### Via Cargo (Recommended)

```bash
cargo install toon-lsp
```

### From GitHub Releases

Download the appropriate binary for your platform:
- `toon-lsp-x86_64-pc-windows-msvc.exe` (Windows x64)
- `toon-lsp-x86_64-apple-darwin` (macOS x64)
- `toon-lsp-aarch64-apple-darwin` (macOS ARM64)
- `toon-lsp-x86_64-unknown-linux-gnu` (Linux x64)

### From Source

```bash
git clone https://github.com/toon-format/toon-lsp
cd toon-lsp
cargo build --release
# Binary at target/release/toon-lsp
```

---

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is installed: `toon-lsp --version`
2. Verify toon-lsp is in PATH: `which toon-lsp` (Unix) or `where toon-lsp` (Windows)
3. Check editor's LSP logs for errors

### No syntax highlighting

- VS Code/Sublime/JetBrains: TextMate grammar should load automatically
- Neovim/Helix/Zed: Requires tree-sitter grammar
- Ensure file has `.toon` extension

### Diagnostics not appearing

1. Check file is saved (some editors only diagnose saved files)
2. Verify LSP server is running (check status bar or logs)
3. Test with a known-invalid file

### Performance issues

For large files (>10MB), consider:
- Increasing editor's LSP timeout
- Checking CPU usage during parsing
- Filing an issue with performance profile

---

## License

AGPL-3.0-only with commercial licensing available. See [LICENSING.md](../LICENSING.md).
