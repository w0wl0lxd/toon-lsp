# TOON — Vim

TOON support in Vim uses `toon-lsp` for diagnostics, formatting, hover, and more.

## Prerequisites

* Vim 8.0+ or Neovim.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.
* One LSP client: [vim-lsp](https://github.com/prabirshrestha/vim-lsp) or [coc.nvim](https://github.com/neoclide/coc.nvim).

## Setup

### Filetype detection

Add filetype detection for `*.toon` files. Copy `ftdetect/toon.vim` to
`~/.vim/ftdetect/toon.vim` for Vim, or `~/.config/nvim/ftdetect/toon.vim` for Neovim.
Or add this line to your `vimrc` (or `init.vim`):

```vim
autocmd BufNewFile,BufRead *.toon set filetype=toon
```

### Option A: vim-lsp

Add this to your `vimrc` after you install [vim-lsp](https://github.com/prabirshrestha/vim-lsp). See `vim-lsp-toon.vim` for a full example with keybindings and formatting.

```vim
if executable('toon-lsp')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'toon-lsp',
    \ 'cmd': {server_info->['toon-lsp']},
    \ 'allowlist': ['toon'],
    \ })
endif
```

Restart Vim after you save the file.

### Option B: coc.nvim

Add this to your `coc-settings.json` after you install [coc.nvim](https://github.com/neoclide/coc.nvim). You can copy `coc-settings.json` from this directory.

```json
{
  "languageserver": {
    "toon": {
      "command": "toon-lsp",
      "filetypes": ["toon"]
    }
  }
}
```

ALE is not supported. `g:ale_linters` only selects linters that are already registered
with `ale#linter#Define()`, and this repository defines no ALE linter for `toon`, so
`let g:ale_linters = {'toon': ['toon-lsp']}` selects nothing. Use vim-lsp or coc.nvim.

## Verify

Open a `.toon` file. Check that diagnostics appear. Run `:LspStatus` for vim-lsp or `:CocInfo` for coc.nvim to confirm the server is running.

## More info

See [docs/ide-support.md](../../docs/ide-support.md) for all features and usage.
