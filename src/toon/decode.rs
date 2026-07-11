//! TOON-to-JSON decoder.
//!
//! Two feature-gated prototypes share the [`decode`] signature and the
//! `tests/toon_codec_decode.rs` suite:
//!
//! - `decoder_a` (default): drives the existing LSP [`crate::parser::Scanner`]
//!   token stream.
//! - `decoder_b`: a purpose-built line/byte scanner (added in a later task).
//!
//! References (`${path}`) have no JSON counterpart, so they are decoded back to
//! their literal `${path}` string form.

#[cfg(all(feature = "decoder_a", feature = "decoder_b"))]
compile_error!(
    "features `decoder_a` and `decoder_b` are mutually exclusive; \
     build with exactly one (e.g. --no-default-features --features decoder_b)"
);

#[cfg(feature = "decoder_a")]
pub use scanner_driven::decode;

#[cfg(feature = "decoder_a")]
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
        let mut parser = Parser { tokens, pos: 0 };
        let value = parser.parse_document()?;
        Ok(value)
    }

    struct Parser {
        tokens: Vec<Token>,
        pos: usize,
    }

    impl Parser {
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

        fn parse_document(&mut self) -> DecodeResult<Value> {
            self.skip_newlines();
            if matches!(self.kind(), TokenKind::Eof) {
                return Ok(Value::Object(Map::new()));
            }
            let value = self.parse_block()?;
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
        fn parse_block(&mut self) -> DecodeResult<Value> {
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
                    self.parse_array_value()
                }
                TokenKind::Identifier(_) | TokenKind::String(_)
                    if self.starts_object_entry() =>
                {
                    self.parse_object()
                }
                _ => {
                    let value = self.parse_scalar()?;
                    self.consume_line_end()?;
                    Ok(value)
                }
            }
        }

        /// True if the current line is `key:` or `key[...`, i.e. an object entry
        /// rather than a bare scalar.
        fn starts_object_entry(&self) -> bool {
            matches!(
                self.kind_at(1),
                TokenKind::Colon | TokenKind::LeftBracket
            )
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

        /// Parses one `key: value` / `key[N]: ...` / `key[N]{cols}:` entry into
        /// `map`, including any indented continuation block.
        fn parse_object_entry(&mut self, map: &mut Map<String, Value>) -> DecodeResult<()> {
            let key = self.parse_key()?;
            if matches!(self.kind(), TokenKind::LeftBracket) {
                let value = self.parse_array_value()?;
                map.insert(key, value);
                return Ok(());
            }
            self.expect(&TokenKind::Colon)?;
            if matches!(self.kind(), TokenKind::LeftBracket)
                && matches!(self.kind_at(1), TokenKind::RightBracket)
            {
                // Literal empty array `key: []`.
                self.bump();
                self.bump();
                self.consume_line_end()?;
                map.insert(key, Value::Array(Vec::new()));
                return Ok(());
            }
            if matches!(self.kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Indent) {
                    self.bump();
                    let child = self.parse_block()?;
                    self.expect(&TokenKind::Dedent)?;
                    map.insert(key, child);
                } else {
                    // Bare `key:` with no indented block is an empty object.
                    map.insert(key, Value::Object(Map::new()));
                }
            } else {
                let value = self.parse_scalar()?;
                self.consume_line_end()?;
                map.insert(key, value);
            }
            Ok(())
        }

        /// Parses `[N]:` (inline scalar array), `[N]{cols}:` (tabular), or
        /// `[N]:` followed by an indented expanded block.
        fn parse_array_value(&mut self) -> DecodeResult<Value> {
            self.expect(&TokenKind::LeftBracket)?;
            let count = self.parse_count()?;
            self.expect(&TokenKind::RightBracket)?;

            if matches!(self.kind(), TokenKind::LeftBrace) {
                return self.parse_tabular();
            }

            self.expect(&TokenKind::Colon)?;
            if matches!(self.kind(), TokenKind::Newline | TokenKind::Eof | TokenKind::Dedent) {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Dedent) {
                    // The block is already closed (e.g. last field in an
                    // expanded block). Emit an empty array regardless of count.
                    return Ok(Value::Array(Vec::new()));
                }
                if matches!(self.kind(), TokenKind::Indent) {
                    self.bump();
                    let items = self.parse_expanded_items()?;
                    self.expect(&TokenKind::Dedent)?;
                    Ok(Value::Array(items))
                } else if count == 0 {
                    // `key[0]:` with no indented block is an empty array.
                    Ok(Value::Array(Vec::new()))
                } else {
                    Ok(Value::Array(Vec::new()))
                }
            } else {
                let items = self.parse_inline_items()?;
                self.consume_line_end()?;
                Ok(Value::Array(items))
            }
        }

        fn parse_tabular(&mut self) -> DecodeResult<Value> {
            self.expect(&TokenKind::LeftBrace)?;
            let mut cols = Vec::new();
            cols.push(self.parse_key()?);
            while matches!(self.kind(), TokenKind::Comma) {
                self.bump();
                cols.push(self.parse_key()?);
            }
            self.expect(&TokenKind::RightBrace)?;
            self.expect(&TokenKind::Colon)?;
            self.skip_newlines();
            self.expect(&TokenKind::Indent)?;

            let mut rows = Vec::new();
            loop {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Dedent | TokenKind::Eof) {
                    break;
                }
                let mut values = Vec::with_capacity(cols.len());
                values.push(self.parse_scalar()?);
                while matches!(self.kind(), TokenKind::Comma) {
                    self.bump();
                    values.push(self.parse_scalar()?);
                }
                self.consume_line_end()?;
                if values.len() != cols.len() {
                    return Err(DecodeError::new(format!(
                        "tabular row has {} values but {} columns",
                        values.len(),
                        cols.len()
                    )));
                }
                let mut obj = Map::new();
                for (col, val) in cols.iter().zip(values) {
                    obj.insert(col.clone(), val);
                }
                rows.push(Value::Object(obj));
            }
            self.expect(&TokenKind::Dedent)?;
            Ok(Value::Array(rows))
        }

        fn parse_expanded_array(&mut self) -> DecodeResult<Value> {
            let items = self.parse_expanded_items()?;
            Ok(Value::Array(items))
        }

        fn parse_expanded_items(&mut self) -> DecodeResult<Vec<Value>> {
            let mut items = Vec::new();
            loop {
                self.skip_newlines();
                if !matches!(self.kind(), TokenKind::Dash) {
                    break;
                }
                self.bump();
                let item = self.parse_expanded_item()?;
                items.push(item);
            }
            Ok(items)
        }

        fn parse_expanded_item(&mut self) -> DecodeResult<Value> {
            // A dash with no content or child is an empty object.
            if matches!(self.kind(), TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof) {
                self.skip_newlines();
                if matches!(self.kind(), TokenKind::Indent) {
                    self.bump();
                    let child = self.parse_block()?;
                    self.expect(&TokenKind::Dedent)?;
                    return Ok(child);
                }
                return Ok(Value::Object(Map::new()));
            }

            // A dash followed by an inline array count form `- [N]: ...`.
            if matches!(self.kind(), TokenKind::LeftBracket)
                && matches!(self.kind_at(1), TokenKind::Number(_))
            {
                return self.parse_array_value();
            }

            let is_object_item = matches!(
                self.kind(),
                TokenKind::Identifier(_) | TokenKind::String(_)
            ) && matches!(
                self.kind_at(1),
                TokenKind::Colon | TokenKind::LeftBracket
            );

            if is_object_item {
                let mut map = Map::new();
                self.parse_object_entry(&mut map)?;
                if matches!(self.kind(), TokenKind::Indent) {
                    self.bump();
                    while !matches!(self.kind(), TokenKind::Dedent | TokenKind::Eof) {
                        self.skip_newlines();
                        if matches!(self.kind(), TokenKind::Dedent | TokenKind::Eof) {
                            break;
                        }
                        self.parse_object_entry(&mut map)?;
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

        fn parse_inline_items(&mut self) -> DecodeResult<Vec<Value>> {
            let mut items = Vec::new();
            items.push(self.parse_scalar()?);
            while matches!(self.kind(), TokenKind::Comma) {
                self.bump();
                items.push(self.parse_scalar()?);
            }
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

    /// Parses a TOON number token into a JSON number.
    ///
    /// TOON normalizes numeric values: a value whose magnitude is integral
    /// (including via an exponent, and both `-0` and `-0.0`) decodes as a JSON
    /// integer, while a genuine fractional value decodes as a minimal float.
    /// Hexadecimal literals (`0x1F`, `-0x10`) are a toon-lsp extension and
    /// decode as integers.
    fn parse_number(s: &str) -> DecodeResult<Value> {
        // Hexadecimal extension: 0x.. / -0x..
        let (hex_neg, hex_body) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        if let Some(hex) = hex_body.strip_prefix("0x").or_else(|| hex_body.strip_prefix("0X")) {
            let magnitude = i64::from_str_radix(hex, 16)
                .map_err(|e| DecodeError::new(format!("invalid hex number '{s}': {e}")))?;
            let value = if hex_neg { -magnitude } else { magnitude };
            return Ok(Value::Number(value.into()));
        }

        // Plain integers (covers the full i64/u64 range without precision loss).
        if let Ok(n) = s.parse::<i64>() {
            return Ok(Value::Number(n.into()));
        }
        if let Ok(n) = s.parse::<u64>() {
            return Ok(Value::Number(n.into()));
        }

        // Everything else: parse as float and normalize integral values.
        let f = s
            .parse::<f64>()
            .map_err(|_| DecodeError::new(format!("invalid number '{s}'")))?;
        if !f.is_finite() {
            return Err(DecodeError::new(format!("non-finite number '{s}'")));
        }
        if f == 0.0 {
            // Collapses -0, -0.0, 0e1, -0e1 to integer zero.
            return Ok(Value::Number(0i64.into()));
        }
        // Integral within the safe-integer range decodes as an integer.
        if f.fract() == 0.0 && (i64::MIN as f64..=i64::MAX as f64).contains(&f) {
            #[allow(clippy::cast_possible_truncation)]
            return Ok(Value::Number((f as i64).into()));
        }
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| DecodeError::new(format!("invalid number '{s}'")))
    }
}
