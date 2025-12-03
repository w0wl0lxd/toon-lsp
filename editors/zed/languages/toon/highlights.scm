; TOON syntax highlighting queries for Zed (tree-sitter)

; Comments
(comment) @comment

; Keys (property names)
(key) @property

; Strings
(double_quoted_string) @string
(single_quoted_string) @string
(unquoted_string) @string

; Escape sequences within strings
(escape_sequence) @string.escape

; Numbers
(number) @number

; Booleans
(boolean) @constant.builtin

; Null
(null) @constant.builtin

; Punctuation
":" @punctuation.delimiter
"," @punctuation.delimiter
"-" @punctuation.special
"|" @punctuation.special
"[" @punctuation.bracket
"]" @punctuation.bracket

; Table cells
(table_cell) @string
