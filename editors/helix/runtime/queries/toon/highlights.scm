; TOON syntax highlighting queries for Helix

; Comments
(comment) @comment

; Keys (property names)
(key) @variable.other.member

; Strings
(double_quoted_string) @string
(single_quoted_string) @string
(unquoted_string) @string

; Escape sequences
(escape_sequence) @constant.character.escape

; Numbers
(number) @constant.numeric

; Booleans
(boolean) @constant.builtin.boolean

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
