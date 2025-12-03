;;; toon-lsp.el --- LSP client for TOON files -*- lexical-binding: t; -*-

;; Copyright (C) 2024 TOON Format
;; License: AGPL-3.0-only

;; Author: TOON Format <support@toon-format.org>
;; URL: https://github.com/toon-format/toon-lsp
;; Version: 0.1.0
;; Package-Requires: ((emacs "27.1") (lsp-mode "8.0.0") (toon-mode "0.1.0"))
;; Keywords: languages, toon, lsp

;;; Commentary:

;; LSP client configuration for TOON files using lsp-mode.
;; Requires toon-lsp binary to be installed and in PATH.
;;
;; Installation:
;;   1. Install toon-lsp: cargo install toon-lsp
;;   2. Add to init.el:
;;      (require 'toon-lsp)
;;      (add-hook 'toon-mode-hook #'lsp)
;;
;; For eglot users, see README.md for alternative configuration.

;;; Code:

(require 'lsp-mode)
(require 'toon-mode)

;; Register the LSP client
(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("toon-lsp"))
  :major-modes '(toon-mode)
  :priority -1
  :server-id 'toon-lsp
  :activation-fn (lsp-activate-on "toon")))

;; Add to language ID configuration
(add-to-list 'lsp-language-id-configuration '(toon-mode . "toon"))

;; Optional: Auto-start LSP when opening TOON files
;; (add-hook 'toon-mode-hook #'lsp)

(provide 'toon-lsp)

;;; toon-lsp.el ends here
