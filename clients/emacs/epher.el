;;; epher.el --- Major mode for the epher scripting language -*- lexical-binding: t; -*-

;; Author: upyesp <upyesp@gmail.com>
;; Assisted-by: z.ai:GLM5.3
;; SPDX-License-Identifier: MIT
;; This file is distributed under the MIT License. The full text is in
;; the LICENSE file at the root of this repository.
;; Version: 0.9.11
;; Package-Requires: ((emacs "29.1"))
;; Keywords: languages
;; URL: https://github.com/upyesp/epher

;;; Commentary:

;; epher-mode is a major mode for the scripting language of the epher
;; calculator: you write ordinary math, with units that convert, and
;; every statement's answer appears inline, right next to the line
;; that produced it.
;;
;; `epher-mode' gives you:
;;
;; - baseline font-lock highlighting for .epher files: comments,
;;   strings, numbers, the twenty-one keywords, and number-adjacent units.
;;   The language server's semantic tokens are the exact rule; eglot
;;   does not consume them today.
;; - LSP through eglot (built into Emacs 29): live diagnostics, inline
;;   answers as inlay hints, hover signatures, completion with the
;;   shared snippets, and definition jumps via xref.  lsp-mode is
;;   registered as well.
;; - `C-c C-c' runs the whole script: the per-statement transcript
;;   lands in the `*epher run*' buffer, and every graph the script
;;   produced is saved as an SVG file and opened with the system
;;   viewer.  `g' in the results buffer re-runs, `q' closes it.
;;
;; Required prerequisite: the `epher-lsp' server binary. Without it,
;; epher files only get highlighting; the inline answers, diagnostics,
;; and script runs all come from the server. Download the build for
;; your operating system from the releases page; per-platform
;; instructions are in the README and on the epher website, in the Any
;; LSP client section of the IDE Extensions page:
;; <https://epher.org/ide.html#any-lsp>
;;
;; lsp-mode instead of eglot: lsp-mode is third-party and optional, so
;; epher never loads it.  With lsp-mode loaded, register the client
;; from your init file:
;;
;;   (with-eval-after-load 'lsp-mode
;;     (add-to-list 'lsp-language-id-configuration '(epher-mode . "epher"))
;;     (lsp-register-client
;;      (make-lsp-client
;;       :new-connection (lsp-stdio-connection (lambda () epher-server-program))
;;       :activation-fn (lsp-activate-on "epher")
;;       :server-id 'epher-lsp)))
;;
;; and set `epher-lsp-autostart' to nil so eglot does not start too.

;;; Code:

(require 'prog-mode)
;; eglot ships with Emacs 29.1, this package's floor: the require is
;; unconditional and `eglot-server-programs' is registered at load.
(require 'eglot)

(defgroup epher nil
  "The epher calculator language."
  :prefix "epher-"
  :group 'languages)

(defcustom epher-server-program '("epher-lsp")
  "Command that starts the epher language server.
A list of strings, argv style.  Required prerequisite for everything
beyond highlighting: download the binary for your operating system
from the epher releases page; see the README and the epher website's
IDE Extensions page (Any LSP client section)."
  :type '(repeat string)
  :group 'epher)

(defcustom epher-lsp-autostart t
  "Start eglot automatically in `epher-mode' buffers.
Only when the server program is found in the variable `exec-path'.  Set to nil
if you run the server through lsp-mode or start it yourself."
  :type 'boolean
  :group 'epher)

;; The twenty-one keywords of crates/core KEYWORDS, verbatim.
(defconst epher-keywords
  '("and" "break" "const" "continue" "def" "do" "else" "end" "for"
    "if" "in" "not" "or" "return" "share" "solve" "step" "then" "to"
    "while" "xor"))

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

(defun epher--maybe-start-lsp ()
  "Start eglot in an `epher-mode' buffer when wanted and possible."
  (when (and epher-lsp-autostart
             (executable-find (car epher-server-program)))
    (eglot-ensure)))

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
  (setq-local font-lock-defaults '(epher-font-lock-keywords))
  (epher--maybe-start-lsp))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.epher\\'" . epher-mode))

