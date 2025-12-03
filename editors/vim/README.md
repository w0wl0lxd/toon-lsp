# TOON Language Support for Vim

Full LSP support for [TOON](https://github.com/toon-format/spec) files in Vim.

## Prerequisites

- Vim 8.0+ or Neovim
- `toon-lsp` binary in PATH
- One of: vim-lsp, coc.nvim, or nvim-lspconfig

## Installation

### 1. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases
# https://github.com/toon-format/toon-lsp/releases
```

### 2. Configure Your LSP Plugin

#### Option A: vim-lsp

1. Install [vim-lsp](https://github.com/prabirshrestha/vim-lsp)

2. Add to your `.vimrc`:

```vim
" Filetype detection
autocmd BufNewFile,BufRead *.toon set filetype=toon

" vim-lsp configuration
if executable('toon-lsp')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'toon-lsp',
    \ 'cmd': {server_info->['toon-lsp']},
    \ 'allowlist': ['toon'],
    \ })
endif

" Optional: Keybindings
function! s:on_lsp_buffer_enabled() abort
  nmap <buffer> gd <plug>(lsp-definition)
  nmap <buffer> gr <plug>(lsp-references)
  nmap <buffer> K <plug>(lsp-hover)
  nmap <buffer> <leader>rn <plug>(lsp-rename)
endfunction

augroup lsp_install
  au!
  autocmd User lsp_buffer_enabled call s:on_lsp_buffer_enabled()
augroup END
```

#### Option B: coc.nvim

1. Install [coc.nvim](https://github.com/neoclide/coc.nvim)

2. Edit `~/.vim/coc-settings.json` (`:CocConfig`):

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

3. Add filetype detection to `.vimrc`:

```vim
autocmd BufNewFile,BufRead *.toon set filetype=toon
```

#### Option C: Neovim nvim-lspconfig

See [editors/neovim/README.md](../neovim/README.md) for Neovim-specific instructions.

## Features

| Feature | vim-lsp | coc.nvim |
|---------|---------|----------|
| Hover | `K` | `K` |
| Go to Definition | `gd` | `gd` |
| References | `gr` | `gr` |
| Rename | `<leader>rn` | `<leader>rn` |
| Format | `:LspDocumentFormat` | `:call CocAction('format')` |

## Files

- `vim-lsp-toon.vim` - Complete vim-lsp configuration
- `coc-settings.json` - coc.nvim language server config
- `ftdetect/toon.vim` - Filetype detection

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is in PATH: `:echo executable('toon-lsp')`
2. For vim-lsp: `:LspStatus`
3. For coc.nvim: `:CocInfo`

### No syntax highlighting

Copy `ftdetect/toon.vim` to `~/.vim/ftdetect/` or add filetype detection to `.vimrc`.

## License

AGPL-3.0-only
