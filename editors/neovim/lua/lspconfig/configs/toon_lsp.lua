-- toon-lsp server definition for nvim-lspconfig
-- Place in: ~/.config/nvim/lua/lspconfig/configs/toon_lsp.lua

local util = require('lspconfig.util')

return {
  default_config = {
    cmd = { 'toon-lsp' },
    filetypes = { 'toon' },
    root_dir = util.root_pattern('.git', '.'),
    single_file_support = true,
    settings = {},
  },
  docs = {
    description = [[
https://github.com/toon-format/toon-lsp

Language server for TOON (Token-Oriented Object Notation) files.

Provides:
- Real-time diagnostics
- Hover information
- Go to definition
- Find references
- Rename symbol
- Document formatting
- Document symbols
- Semantic tokens

Install via cargo:
  cargo install toon-lsp

Or download from releases and add to PATH.
]],
    default_config = {
      root_dir = [[root_pattern(".git", ".")]],
    },
  },
}
