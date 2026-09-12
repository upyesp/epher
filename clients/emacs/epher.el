;;; epher.el --- The epher calculator language in Emacs -*- lexical-binding: t; -*-

;; Version: 0.5
;; Package-Requires: ((emacs "29.1"))
;; Keywords: languages
;; URL: https://github.com/upyesp/epher

;;; Commentary:

;; Ready-made glue for the shared epher language server (ADR-0066
;; ships Emacs as a ready-made config, not a package.el plugin).
;;
;; - `epher-mode': baseline font-lock highlighting (the conservative
;;   view of the grammar -- comments, strings, numbers, the twenty
;;   keywords, and number-adjacent units), `#' comments, and filetype
;;   detection for .epher files.
;; - LSP through eglot (built into Emacs 29): diagnostics from spans,
;;   inline answers as inlay hints, hover signatures, completion.
;;   lsp-mode is registered as well; set `epher-lsp-autostart' to nil
;;   if you would rather start the server yourself.
;;
;; The server binary (`epher-lsp') is a separate download from the
;; releases page; the README has the lines per platform.

;;; Code:

(require 'prog-mode)

(defgroup epher nil
  "The epher calculator language."
  :prefix "epher-"
  :group 'languages)

(defcustom epher-server-program '("epher-lsp")
  "Command that starts the epher language server.
A list of strings, argv style. The binary comes from the epher
releases page; see the README."
  :type '(repeat string)
  :group 'epher)

(defcustom epher-lsp-autostart t
  "Start eglot automatically in `epher-mode' buffers.
Only when the server program is found on `exec-path'. Set to nil
if you run the server through lsp-mode or start it yourself."
  :type 'boolean
  :group 'epher)

;; The twenty keywords of crates/core KEYWORDS, verbatim.
(defconst epher-keywords
  '("and" "break" "const" "continue" "def" "do" "else" "end" "for"
    "if" "in" "not" "or" "return" "solve" "step" "then" "to" "while"
    "xor"))

(defvar epher-mode-syntax-table
  (let ((table (make-syntax-table prog-mode-syntax-table)))
    ;; "#" starts a line comment; newline ends it. The "//" and
    ;; "/* */" forms are font-lock keywords below.
    (modify-syntax-entry ?# "<" table)
    (modify-syntax-entry ?\n ">" table)
    ;; "\"" quotes strings; "\\" escapes.
    (modify-syntax-entry ?\" "\"" table)
    (modify-syntax-entry ?\\ "\\" table)
    ;; "_" is a word character: `x1' and `sin_x' stay whole.
    (modify-syntax-entry ?_ "w" table)
    table)
  "Syntax table for `epher-mode'.")

(defvar epher-font-lock-keywords
  `(
    ;; Comments beyond the syntax table's "#": the other two forms
    ;; the lexer takes. Overriding so keywords inside never win.
    ("//.*$" 0 font-lock-comment-face t)
    ("/\\*.*?\\*/" 0 font-lock-comment-face t)
    ;; The twenty keywords.
    (,(regexp-opt epher-keywords 'symbols) . font-lock-keyword-face)
    ;; Numbers: decimal, scientific, the 0b/0o/0x bases, and the
    ;; imaginary `i' suffix.
    ("\\_<\\(?:0[bB][01]+\\|0[oO][0-7]+\\|0[xX][0-9a-fA-F]+\\|[0-9]+\\(?:\\.[0-9]+\\)?\\(?:[eE][-+]?[0-9]+\\)?i?\\)\\_>"
     . font-lock-constant-face)
    ;; Units: an identifier immediately after a number (whitespace
    ;; between is the parser's adjacency rule). Conservative on
    ;; purpose -- a call right after a number slips through; the
    ;; server's semantic tokens are the exact rule.
    ("\\_<[0-9][ \t]+\\([a-zA-Z_][a-zA-Z_]*\\)\\_>"
     1 font-lock-type-face)
    ;; Calls: a name directly followed by a paren.
    ("\\_<\\([a-zA-Z_][a-zA-Z_0-9]*\\)\\_>[ \t]*("
     1 font-lock-function-call-face))
  "Conservative highlighting for `epher-mode'.
The language server's semantic tokens are the exact rule.")

;;;###autoload
(define-derived-mode epher-mode prog-mode "epher"
  "Major mode for epher calculator scripts.
Baseline highlighting comes from `epher-font-lock-keywords'; the
language server (eglot, Emacs 29+) adds diagnostics, inline
answers as inlay hints, hover, and completion."
  :syntax-table epher-mode-syntax-table
  (setq-local comment-start "# ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "#+\\s-*")
  (setq-local font-lock-defaults '(epher-font-lock-keywords)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.epher\\'" . epher-mode))

(defun epher--maybe-start-lsp ()
  "Start eglot in an `epher-mode' buffer when wanted and possible."
  (when (and epher-lsp-autostart
             (executable-find (car epher-server-program)))
    (eglot-ensure)))

(add-hook 'epher-mode-hook #'epher--maybe-start-lsp)

;; eglot (Emacs 29+ ships it): the contact is read at activation, so
;; customizing `epher-server-program' is enough.
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               `(epher-mode . ,(lambda () epher-server-program))))

;; lsp-mode, for those who run it instead of eglot.
(with-eval-after-load 'lsp-mode
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection
                     (lambda () epher-server-program))
    :major-modes '(epher-mode)
    :server-id 'epher-lsp)))

(provide 'epher)
;;; epher.el ends here
