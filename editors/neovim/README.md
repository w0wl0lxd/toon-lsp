# TOON Language Server for Neovim

Full LSP support for [TOON](https://github.com/toon-format/spec) files in Neovim.

## Prerequisites

- Neovim 0.8+
- [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)
- `toon-lsp` binary in PATH

## Installation

### 1. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases
# https://github.com/toon-format/toon-lsp/releases
```

### 2. Add LSP Configuration

Add to your `init.lua`:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Register toon-lsp if not already defined
if not configs.toon_lsp then
  configs.toon_lsp = {
    default_config = {
      cmd = { 'toon-lsp' },
      filetypes = { 'toon' },
      root_dir = lspconfig.util.root_pattern('.git', '.'),
      single_file_support = true,
    },
  }
end

-- Setup the server
lspconfig.toon_lsp.setup({})

-- File type detection
vim.filetype.add({ extension = { toon = 'toon' } })
```

### 3. Add Keybindings (Recommended)

```lua
lspconfig.toon_lsp.setup({
  on_attach = function(client, bufnr)
    local opts = { noremap = true, silent = true, buffer = bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>f', vim.lsp.buf.format, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
  end,
})
```

## Features

| Feature | Keybinding | Description |
|---------|------------|-------------|
| Go to Definition | `gd` | Jump to first occurrence of duplicate key |
| Hover | `K` | Show key/value information |
| References | `gr` | Find all usages of a key |
| Rename | `<leader>rn` | Rename key across document |
| Format | `<leader>f` | Format document |

## With nvim-cmp

If using nvim-cmp for completions:

```lua
lspconfig.toon_lsp.setup({
  capabilities = require('cmp_nvim_lsp').default_capabilities(),
})
```

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is installed: `toon-lsp --version`
2. Check `:LspLog` for errors
3. Ensure file has `.toon` extension

### No syntax highlighting

Install tree-sitter TOON grammar or use filetype detection:

```lua
vim.filetype.add({ extension = { toon = 'toon' } })
```

## Files

Copy these files to your Neovim config:

- `lua/lspconfig/configs/toon_lsp.lua` → Server definition
- `ftdetect/toon.lua` → Filetype detection

## License

AGPL-3.0-only
