# TOON — Neovim

Requires Neovim 0.8+, nvim-lspconfig, and `toon-lsp` on `PATH`. Install the server with
`cargo install toon-lsp`.

nvim-lspconfig does not ship a `toon_lsp` definition, so copy the two files from this
directory into your Neovim runtime path first:

```sh
cp editors/neovim/lua/lspconfig/configs/toon_lsp.lua ~/.config/nvim/lua/lspconfig/configs/
cp editors/neovim/ftdetect/toon.lua ~/.config/nvim/ftdetect/
```

Then add this to your config:

```lua
require('lspconfig').toon_lsp.setup{}
```

Optional `on_attach`: bind `gd`, `K`, `gr`, `<leader>rn` to definition, hover, references, rename.
Verify: run `:LspInfo` or `:checkhealth`, then open a `.toon` file.
See [IDE support](../../docs/ide-support.md) for all features.
