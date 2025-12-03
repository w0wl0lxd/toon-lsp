# TOON Language Support for Emacs

Full language support for [TOON](https://github.com/toon-format/spec) files in Emacs.

## Prerequisites

- Emacs 27.1+
- `toon-lsp` binary in PATH

## Installation

### 1. Install toon-lsp

```bash
# Via cargo (recommended)
cargo install toon-lsp

# Or download from releases
# https://github.com/toon-format/toon-lsp/releases
```

### 2. Install the Emacs packages

#### Option A: use-package with lsp-mode

```elisp
;; In your init.el

;; TOON major mode
(use-package toon-mode
  :load-path "/path/to/toon-lsp/editors/emacs"
  :mode "\\.toon\\'"
  :custom
  (toon-indent-offset 2))

;; LSP client for TOON
(use-package toon-lsp
  :load-path "/path/to/toon-lsp/editors/emacs"
  :after (lsp-mode toon-mode)
  :hook (toon-mode . lsp))
```

#### Option B: Manual with lsp-mode

```elisp
;; Load toon-mode
(add-to-list 'load-path "/path/to/toon-lsp/editors/emacs")
(require 'toon-mode)
(require 'toon-lsp)

;; Auto-start LSP
(add-hook 'toon-mode-hook #'lsp)
```

#### Option C: eglot (Emacs 29+)

```elisp
(require 'eglot)

;; Define TOON mode (or use toon-mode.el)
(define-derived-mode toon-mode prog-mode "TOON"
  "Major mode for editing TOON files."
  (setq-local comment-start "# "))

(add-to-list 'auto-mode-alist '("\\.toon\\'" . toon-mode))

;; Register LSP server
(add-to-list 'eglot-server-programs '(toon-mode . ("toon-lsp")))

;; Auto-start eglot
(add-hook 'toon-mode-hook #'eglot-ensure)
```

## Features

| Feature | Default Binding | Description |
|---------|-----------------|-------------|
| Hover | `K` (evil) / `C-h .` | Show key/value info |
| Go to Definition | `gd` / `M-.` | Jump to first occurrence |
| Find References | `gr` / `M-?` | Find all usages |
| Rename | `SPC r n` / `M-RET r` | Rename symbol |
| Format | `SPC c f` / `C-c C-f` | Format document |

## Doom Emacs

```elisp
;; In packages.el
(package! toon-mode :recipe (:local-repo "/path/to/toon-lsp/editors/emacs"))
(package! toon-lsp :recipe (:local-repo "/path/to/toon-lsp/editors/emacs"))

;; In config.el
(use-package! toon-mode
  :mode "\\.toon\\'")

(use-package! toon-lsp
  :after (lsp-mode toon-mode)
  :hook (toon-mode . lsp))
```

## Spacemacs

```elisp
;; In dotspacemacs-additional-packages
(toon-mode :location "/path/to/toon-lsp/editors/emacs")
(toon-lsp :location "/path/to/toon-lsp/editors/emacs")

;; In dotspacemacs/user-config
(require 'toon-lsp)
(add-hook 'toon-mode-hook #'lsp)
```

## Configuration

```elisp
;; Customize indentation
(setq toon-indent-offset 4)

;; Enable format on save
(add-hook 'toon-mode-hook
          (lambda () (add-hook 'before-save-hook #'lsp-format-buffer nil t)))
```

## Troubleshooting

### LSP not starting

1. Verify toon-lsp is in PATH: `M-x shell-command RET which toon-lsp`
2. Check `*lsp-log*` buffer for errors
3. Run `M-x lsp-describe-session` to see server status

### No syntax highlighting

Ensure `toon-mode.el` is loaded before opening `.toon` files.

## Files

- `toon-mode.el` - Major mode with syntax highlighting
- `toon-lsp.el` - LSP client configuration for lsp-mode

## License

AGPL-3.0-only
