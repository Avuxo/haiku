;;; haiku-mode.el --- major mode for haiku  -*- lexical-binding: t; -*-

;;; my dot emacs grows
;;; one day i look inside it
;;; singularity
;;;    - https://www.emacswiki.org/emacs/EmacsHaiku

;;; snippets below taken from:
;;; https://www.omarpolo.com/post/writing-a-major-mode.html
;;; https://emacs.stackexchange.com/questions/16800/syntax-highlighting-for-comments-in-text-mode

(eval-when-compile
  (require 'rx))

(defconst haiku--font-lock-defaults
  (let ((keywords '("instrs" "bytes"))
        (types '("!call" "!jump")))
    `(((,(rx-to-string `(: (or ,@keywords))) 0 font-lock-keyword-face)
       (,(rx-to-string `(: (or ,@types))) 0 font-lock-type-face)))))

;;;###autoload
(define-derived-mode haiku-mode prog-mode "haiku"
  "Major mode for nps files."
  (setq font-lock-defaults haiku--font-lock-defaults)
  (setq-local comment-start "//")
  (font-lock-add-keywords nil '(("//.+" . font-lock-comment-face)))
  (setq-local comment-start-skip "//+[\t ]*")
  (setq-local tab-width 4)
  (setq-local indent-tabs-mode nil))
