//! Scanner-driven TOON decoder (prototype A).
//!
//! This decoder reuses the existing LSP [`crate::parser::Scanner`] token stream
//! and implements a recursive-descent parser over those tokens.

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
    let mut parser = Parser { tokens, pos: 0, delimiter: ',', input };
    let value = parser.parse_document()?;
    Ok(value)
}

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    delimiter: char,
    input: &'a str,
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

fn detect_delimiter(&mut self) -> char {
        // Look at raw input after the count token to find delimiter
        if let Some(tok) = self.tokens.get(self.pos) {
            let after_count = tok.span.end.offset as usize;
            if after_count < self.input.len() {
                let ch = self.input.as_bytes()[after_count];
                if ch == b'\t' || ch == b'|' {
                    return ch as char;
                }
            }
        }
        ','
    }

    /// Parse inline items using current delimiter.
    /// For custom delimiters (tab/pipe), we must parse from raw input
    /// since Scanner only tokenizes commas.
    fn parse_inline_items(&mut self) -> DecodeResult<Vec<Value>> {
        if self.delimiter == ',' {
            // Fast path: Scanner tokenizes commas
            let mut items = Vec::new();
            items.push(self.parse_scalar()?);
            while matches!(self.kind(), TokenKind::Comma) {
                self.bump();
                items.push(self.parse_scalar()?);
            }
            Ok(items)
        } else {
            // Custom delimiter: parse from raw input line
            self.parse_inline_items_raw()
        }
    }

    /// Parse inline items from raw input using custom delimiter.
    /// Assumes current token is the first scalar, and we're on the same line.
    fn parse_inline_items_raw(&mut self) -> DecodeResult<Vec<Value>> {
        // Get the rest of the line from current position
        let mut items = Vec::new();
        
        // First, parse the current scalar
        items.push(self.parse_scalar()?);
        
        // Find line end in raw input
        let line_start = if let Some(tok) = self.tokens.get(self.pos - 1) {
            tok.span.start.offset as usize
        } else {
            return Ok(items);
        };
        
        let line_end = self.input[line_start..].find('\n').map_or(self.input.len(), |i| line_start + i);
        let line_text = &self.input[line_start..line_end];
        
        // Split by delimiter (skip the first item we already parsed)
        let parts: Vec<&str> = line_text.split(self.delimiter).collect();
        if parts.len() <= 1 {
            return Ok(items);
        }
        
        // Parse remaining parts as scalars
        for part in &parts[1..] {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                items.push(Value::String(String::new()));
            } else {
                items.push(parse_scalar_raw(trimmed)?);
            }
        }
        
        // Advance token position past the inline items
        // We need to consume tokens until we hit Newline or Eof
        while !matches!(self.kind(), TokenKind::Newline | TokenKind::Eof) {
            self.bump();
        }
        
        Ok(items)
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
        if matches!(self.kind(), TokenKind::Newline | TokenKind::Eof) {
            self.skip_newlines();
            if matches!(self.kind(), TokenKind::Indent) {
                self.bump();
                let child = self.parse_block()?;
                self.expect(&TokenKind::Dedent)?;
                return Ok(child);
            }
            return Ok(Value::Null);
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
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        // Unquote and unescape
        let inner = &s[1..s.len()-1];
        return Ok(Value::String(unescape(inner)));
    }
    if s.starts_with("${") && s.ends_with('}') {
        return Ok(Value::String(s.to_string()));
    }
    // Try number
    parse_number(s)
}

fn unescape(s: &str) -> String {
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
                    for _ in 0..4 {
                        if let Some(&c) = chars.peek() {
                            if c.is_ascii_hexdigit() {
                                hex.push(chars.next().unwrap());
                            }
                        }
                    }
                    if let Ok(val) = u32::from_str_radix(&hex, 16) {
                        if let Some(c) = char::from_u32(val) {
                            out.push(c);
                        }
                    }
                }
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn parse_number(s: &str) -> DecodeResult<Value> {
    serde_json::from_str::<Value>(s)
        .ok()
        .filter(Value::is_number)
        .ok_or_else(|| DecodeError::new(format!("invalid number '{s}'")))
}