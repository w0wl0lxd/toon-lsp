# TOON — Neovim
Requires Neovim 0.8+, nvim-lspconfig, and `toon-lsp` on `PATH`. Install the server with `cargo install toon-lsp`.
Add this to your config:
```lua
require('lspconfig').toon_lsp.setup{}
```
Optional `on_attach`: bind `gd`, `K`, `gr`, `<leader>rn` to definition, hover, references, rename.
Verify: run `:LspInfo` or `:checkhealth`, then open a `.toon` file.
See [IDE support](../../docs/ide-support.md) for all features.
