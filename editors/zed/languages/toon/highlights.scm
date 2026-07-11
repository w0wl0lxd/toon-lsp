; TOON syntax highlighting queries for Zed (tree-sitter)

; Comments
(comment) @comment

; Keys (property names)
(key) @property

; Strings
(double_quoted_string) @string
(single_quoted_string) @string
(unquoted_string) @string

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
"[" @punctuation.bracket
"]" @punctuation.bracket
