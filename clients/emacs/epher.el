;;; epher.el --- The epher calculator language in Emacs -*- lexical-binding: t; -*-

;; Version: 0.5.53
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

(defface epher-function-call-face
  '((t :inherit font-lock-function-name-face))
  "Face for names at call positions in `epher-mode'.
Looks identical to `font-lock-function-call-face', which Emacs 29
defines but does not declare as a variable, and
`font-lock-apply-highlight' evals the face field of every keyword
entry (Emacs 30 added the missing declaration)."
  :group 'epher)
;; The eval in `font-lock-apply-highlight' needs a variable binding
;; too, exactly as font-lock.el declares for its own faces.
(defvar epher-function-call-face 'epher-function-call-face)

(defvar epher-font-lock-keywords
  `(
    ;; Comments beyond the syntax table's "#": the other two forms
    ;; the lexer takes. Overriding so keywords inside never win.
    ("//.*$" 0 font-lock-comment-face t)
    ("/\\*.*?\\*/" 0 font-lock-comment-face t)
    ;; The twenty keywords.
    (,(regexp-opt epher-keywords 'symbols) . font-lock-keyword-face)
    ;; Statement commands (graph, graph3d, solar3d, save, table): not
    ;; grammar keywords, but a line opening with one is a shell-level
    ;; statement. The language server's statement-start rule is the
    ;; exact one; line start is the grammar's approximation.
    ("^[ \\t]*\\_<\\(graph3d\\|solar3d\\|graph\\|save\\|table\\)\\_>"
     1 font-lock-keyword-face)
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
     1 epher-function-call-face))
  "Conservative highlighting for `epher-mode'.
The language server's semantic tokens are the exact rule.")

(defvar epher-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-c C-c") #'epher-run)
    map)
  "Keymap for `epher-mode'.")

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
;; customizing `epher-server-program' is enough. Emacs 29's eglot
;; calls the contact with its `interactive' argument; Emacs 30 asks
;; the arity first. An optional argument satisfies both.
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               `(epher-mode . ,(lambda (&optional _interactive)
                                 epher-server-program))))

;; lsp-mode, for those who run it instead of eglot.
(with-eval-after-load 'lsp-mode
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection
                     (lambda () epher-server-program))
    :major-modes '(epher-mode)
    :server-id 'epher-lsp)))

;; The run (ADR-0069, decision 3: the text-first editors send the same
;; `epher/run' request the graphical clients use): the per-statement
;; transcript lands in the `*epher run*' buffer, and each plot the
;; script produced is written as an SVG file and opened with the
;; system viewer, the nvim pattern. `C-c C-c' runs; `g' in the
;; results buffer re-runs.

(defvar epher-run-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map "q" #'quit-window)
    (define-key map "g" #'epher-run--rerun)
    map)
  "Keymap for `epher-run-mode'.")

(defvar-local epher-run--script-buffer nil
  "The `epher-mode' buffer the results in this buffer came from.")

(define-derived-mode epher-run-mode special-mode "epher-run"
  "The transcript of an `epher-run'.
`g' re-runs the script, `q' closes the pane.")

(defun epher-run--rerun ()
  "Re-run the script whose results this buffer shows."
  (interactive)
  (if (buffer-live-p epher-run--script-buffer)
      (with-current-buffer epher-run--script-buffer (epher-run))
    (user-error "the script buffer is gone")))

(defun epher-run ()
  "Run the current script through the language server.
Puts the per-statement transcript in the `*epher run*' buffer
and opens each plot the script produced with the system viewer."
  (interactive)
  (unless (derived-mode-p 'epher-mode)
    (user-error "epher-run runs in an epher-mode buffer"))
  (let ((server (eglot-current-server)))
    (unless server
      (user-error "the language server is not attached yet; try again in a moment"))
    ;; The method goes as a keyword: Emacs 29.3's jsonrpc nulls a
    ;; string method on the wire (fixed in 30), and the server would
    ;; never answer. The reply decodes as a plist with keyword keys
    ;; and vector arrays in every version.
    (let* ((script-buffer (current-buffer))
           (report (jsonrpc-request server :epher/run
                                    (list :textDocument
                                          (list :uri (eglot--path-to-uri
                                                      buffer-file-name)))))
           (statements (append (plist-get report :statements) nil))
           (svgs (append (plist-get report :svgs) nil))
           (graph-paths
            (and svgs
                 (let* ((dir (expand-file-name
                              "epher/runs"
                              (or (getenv "XDG_CACHE_HOME") "~/.cache")))
                        (stamp (format-time-string "%Y%m%d-%H%M%S")))
                   (make-directory dir t)
                   (let ((paths nil) (index 0))
                     (dolist (svg svgs (nreverse paths))
                       (setq index (1+ index))
                       (let ((path (expand-file-name
                                    (format "run-%s-%d.svg" stamp index)
                                    dir)))
                         (with-temp-file path (insert svg))
                         (push path paths)))))))
           (run-buffer (get-buffer-create "*epher run*")))
      (with-current-buffer run-buffer
        (let ((inhibit-read-only t))
          (erase-buffer)
          (insert (propertize (format "epher run: %s\n"
                                      (buffer-name script-buffer))
                              'face 'bold))
          (dolist (stmt statements)
            (let ((source (plist-get stmt :source))
                  (display (plist-get stmt :display))
                  (error-message (plist-get stmt :error)))
              (when (eq error-message :json-false)
                (setq error-message nil))
              (insert source "\n")
              (cond
               (error-message
                (insert (propertize (format "  %s\n" error-message)
                                    'face 'error)))
               (display
                (insert (propertize (format "  %s\n" display)
                                    'face 'success))))))
          (when graph-paths
            (insert (format "\nGraphs (%d, each opened with the system viewer):\n"
                            (length graph-paths)))
            (dolist (path graph-paths)
              (insert "  " path "\n"))))
        (epher-run-mode)
        (setq epher-run--script-buffer script-buffer))
      (pop-to-buffer run-buffer)
      (cond
       ((and graph-paths (display-graphic-p))
        (dolist (path graph-paths)
          (browse-url (browse-url-file-url path))))
       ((and graph-paths
             (executable-find "xdg-open")
             (or (getenv "DISPLAY") (getenv "WAYLAND_DISPLAY")))
        ;; A terminal inside a desktop session: hand the files to the
        ;; desktop directly. `browse-url' in -nw can fall back to the
        ;; in-Emacs text browser, which shows SVG files as raw source.
        (dolist (path graph-paths)
          (call-process "xdg-open" nil 0 nil path)))
       (graph-paths
        (message "no desktop session found; graphs saved under %s"
                 (expand-file-name "epher/runs"
                                   (or (getenv "XDG_CACHE_HOME")
                                       "~/.cache"))))))))

(provide 'epher)
;;; epher.el ends here
