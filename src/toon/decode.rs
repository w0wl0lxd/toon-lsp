//! TOON-to-JSON decoder.
//!
//! Drives the existing LSP [`crate::parser::Scanner`] token stream.
//!
//! References (`${path}`) have no JSON counterpart, so they are decoded back to
//! their literal `${path}` string form.

pub use scanner_driven::decode;

mod scanner_driven {
    use serde_json::{Map, Value};

    use crate::parser::{Scanner, Token, TokenKind};
    use crate::toon::error::{DecodeError, DecodeResult};

    /// Decodes TOON `input` into a [`serde_json::Value`].
    ///
    /// # Errors
    /// Returns [`DecodeError`] on malformed TOON (unexpected tokens, scanner
    /// errors, or unparseable numbers).
    pub fn decode(input: &str) -> DecodeResult<Value> {
        let tokens = Scanner::new(input).scan_all();
        let mut parser = Parser {
            tokens,
            pos: 0,
            delimiter: ',',
            input,
        };
        let value = parser.parse_document()?;
        Ok(value)
    }

    struct Parser<'a> {
        tokens: Vec<Token>,
        pos: usize,
        delimiter: char,
        input: &'a str,
    }

    impl Parser<'_> {
        fn kind(&self) -> &TokenKind {
            self.tokens
                .get(self.pos)
                .map_or(&TokenKind::Eof, |t| &t.kind)
        }

        fn kind_at(&self, offset: usize) -> &TokenKind {
            self.tokens
                .get(self.pos + offset)
                .map_or(&TokenKind::Eof, |t| &t.kind)
        }

        fn bump(&mut self) -> TokenKind {
            let kind = self.kind().clone();
            if self.pos < self.tokens.len() {
                self.pos += 1;
            }
            kind
        }

        fn token(&self) -> Option<&Token> {
            self.tokens.get(self.pos)
        }

        fn expect(&mut self, want: &TokenKind) -> DecodeResult<()> {
            if self.kind() == want {
                self.pos += 1;
                Ok(())
            } else {
                Err(DecodeError::new(format!(
                    "expected {want}, found {}",
                    self.kind()
                )))
            }
        }

        fn skip_newlines(&mut self) {
            while matches!(self.kind(), TokenKind::Newline) {
                self.pos += 1;
            }
        }

        /// Bumps tokens to end of line, surfacing any scanner `Error` token as a
        /// decode error (e.g. tabs in indentation, invalid escapes).
        fn bump_to_line_end(&mut self) -> DecodeResult<()> {
            while !matches!(self.kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
                if let TokenKind::Error(msg) = self.kind() {
                    return Err(DecodeError::new(format!("scanner error: {msg}")));
                }
                self.bump();
            }
            Ok(())
        }

        /// Returns the delimiter declared in the current array header, or `,`
        /// (comma) when none is declared. The scanner treats both `\t` and `|`
        /// as trivia, so the delimiter is detected from the raw header text: an
        /// explicit `\t` / `|` appearing inside the `[N]` count bracket or right
        /// before the colon (after a `{cols}` header) selects that delimiter.
        /// Nested arrays without an explicit delimiter use comma, per TOON.
        fn detect_delimiter(&self) -> char {
            let line = self.current_line_raw();
            let header = match line.find(':') {
                Some(i) => &line[..i],
                None => &line[..],
            };
            if header.contains('\t') {
                '\t'
            } else if header.contains('|') {
                '|'
            } else {
                ','
            }
        }

        fn parse_document(&mut self) -> DecodeResult<Value> {
            self.skip_newlines();
            if matches!(self.kind(), TokenKind::Eof) {
                return Ok(Value::Object(Map::new()));
            }
            let value = self.parse_block(true)?;
            self.skip_newlines();
            if !matches!(self.kind(), TokenKind::Eof) {
                return Err(DecodeError::new(format!(
                    "trailing tokens after document: {}",
                    self.kind()
                )));
            }
            Ok(value)
        }

        /// Parses a block of sibling lines at the current indentation: an
        /// expanded array if it starts with `-`, otherwise an object (or a bare
        /// scalar line at document root).
        ///
        /// `allow_scalar` is true only at document root; inside an object or
        /// list-item block a bare scalar line is a structural error.
        fn parse_block(&mut self, allow_scalar: bool) -> DecodeResult<Value> {
            self.skip_newlines();
            match self.kind() {
                TokenKind::Dash => self.parse_expanded_array(),
                TokenKind::LeftBracket
                    if matches!(self.kind_at(1), TokenKind::RightBracket) =>
                {
                    // Literal empty array `[]` at document root.
                    self.bump();
                    self.bump();
                    self.consume_line_end()?;
                    Ok(Value::Array(Vec::new()))
                }
                TokenKind::LeftBracket
                    if matches!(self.kind_at(1), TokenKind::Number(_)) =>
                {
                    // Root-level array count form `[N]: ...` / `[N]{cols}: ...`.
                    let old_delim = self.delimiter;
                    self.delimiter = self.detect_delimiter();
                    let value = self.parse_array_value()?;
                    self.delimiter = old_delim;
                    Ok(value)
                }
                TokenKind::Identifier(_) | TokenKind::String(_)
                    if self.starts_object_entry() =>
                {
                    self.parse_object()
                }
                _ => {
                    if !allow_scalar {
                        return Err(DecodeError::new(
                            "expected a key or list item, found a bare value",
                        ));
                    }
                    let value = self.parse_line_value()?;
                    self.consume_line_end()?;
                    Ok(value)
                }
            }
        }

        /// True if the current line is `key:` or `key[...`, i.e. an object entry
        /// rather than a bare scalar.
        fn starts_object_entry(&self) -> bool {
            matches!(self.kind_at(1), TokenKind::Colon | TokenKind::LeftBracket)
        }

        fn parse_object(&mut self) -> DecodeResult<Value> {
            let mut map = Map::new();
            loop {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Dedent | TokenKind::Eof) {
                    break;
                }
                if !matches!(self.kind(), TokenKind::Identifier(_) | TokenKind::String(_)) {
                    break;
                }
                self.parse_object_entry(&mut map)?;
            }
            Ok(Value::Object(map))
        }

        /// The raw text of the current line, from the current token's start to
        /// the next newline (or end of input).
        fn current_line_raw(&self) -> String {
            let start = self.token().map_or(0, |t| t.span.start.offset as usize);
            let line_end = self.input[start..]
                .find('\n')
                .map_or(self.input.len(), |i| start + i);
            self.input[start..line_end].to_string()
        }

        /// Parses one `key: value` / `key[N]: ...` / `key[N]{cols}:` entry into
        /// `map`, including any indented continuation block.
        ///
        /// In non-strict mode a key may carry extra bracket segments or trailing
        /// text (e.g. `foo[1][bar]:`, `foo[2]extra:`); the entire text before
        /// the colon becomes a literal key.
        fn parse_object_entry(&mut self, map: &mut Map<String, Value>) -> DecodeResult<()> {
            let key_start = self.token().map_or(0, |t| t.span.start.offset as usize);
            let line_end = self.input[key_start..]
                .find('\n')
                .map_or(self.input.len(), |i| key_start + i);
            let line = &self.input[key_start..line_end];
            let colon_pos = find_key_colon(line)
                .ok_or_else(|| DecodeError::new("expected colon after key"))?;
            let key_text = line[..colon_pos].trim();

            // A valid array key is `name[N]` or `name[N]{cols}`. Anything else
            // (including extra brackets or trailing text) is a literal key.
            if is_array_key(key_text) {
                let key = self.parse_key()?;
                // The scanner skips `\t`/`|` as trivia, so the explicit
                // delimiter must be detected from the raw key text.
                let old_delim = self.delimiter;
                self.delimiter = self.detect_delimiter();
                let value = self.parse_array_value()?;
                self.delimiter = old_delim;
                map.insert(key, value);
                return Ok(());
            }

            // Literal (or plain) key: a quoted key is unescaped, otherwise the
            // raw text before the colon is used verbatim.
            let literal_key = if key_text.starts_with('"')
                && key_text.ends_with('"')
                && key_text.len() >= 2
            {
                unescape(&key_text[1..key_text.len() - 1])?
            } else {
                key_text.to_string()
            };

            while !matches!(self.kind(), TokenKind::Colon) {
                if matches!(self.kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
                    return Err(DecodeError::new("expected colon after key"));
                }
                self.bump();
            }
            self.expect(&TokenKind::Colon)?;

            if matches!(self.kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Dedent) {
                    map.insert(literal_key, Value::Array(Vec::new()));
                } else if matches!(self.kind(), TokenKind::Indent) {
                    self.bump();
                    let child = self.parse_block(false)?;
                    self.expect(&TokenKind::Dedent)?;
                    map.insert(literal_key, child);
                } else {
                    map.insert(literal_key, Value::Object(Map::new()));
                }
            } else if matches!(self.kind(), TokenKind::LeftBracket)
                && matches!(self.kind_at(1), TokenKind::RightBracket)
            {
                // Literal empty array `key: []`.
                self.bump();
                self.bump();
                self.consume_line_end()?;
                map.insert(literal_key, Value::Array(Vec::new()));
            } else {
                let value = self.parse_line_value()?;
                map.insert(literal_key, value);
            }
            Ok(())
        }

        /// Parses `[N]:` (inline scalar array), `[N]{cols}:` (tabular), or
        /// `[N]:` followed by an indented expanded block.
        fn parse_array_value(&mut self) -> DecodeResult<Value> {
            self.expect(&TokenKind::LeftBracket)?;
            let count = self.parse_count()?;
            let old_delim = self.delimiter;
            self.expect(&TokenKind::RightBracket)?;

            let value = if matches!(self.kind(), TokenKind::LeftBrace) {
                self.parse_tabular(count)?
            } else {
                self.expect(&TokenKind::Colon)?;
                match self.kind() {
                        TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent => {
                            self.skip_newlines();
                            if count == 0 {
                                // An empty array (`key[0]:`) has no items; any
                                // following Indent is the object-item's deeper
                                // field continuation, so leave it unconsumed.
                                Value::Array(Vec::new())
                            } else if matches!(self.kind(), TokenKind::Indent) {
                                self.bump();
                                let items = self.parse_expanded_items(Some(count))?;
                                self.expect(&TokenKind::Dedent)?;
                                Value::Array(items)
                            } else {
                                Value::Array(Vec::new())
                            }
                        }
                        _ => {
                            let items = self.parse_inline_items(count)?;
                            Value::Array(items)
                        }
                    }
                }
            ;
            self.delimiter = old_delim;
            Ok(value)
        }

        fn parse_tabular(&mut self, count: usize) -> DecodeResult<Value> {
            // Current token is `{`; read the header from raw input so quoted
            // fields containing the active delimiter survive.
            let line = self.current_line_raw();
            let open = line
                .find('{')
                .ok_or_else(|| DecodeError::new("expected { in tabular header"))?;
            let close_rel = line[open..]
                .find('}')
                .ok_or_else(|| DecodeError::new("expected } in tabular header"))?;
            let close = open + close_rel;
            let header_inner = &line[open + 1..close];
            let cols: Vec<String> = split_respecting_quotes(header_inner, self.delimiter)
                .into_iter()
                .map(|p| match parse_scalar_raw(p.trim()) {
                    Ok(Value::String(s)) => Ok(s),
                    Ok(other) => Ok(other.to_string()),
                    Err(e) => Err(e),
                })
                .collect::<DecodeResult<Vec<_>>>()?;

            // Consume the header tokens up to and including `}`.
            while !matches!(self.kind(), TokenKind::RightBrace) {
                self.bump();
            }
            self.bump();
            self.expect(&TokenKind::Colon)?;
            self.skip_newlines();
            self.expect(&TokenKind::Indent)?;

            let mut rows = Vec::new();
            loop {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Dedent | TokenKind::Eof) {
                    break;
                }
                let line = self.current_line_raw();
                let values: Vec<Value> = split_respecting_quotes(&line, self.delimiter)
                    .into_iter()
                    .map(|p| parse_scalar_raw(p.trim()))
                    .collect::<DecodeResult<Vec<_>>>()?;
                if values.len() != cols.len() {
                    return Err(DecodeError::new(format!(
                        "tabular row has {} values but {} columns",
                        values.len(),
                        cols.len()
                    )));
                }
                let mut obj = Map::new();
                for (c, v) in cols.iter().zip(values) {
                    obj.insert(c.clone(), v);
                }
                rows.push(Value::Object(obj));
                self.bump_to_line_end()?;
            }
            self.expect(&TokenKind::Dedent)?;
            if count != rows.len() {
                return Err(DecodeError::new(format!(
                    "tabular array declared {count} rows but found {}",
                    rows.len()
                )));
            }
            Ok(Value::Array(rows))
        }

        fn parse_expanded_array(&mut self) -> DecodeResult<Value> {
            let items = self.parse_expanded_items(None)?;
            Ok(Value::Array(items))
        }

        fn parse_expanded_items(&mut self, count: Option<usize>) -> DecodeResult<Vec<Value>> {
            let mut items = Vec::new();
            loop {
                self.skip_newlines();
                if !matches!(self.kind(), TokenKind::Dash) {
                    break;
                }
                self.bump();
                items.push(self.parse_expanded_item()?);
            }
            if let Some(c) = count {
                if items.len() != c {
                    return Err(DecodeError::new(format!(
                        "expanded array declared {c} items but found {}",
                        items.len()
                    )));
                }
            }
            Ok(items)
        }

        fn parse_expanded_item(&mut self) -> DecodeResult<Value> {
            // A dash followed by an inline array count form `- [N]: ...`.
            if matches!(self.kind(), TokenKind::LeftBracket)
                && matches!(self.kind_at(1), TokenKind::Number(_))
            {
                let old_delim = self.delimiter;
                self.delimiter = self.detect_delimiter();
                let value = self.parse_array_value()?;
                self.delimiter = old_delim;
                return Ok(value);
            }
            if matches!(self.kind(), TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof) {
                if matches!(self.kind(), TokenKind::Newline)
                    && matches!(self.kind_at(1), TokenKind::Indent)
                {
                    self.bump(); // newline
                    self.bump(); // indent
                    let child = self.parse_block(true)?;
                    self.expect(&TokenKind::Dedent)?;
                    return Ok(child);
                }
                return Ok(Value::Object(Map::new()));
            }

            let is_object_item = matches!(
                self.kind(),
                TokenKind::Identifier(_) | TokenKind::String(_)
            ) && matches!(self.kind_at(1), TokenKind::Colon | TokenKind::LeftBracket);

            if is_object_item {
                // First field sits on the dash line; any deeper fields are
                // indented and parsed as a sibling block.
                let mut map = Map::new();
                self.parse_object_entry(&mut map)?;
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Indent) {
                    self.bump();
                    if let Value::Object(child) = self.parse_object()? {
                        for (k, v) in child {
                            map.insert(k, v);
                        }
                    }
                    self.expect(&TokenKind::Dedent)?;
                }
                Ok(Value::Object(map))
            } else {
                let value = self.parse_scalar()?;
                self.consume_line_end()?;
                Ok(value)
            }
        }

        /// Parse inline items using the current delimiter, reading from raw
        /// input so that quoted segments containing the delimiter survive.
        fn parse_inline_items(&mut self, count: usize) -> DecodeResult<Vec<Value>> {
            let line = self.current_line_raw();
            let items: Vec<Value> = split_respecting_quotes(&line, self.delimiter)
                .into_iter()
                .map(|p| parse_scalar_raw(p.trim()))
                .collect::<DecodeResult<Vec<_>>>()?;
            if items.len() != count {
                return Err(DecodeError::new(format!(
                    "inline array declared {count} items but found {}",
                    items.len()
                )));
            }
            self.bump_to_line_end()?;
            Ok(items)
        }

        fn parse_key(&mut self) -> DecodeResult<String> {
            match self.bump() {
                TokenKind::Identifier(s) | TokenKind::String(s) => Ok(s),
                other => Err(DecodeError::new(format!("expected key, found {other}"))),
            }
        }

        fn parse_count(&mut self) -> DecodeResult<usize> {
            match self.bump() {
                TokenKind::Number(s) => s
                    .parse::<usize>()
                    .map_err(|e| DecodeError::new(format!("invalid array count '{s}': {e}"))),
                other => Err(DecodeError::new(format!("expected count, found {other}"))),
            }
        }

        fn parse_scalar(&mut self) -> DecodeResult<Value> {
            match self.bump() {
                TokenKind::Identifier(s) | TokenKind::String(s) => Ok(Value::String(s)),
                TokenKind::Number(s) => parse_number(&s),
                TokenKind::True => Ok(Value::Bool(true)),
                TokenKind::False => Ok(Value::Bool(false)),
                TokenKind::Null => Ok(Value::Null),
                TokenKind::Reference(raw) => Ok(Value::String(format!("${{{raw}}}"))),
                TokenKind::Error(msg) => Err(DecodeError::new(format!("scanner error: {msg}"))),
                other => Err(DecodeError::new(format!("expected scalar, found {other}"))),
            }
        }

        /// Parses the whole current line as a single scalar value. Used for
        /// object values and bare root/dash scalars where the entire line
        /// (including spaces, commas, tabs and pipes) is the value.
        fn parse_line_value(&mut self) -> DecodeResult<Value> {
            let line = self.current_line_raw();
            self.bump_to_line_end()?;
            let trimmed = line.trim();
            if trimmed.starts_with('[')
                && (!trimmed.ends_with(']') || trimmed.contains(char::is_whitespace))
            {
                return Err(DecodeError::new(format!("unclosed array: {trimmed}")));
            }
            parse_scalar_raw(trimmed)
        }

        fn consume_line_end(&mut self) -> DecodeResult<()> {
            match self.kind() {
                TokenKind::Newline => {
                    self.pos += 1;
                    Ok(())
                }
                TokenKind::Eof | TokenKind::Dedent => Ok(()),
                other => Err(DecodeError::new(format!(
                    "expected end of line, found {other}"
                ))),
            }
        }
    }

    fn parse_scalar_raw(s: &str) -> DecodeResult<Value> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(Value::String(String::new()));
        }
        if s == "true" {
            return Ok(Value::Bool(true));
        }
        if s == "false" {
            return Ok(Value::Bool(false));
        }
        if s == "null" {
            return Ok(Value::Null);
        }
        // TOON treats integer literals with redundant leading zeros (e.g. `05`,
        // `-007`) as strings, not numbers.
        if is_leading_zero_int(s) {
            return Ok(Value::String(s.to_string()));
        }
        if s.starts_with('"') {
            if s.len() >= 2 && s.ends_with('"') {
                let inner = &s[1..s.len() - 1];
                return Ok(Value::String(unescape(inner)?));
            }
            return Err(DecodeError::new("unterminated string literal"));
        }
        if s.starts_with("${") && s.ends_with('}') {
            return Ok(Value::String(s.to_string()));
        }
        // Try number first; any non-numeric word (e.g. `hello`, `reading`) is a
        // bare string value.
        if let Ok(f) = s.parse::<f64>() {
            if f.is_finite() {
                if f == 0.0 {
                    return Ok(Value::Number(0i64.into()));
                }
                if f.fract() == 0.0 && (i64::MIN as f64..=i64::MAX as f64).contains(&f) {
                    return Ok(Value::Number((f as i64).into()));
                }
                if let Some(n) = serde_json::Number::from_f64(f) {
                    return Ok(Value::Number(n));
                }
            }
        }
        Ok(Value::String(s.to_string()))
    }

    /// True when `s` is an integer literal with a redundant leading zero
    /// (e.g. `05`, `007`, `-05`) which TOON decodes as a string.
    fn is_leading_zero_int(s: &str) -> bool {
        let s = s.trim();
        let digits = s.strip_prefix('-').unwrap_or(s);
        digits.len() > 1 && digits.starts_with('0') && digits.bytes().all(|b| b.is_ascii_digit())
    }

    /// True when `s` (the text before a colon) is a valid array key of the form
    /// `name[N]` or `name[N]{cols}`. Anything else is treated as a literal key.
    fn is_array_key(s: &str) -> bool {
        // Array spec at the END: `key[N]`, `key[N]\t`, `key[N]|`,
        // `key[N]{cols}`, where `key` is a plain identifier or a quoted
        // string. This lets quoted keys with brackets
        // (e.g. `"key[test]"[3]`) be recognised as array keys too.
        let close = match s.rfind(']') {
            Some(c) => c,
            _ => return false,
        };
        let open = match s[..close].rfind('[') {
            Some(o) => o,
            _ => return false,
        };
        let inner = &s[open + 1..close];
        let digits_end = inner.find(|c: char| !c.is_ascii_digit()).unwrap_or(inner.len());
        if digits_end == 0 {
            return false;
        }
        let rest = &inner[digits_end..];
        if !rest.is_empty() && rest != "\t" && rest != "|" {
            return false;
        }
        // Optional `{cols}` tail after the closing `]`.
        let tail = &s[close + 1..];
        if !tail.is_empty()
            && (!tail.starts_with('{') || !tail.ends_with('}') || tail.len() < 2)
        {
            return false;
        }
        let key_prefix = &s[..open];
        is_plain_ident(key_prefix)
            || (key_prefix.starts_with('"')
                && key_prefix.ends_with('"')
                && key_prefix.len() >= 2)
    }

    fn is_plain_ident(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
    }

    /// Finds the colon that ends an object key, ignoring colons that appear
    /// inside a quoted segment (e.g. `"order:id":`).
    fn find_key_colon(line: &str) -> Option<usize> {
        let mut in_quote = false;
        let mut chars = line.char_indices().peekable();
        while let Some((i, ch)) = chars.next() {
            match ch {
                '\\' => {
                    chars.next();
                }
                '"' => in_quote = !in_quote,
                ':' if !in_quote => return Some(i),
                _ => {}
            }
        }
        None
    }

    /// Split `s` by `delim`, ignoring delimiters that appear inside a
    /// double-quoted segment. A backslash escapes the following character.
    fn split_respecting_quotes(s: &str, delim: char) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    if let Some(next) = chars.next() {
                        current.push('\\');
                        current.push(next);
                    }
                }
                '"' => {
                    in_quote = !in_quote;
                    current.push(ch);
                }
                c if c == delim && !in_quote => {
                    parts.push(std::mem::take(&mut current));
                }
                c => current.push(c),
            }
        }
        parts.push(current);
        parts
    }

    fn unescape(s: &str) -> DecodeResult<String> {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('u') => {
                        let mut hex = String::new();
                        while hex.len() < 4 && chars.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                            hex.push(chars.next().unwrap());
                        }
                        if hex.len() != 4 {
                            return Err(DecodeError::new(format!(
                                "invalid \\u escape: expected 4 hex digits, got {hex}"
                            )));
                        }
                        let val = u32::from_str_radix(&hex, 16)
                            .map_err(|_| DecodeError::new("invalid \\u escape"))?;
                        let c = char::from_u32(val)
                            .ok_or_else(|| DecodeError::new(format!("invalid unicode scalar U+{val:04X}")))?;
                        out.push(c);
                    }
                    Some(other) => {
                        return Err(DecodeError::new(format!("invalid escape sequence: \\{other}")));
                    }
                    None => return Err(DecodeError::new("unterminated escape sequence")),
                }
            } else {
                out.push(ch);
            }
        }
        Ok(out)
    }

    fn parse_number(s: &str) -> DecodeResult<Value> {
        let f = s
            .parse::<f64>()
            .map_err(|_| DecodeError::new(format!("invalid number '{s}'")))?;
        if !f.is_finite() {
            return Err(DecodeError::new(format!("non-finite number '{s}'")));
        }
        if f == 0.0 {
            return Ok(Value::Number(0i64.into()));
        }
        if f.fract() == 0.0 && (i64::MIN as f64..=i64::MAX as f64).contains(&f) {
            return Ok(Value::Number((f as i64).into()));
        }
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| DecodeError::new(format!("invalid number '{s}'")))
    }
}
