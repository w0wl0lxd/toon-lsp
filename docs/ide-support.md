# IDE Support

toon-lsp implements 18 LSP features: diagnostics, hover, completions, go to definition, find references, rename, and formatting. The other 11 are semantic tokens, document symbols, code actions, code lens, document highlight, document link, folding range, inlay hints, linked editing, selection range, and workspace symbols. Each editor exposes them through its LSP client. You need toon-lsp on PATH unless bundled (VS Code, Zed, JetBrains). Install it with `cargo install toon-lsp` or download a binary from releases.

## Eclipse

Requires Eclipse 2022-03+ and LSP4E. Install LSP4E from Marketplace, then add a language server with program `toon-lsp` for `*.toon`. The plugin registers `org.eclipse.lsp4e.languageServer` with content type `toon`. More in [Eclipse README](../editors/eclipse/README.md).

## Emacs

Requires Emacs 27.1+, `toon-lsp` on PATH, and lsp-mode or eglot. For eglot add the line below. For lsp-mode load `toon-lsp.el` and hook `toon-mode` to `lsp`. More in [Emacs README](../editors/emacs/README.md).

```elisp
(add-to-list 'eglot-server-programs '(toon-mode . ("toon-lsp")))
```

## Helix

Requires Helix 23.05+ and `toon-lsp` on PATH. Copy the blocks below into `~/.config/helix/languages.toml`. More in [Helix README](../editors/helix/README.md).

```toml
[[language]]
name = "toon"
scope = "source.toon"
file-types = ["toon"]
language-servers = ["toon-lsp"]
[language-server.toon-lsp]
command = "toon-lsp"
```

## JetBrains

Supports IntelliJ IDEA, WebStorm, PyCharm, PhpStorm, RubyMine, CLion, GoLand, Rider, and DataGrip. Install "TOON Language" from Plugins Marketplace. The plugin bundles `toon-lsp` and needs no PATH setup. More in [JetBrains README](../editors/jetbrains/README.md).

## Kate

Requires Kate 21.08+ or KDevelop 5.7+ and `toon-lsp` on PATH. Add the JSON below in Settings > Configure Kate > LSP Client > User Server Settings. More in [Kate README](../editors/kate/README.md).

```json
"command": ["toon-lsp"],
"highlightingModeRegex": "^TOON$"
```

## Neovim

Requires Neovim 0.8+, nvim-lspconfig, and `toon-lsp` on PATH. Register the server with the line below. More in [Neovim README](../editors/neovim/README.md).

```lua
require('lspconfig').toon_lsp.setup{}
```

## Notepad++

No LSP support, highlighting only. Import `toon-udl.xml` via Language > User Defined Language > Define your language > Import. More in [Notepad++ README](../editors/notepad++/README.md).

## Sublime Text

Requires Sublime Text 4+, LSP package, and `toon-lsp` on PATH. Add the client config below via Preferences > Package Settings > LSP > Settings. More in [Sublime README](../editors/sublime/README.md).

```json
"command": ["toon-lsp"],
"selector": "source.toon"
```

## Vim

Requires Vim 8.0+ and `toon-lsp` on PATH, with vim-lsp or coc.nvim. More in [Vim README](../editors/vim/README.md).

```vim
'cmd': {server_info->['toon-lsp']},
'allowlist': ['toon'],
```

## VS Code

Install "TOON Language" from the VS Code Marketplace. The extension bundles `toon-lsp` and needs no PATH setup. Override the binary path with `toon-lsp.path` if needed. More in [VS Code README](../editors/vscode/README.md).

```json
"toon-lsp.path": "/custom/path/to/toon-lsp"
```

## Zed

Install "TOON" from Zed Extensions. The extension bundles `toon-lsp` and needs no PATH setup. Its `extension.toml` sets `id = "toon"` and `[language_servers.toon-lsp]` with `languages = ["TOON"]`. More in [Zed README](../editors/zed/README.md).
