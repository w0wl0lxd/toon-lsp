; TOON syntax highlighting queries for tree-sitter

; Comments
(comment) @comment
(block_comment) @comment

; Keys (property names)
(key) @property

; Strings
(double_quoted_string) @string
(single_quoted_string) @string
(block_string) @string
(unquoted_string) @string

; Escape sequences are inline in string tokens

; Numbers
(number) @number

; References (${path} and ${env:VAR})
(reference) @variable

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
