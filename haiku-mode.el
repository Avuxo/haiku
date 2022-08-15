;;; haiku-mode.el --- major mode for nps  -*- lexical-binding: t; -*-

;;; my dot emacs grows
;;; one day i look inside it
;;; singularity
;;;    - https://www.emacswiki.org/emacs/EmacsHaiku

;;; snippets below taken from:
;;; https://www.omarpolo.com/post/writing-a-major-mode.html

(eval-when-compile
  (require 'rx))

(defconst haiku--font-lock-defaults
  (let ((keywords '("instrs" "bytes")))))

;;;###autoload
(define-derived-mode haiku-mode prog-mode "nps"
  "Major mode for nps files."
  (setq font-lock-defaults haiku--font-lock-defaults)
  (setq-local comment-start "//")
  (setq-local comment-start-skip "#+[\t ]*")
  (setq-local indent-tabs-mode f))