;; The eglot server registration. The contact is read at activation,
;; so customizing `epher-server-program' is enough. Emacs 29's eglot
;; calls the contact with its `interactive' argument; Emacs 30 asks
;; the arity first. An optional argument satisfies both.
(add-to-list 'eglot-server-programs
             `(epher-mode . ,(lambda (&optional _interactive)
                               epher-server-program)))

;; The run (ADR-0069, decision 3: the text-first editors send the same
;; `epher/run' request the graphical clients use): the per-statement
;; transcript lands in the `*epher run*' buffer with the run's pane
;; presentation (ADR-0078) applied with Emacs's own means: the setup
;; echo dim, failures their own tier, verdict glyphs, matrix grids,
;; folded list tails, the answer bold under the cursor. Every plot
;; the script produced is written as an SVG file and opened with the
;; system viewer, the nvim pattern. `C-c C-c' runs; `g' re-runs,
;; `w' copies a row verbatim, `t' unfolds, `q' closes.

(defface epher-run-setup-face
  '((t :inherit shadow))
  "The dim face of the result pane's setup echo (ADR-0078)."
  :group 'epher)

(defface epher-run-answer-face
  '((t :inherit success :weight bold))
  "The emphasized face of the result pane's answer (ADR-0078 R4)."
  :group 'epher)

(defvar epher-run-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map "q" #'quit-window)
    (define-key map "g" #'epher-run--rerun)
    (define-key map "w" #'epher-run--copy-verbatim)
    (define-key map "t" #'epher-run--toggle-fold)
    map)
  "Keymap for `epher-run-mode'.")

(defvar-local epher-run--script-buffer nil
  "The `epher-mode' buffer the results in this buffer came from.")

(define-derived-mode epher-run-mode special-mode "epher-run"
  "The transcript of an `epher-run'.
`g' re-runs the script, `w' copies a row verbatim, `t' unfolds,
`q' closes the pane.")

(defun epher-run--grid-body (grid hang)
  "One matrix as padded grid rows (ADR-0078 R3): cells padded to
their column's width, columns joined with two spaces, the wire
prefix on the first row, the suffix on the last, continuation
rows indented HANG spaces."
  (let* ((rows (append (plist-get grid :cells) nil))
         (widths (append (plist-get grid :widths) nil))
         (prefix (or (plist-get grid :prefix) ""))
         (suffix (or (plist-get grid :suffix) "")))
    (if (null rows)
        (list (concat prefix suffix))
      (let ((out nil)
            (last (length rows))
            (i 1))
        (dolist (row rows (nreverse out))
          (let ((cells nil)
                (j 0))
            (dolist (cell (append row nil))
              (let ((w (or (nth j widths) (length cell))))
                (push (concat (make-string (max 0 (- w (length cell))) ?\s)
                              cell)
                      cells))
              (setq j (1+ j)))
            (push (concat (if (= i 1) prefix (make-string hang ?\s))
                          (mapconcat #'identity (nreverse cells) "  ")
                          (when (= i last) suffix))
                  out))
          (setq i (1+ i)))))))

(defun epher-run--list-body (list width hang)
  "A long list folded at the threshold (ADR-0078 R3): the shown
elements with a (+n more) note, the hidden tail on its own rows,
wrapped on element boundaries to WIDTH. Returns (ROWS .
TAIL-ROWS)."
  (let* ((elements (append (plist-get list :elements) nil))
         (hidden (or (plist-get list :hidden) 0))
         (shown-count (max 0 (- (length elements) hidden)))
         (shown nil)
         (i 0))
    (while (< i shown-count)
      (push (nth i elements) shown)
      (setq i (1+ i)))
    (let* ((note (if (> hidden 0) (format " (+%d more)" hidden) ""))
           (head (concat (or (plist-get list :prefix) "")
                         "{ " (mapconcat #'identity (nreverse shown) ", ") " }" note
                         (or (plist-get list :suffix) "")))
           (rows (list head))
           (tail-rows 0)
           (line nil))
      (while (< i (length elements))
        (let ((piece (nth i elements)))
          (cond
           ((null line)
            (setq line piece))
           ((<= (+ (length line) 2 (length piece)) (- width hang))
            (setq line (concat line ", " piece)))
           (t
            (push (concat (make-string hang ?\s) line) rows)
            (setq tail-rows (1+ tail-rows))
            (setq line piece))))
        (setq i (1+ i)))
      (when line
        (push (concat (make-string hang ?\s) line) rows)
        (setq tail-rows (1+ tail-rows)))
      (cons (nreverse rows) tail-rows))))

(defun epher-run--wrap-segments (segments width hang plain-text)
  "The terminal pane's wrap (ADR-0078 R2) over the wire's
soft-break SEGMENTS: rows no wider than WIDTH, continuation rows
indented HANG spaces, breaks never inside a number. PLAIN-TEXT is
the pane's display text, whose `= ' voice the first segment may
need to shed."
  (let* ((width (max width 8))
         (indent (make-string (min hang width) ?\s))
         (segs (append segments nil)))
    (when (and segs plain-text
               (not (string-prefix-p "= " plain-text))
               (string-prefix-p "= " (car segs)))
      ;; Prose shed the `= ' voice in the pane text; the segments
      ;; (from the verbatim display) keep it, so the first segment
      ;; sheds it too when the text does not carry it.
      (setcar segs (substring (car segs) 2)))
    (let ((rows nil)
          (current "")
          (has-content nil)
          (continuation nil))
      (dolist (seg segs)
        (dolist (word (split-string seg " " t))
          (let ((rest word))
            (while (> (length rest) 0)
              (when (and (not has-content) continuation (= (length current) 0))
                (setq current indent))
              (let* ((fresh (if (and (not has-content) continuation)
                                (- width (length indent))
                              width))
                     (budget (if has-content
                                 (- width (length current) 1)
                               fresh)))
                (when (< budget 1)
                  (setq budget 1))
                (let* ((len (length rest))
                       (take (cond ((<= len budget) len)
                                   ((<= len fresh) 0)
                                   (t budget))))
                  (cond
                   ((<= take 0)
                    (when has-content
                      (push current rows)
                      (setq current "" has-content nil))
                    (setq continuation t))
                   (t
                    (let ((tail (if (> len take) (substring rest take) "")))
                      (when has-content
                        (setq current (concat current " ")))
                      (setq current (concat current (substring rest 0 take)))
                      (setq has-content t)
                      (setq rest tail)
                      (when (> (length rest) 0)
                        (push current rows)
                        (setq current "" has-content nil)
                        (setq continuation t)))))))))))
      (when has-content
        (push current rows))
      (unless rows
        (push "" rows))
      (nreverse rows))))

(defun epher-run--pane-line-rows (pane width verbatim)
  "One wire presentation line's display rows, propertized. Faces
ride the tier (ADR-0078): the setup echo dims, a failure stays an
error (ADR-0079), a labeled result keeps the success color; the
verdict wears its glyph; a matrix renders as padded grid rows, a
long list folded with its hidden tail, a plain line wrapped at
the wire's soft-break segments, never mid-number. VERBATIM rides
every row for copy and hover. Returns (ROWS . FOLD-COUNT), where
FOLD-COUNT is the hidden tail sitting behind the last rows."
  (let* ((kind (plist-get pane :kind))
         (face (cond ((string= kind "setup") 'epher-run-setup-face)
                     ((string= kind "error") 'error)
                     (t 'success)))
         (verdict (plist-get pane :verdict))
         (glyph (cond ((eq verdict t) "✓ ")
                      ((eq verdict :json-false) "✗ ")
                      (t "")))
         (fold-count 0)
         lines)
    (cond
     ((plist-get pane :grid)
      (setq lines (epher-run--grid-body (plist-get pane :grid) 2)))
     ((plist-get pane :list)
      (let ((pair (epher-run--list-body (plist-get pane :list) width 2)))
        (setq lines (car pair)
              fold-count (cdr pair))))
     (t
      (setq lines (epher-run--wrap-segments
                   (append (plist-get pane :segments) nil)
                   width 2 (plist-get pane :text)))))
    (let ((rows nil)
          (i 0)
          (tail-start (max 0 (- (length lines) fold-count))))
      (dolist (line lines (cons (nreverse rows) fold-count))
        (let* ((text (concat (when (= i 0) (concat "  " glyph)) line))
               (props (list 'face face 'epher-verbatim verbatim)))
          (unless (string= verbatim text)
            (setq props (append props (list 'help-echo verbatim))))
          (when (and (= i tail-start) (> fold-count 0))
            (setq props (append props (list 'epher-fold-count fold-count))))
          (push (apply #'propertize text props) rows))
        (setq i (1+ i))))))

(defun epher-run--statement-rows (statement width)
  "One statement's display rows: the run's presentation applied
when it carries one, the verbatim display alone otherwise (an
older server). Returns (ROWS . ANSWER), ANSWER non-nil when the
statement carries the run's answer (ADR-0078 R4)."
  (let* ((display (plist-get statement :display))
         (presentation (plist-get statement :presentation))
         (err (eq (plist-get statement :error) t))
         (answer (and presentation (eq (plist-get presentation :answer) t)))
         (rows nil))
    (cond
     ;; The silent const (ADR-0080) binds without a display: no rows
     ;; at all, exactly as the graphical panes draw nothing.
     ((null display)
      (setq answer nil))
     ((and presentation (plist-get presentation :lines))
      (dolist (pane (append (plist-get presentation :lines) nil))
        (let ((pair (epher-run--pane-line-rows pane width display)))
          (setq rows (append rows (car pair))))))
     (t
      ;; No presentation: the verbatim display, as before. A failed
      ;; statement's text is its display (the run carries no `error: '
      ;; prefix; the shells add it when they print), so the failure
      ;; prints its message, its own tier (ADR-0079).
      (let ((text (concat "  " display)))
        (setq rows (list (propertize text
                                     'face (if err 'error 'success)
                                     'epher-verbatim display))))
      (setq answer nil)))
    (when (and answer rows)
      (setq rows (mapcar (lambda (row)
                           (propertize row 'epher-answer t))
                         rows)))
    (cons rows answer)))

(defun epher-run--toggle-fold ()
  "Show or hide the folded hidden list tails (ADR-0078 R3)."
  (interactive)
  (if (memq 'epher-run-fold buffer-invisibility-spec)
      (setq-local buffer-invisibility-spec
                  (delq 'epher-run-fold buffer-invisibility-spec))
    (setq-local buffer-invisibility-spec
                (append buffer-invisibility-spec '(epher-run-fold)))))

(defun epher-run--copy-verbatim ()
  "Copy the verbatim output of the row at point (ADR-0078's first
rule: copy never sees the shaped text)."
  (interactive)
  (let ((verbatim (get-text-property (line-beginning-position)
                                     'epher-verbatim)))
    (when (and verbatim (> (length verbatim) 0))
      (kill-new verbatim)
      (message "epher: copied the verbatim output"))))

;; browse-url is built in and autoloaded; the declaration keeps a
;; byte-compile quiet, not the runtime.
(declare-function browse-url-file-url "browse-url")

(defun epher--path-to-uri (path)
  "Return the LSP URI of PATH through the running eglot.
eglot 1.16 renamed the helper and Emacs 30 ships that rename;
Emacs 29.1's built-in eglot predates it and Emacs 30 keeps the old
name only as an obsolete alias, so this calls whichever exists."
  (if (fboundp 'eglot-path-to-uri)
      (funcall (symbol-function 'eglot-path-to-uri) path)
    (funcall (symbol-function 'eglot--path-to-uri) path)))

(defun epher-run--rerun ()
  "Re-run the script that produced these results."
  (interactive)
  (if (buffer-live-p epher-run--script-buffer)
      (with-current-buffer epher-run--script-buffer (epher-run))
    (user-error "The script buffer is gone")))

(defun epher-run ()
  "Run the current script through the language server.
Puts the per-statement transcript in the `*epher run*' buffer
and opens each plot the script produced with the system viewer."
  (interactive)
  (unless (derived-mode-p 'epher-mode)
    (user-error "This command runs in an epher-mode buffer"))
  (let ((server (eglot-current-server)))
    (unless server
      (user-error "The language server is not attached yet; try again in a moment"))
    ;; The method goes as a keyword: Emacs 29.3's jsonrpc nulls a
    ;; string method on the wire (fixed in 30), and the server would
    ;; never answer. The reply decodes as a plist with keyword keys
    ;; and vector arrays in every version.
    (let* ((script-buffer (current-buffer))
           (report (jsonrpc-request server :epher/run
                                    (list :textDocument
                                          (list :uri (epher--path-to-uri
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
      ;; The window shows first: the rows wrap to the real pane width
      ;; (ADR-0078 R2), which only an open window knows.
      (pop-to-buffer run-buffer)
      (let* ((run-window (get-buffer-window run-buffer t))
             (width (or (and run-window (window-body-width run-window)) 78))
             (inhibit-read-only t)
             (fold-spans nil)
             (answer-line nil))
        (with-current-buffer run-buffer
          (erase-buffer)
          (insert (propertize (format "epher run: %s\n"
                                      (buffer-name script-buffer))
                              'face 'bold))
          (let ((line 1))
            (dolist (stmt statements)
              (let* ((source (plist-get stmt :source))
                     (pair (epher-run--statement-rows stmt width)))
                (insert source "\n")
                (setq line (1+ line))
                (dolist (row (car pair))
                  (setq line (1+ line))
                  (insert row "\n")
                  (let ((fold-count (get-text-property 0 'epher-fold-count row)))
                    (when fold-count
                      (push (cons (- line (1- fold-count)) fold-count)
                            fold-spans)))
                  (when (and (null answer-line)
                             (get-text-property 0 'epher-answer row))
                    (setq answer-line line)))))
            (when graph-paths
              (insert (format "\nGraphs (%d, each opened with the system viewer):\n"
                              (length graph-paths)))
              (dolist (path graph-paths)
                (insert "  " path "\n"))))
          (epher-run-mode)
          (setq-local epher-run--script-buffer script-buffer)
          ;; The hidden list tails hide behind an invisible overlay
          ;; (ADR-0078 R3); `t' unfolds, the way every fold opens.
          (dolist (span fold-spans)
            (let* ((beg (progn (goto-char (point-min))
                               (forward-line (1- (car span)))
                               (point)))
                   (end (progn (forward-line (1- (cdr span)))
                               (end-of-line)
                               (point)))
                   (overlay (make-overlay beg end)))
              (overlay-put overlay 'invisible 'epher-run-fold)))
          (setq-local buffer-invisibility-spec
                      (append buffer-invisibility-spec '(epher-run-fold)))
          ;; The run's answer carries the weight (ADR-0078 R4/R5):
          ;; the point lands on it, so the pane opens at what matters.
          (when answer-line
            (goto-char (point-min))
            (forward-line (1- answer-line)))))
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
