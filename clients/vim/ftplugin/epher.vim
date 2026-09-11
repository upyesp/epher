" Buffer settings for epher buffers: the comment form the lexer
" takes, word characters that keep `x1` and `sin_x` whole, and no
" automatic formatting surprises.

setlocal commentstring=#\ %s
setlocal iskeyword+=_
setlocal formatoptions-=t
setlocal suffixesadd=.epher

if !exists("b:undo_ftplugin")
  let b:undo_ftplugin = ""
endif
let b:undo_ftplugin .=
      \ "| setlocal commentstring< iskeyword< formatoptions< suffixesadd<"
