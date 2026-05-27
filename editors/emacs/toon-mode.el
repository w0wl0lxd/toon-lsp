;;; toon-mode.el --- Major mode for TOON files -*- lexical-binding: t; -*-

;; Copyright (C) 2024 TOON Format
;; License: AGPL-3.0-only

;; Author: TOON Format <support@toon-format.org>
;; URL: https://github.com/toon-format/toon-lsp
;; Version: 0.1.0
;; Package-Requires: ((emacs "27.1"))
;; Keywords: languages, toon

;;; Commentary:

;; Major mode for editing TOON (Token-Oriented Object Notation) files.
;; Provides basic syntax highlighting and comment support.

;;; Code:

(defgroup toon nil
  "Major mode for TOON files."
  :group 'languages
  :prefix "toon-")

(defcustom toon-indent-offset 2
  "Indentation offset for TOON files."
  :type 'integer
  :group 'toon)

;; Syntax highlighting
(defvar toon-font-lock-keywords
  `(;; Comments
    ("#.*$" . font-lock-comment-face)
    ;; Keys
    ("^\\s-*\\(\\w+\\):" 1 font-lock-variable-name-face)
    ;; Booleans
    ("\\<\\(true\\|false\\)\\>" . font-lock-constant-face)
    ;; Null
    ("\\<null\\>" . font-lock-constant-face)
    ;; Numbers
    ("-?[0-9]+\\(?:\\.[0-9]+\\)?\\(?:[eE][+-]?[0-9]+\\)?" . font-lock-constant-face)
    ;; Strings (double quoted)
    ("\"[^\"]*\"" . font-lock-string-face)
    ;; Strings (single quoted)
    ("'[^']*'" . font-lock-string-face)
    ;; Array markers
    ("^\\s-*-" . font-lock-keyword-face)
    ;; Table markers
    ("|" . font-lock-keyword-face))
  "Font lock keywords for TOON mode.")

;; Syntax table
(defvar toon-mode-syntax-table
  (let ((st (make-syntax-table)))
    ;; Comments start with #
    (modify-syntax-entry ?# "<" st)
    (modify-syntax-entry ?\n ">" st)
    ;; Strings
    (modify-syntax-entry ?\" "\"" st)
    (modify-syntax-entry ?' "\"" st)
    ;; Brackets
    (modify-syntax-entry ?\[ "(]" st)
    (modify-syntax-entry ?\] ")[" st)
    st)
  "Syntax table for TOON mode.")

;;;###autoload
(define-derived-mode toon-mode prog-mode "TOON"
  "Major mode for editing TOON files.

\\{toon-mode-map}"
  :syntax-table toon-mode-syntax-table
  (setq-local comment-start "# ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "#+ *")
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width toon-indent-offset)
  (setq-local font-lock-defaults '(toon-font-lock-keywords)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.toon\\'" . toon-mode))

(provide 'toon-mode)

;;; toon-mode.el ends here
