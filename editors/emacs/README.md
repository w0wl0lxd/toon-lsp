# TOON Language Support for Emacs

**Prerequisites**: Emacs 27.1+, `toon-lsp` in `$PATH` (`cargo install toon-lsp`), lsp-mode or eglot.

**Setup**: See `toon-mode.el` (major mode) and `toon-lsp.el` (lsp-mode integration). For eglot (Emacs 29+): `(add-to-list 'eglot-server-programs '(toon-mode . ("toon-lsp")))`.

See [docs/ide-support.md](../docs/ide-support.md) for all features and usage.
