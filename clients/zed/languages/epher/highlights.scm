; epher highlights for Zed: baseline tree-sitter coloring, with the
; language server's semantic tokens refining it live (ADR-0066).
; The unit suffix is its own node (src/scanner.c), so it colors by
; meaning exactly where the real parser sees one.

(comment) @comment

(string) @string
(escape_sequence) @string.escape

(number) @number
(quantity
  number: (number) @number
  unit: (unit) @type)

[
  "and" "break" "const" "continue" "def" "do" "else" "end" "for" "if"
  "in" "not" "or" "return" "solve" "step" "then" "to" "while" "xor"
] @keyword

(call_expression
  function: (identifier) @function.call)

(assignment
  name: (identifier) @variable)

(constant_definition
  name: (identifier) @constant)

(function_definition
  name: (identifier) @function)

(function_definition
  (parameters
    (identifier) @variable.parameter))

(destructuring
  name: (identifier) @variable)

(identifier) @variable

[
  "+" "-" "*" "/" "^" "!" "%" "&" "|" "~" "->" "==" "!=" ">=" "<=" ">" "<" "="
] @operator

[
  "(" ")" "[" "]" "{" "}" "[[" "]]"
] @punctuation.bracket

"," @punctuation.delimiter
