# TOON — Emacs

TOON support in Emacs uses `toon-mode.el` for the `toon-mode` major mode and `toon-lsp` for LSP features.

## Prerequisites

* Emacs 27.1 or later.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.
* `lsp-mode` or `eglot` (eglot is built in from Emacs 29).

## Setup

Load `toon-mode.el` to enable syntax highlighting and `toon-mode` for `.toon` files.

### eglot (Emacs 29+)

Add this to your init:

```elisp
(add-to-list 'eglot-server-programs '(toon-mode . ("toon-lsp")))
```

Open a `.toon` file. Eglot starts the server automatically.

### lsp-mode

Load `toon-lsp.el`. It registers the client in 5 lines. Then enable LSP:

```elisp
(require 'toon-lsp)
(add-hook 'toon-mode-hook #'lsp)
```

Or register the client directly:

```elisp
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection '("toon-lsp"))
                  :major-modes '(toon-mode) :server-id 'toon-lsp))
```

## More info

See [docs/ide-support.md](../../docs/ide-support.md) for all features and usage.
