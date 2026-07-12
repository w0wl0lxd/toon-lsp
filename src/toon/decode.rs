//! TOON-to-JSON decoder.
//!
//! A purpose-built line/byte scanner that decodes TOON text into a [`serde_json::Value`].

use crate::toon::error::{DecodeError, DecodeResult};
use serde_json::{Map, Value};

/// Decodes TOON `input` into a [`serde_json::Value`].
///
/// # Errors
/// Returns [`DecodeError`] on malformed TOON (unexpected tokens, scanner
/// errors, or unparseable numbers).
pub fn decode(input: &str) -> DecodeResult<Value> {
    let normalized = remove_block_comments(input)?;
    let mut parser = Parser::new(&normalized);
    parser.parse_document()
}

/// Pre-processes the input to replace block comments `/* ... */` with spaces.
/// This preserves character offsets and line/column positions.
fn remove_block_comments(input: &str) -> DecodeResult<String> {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
        if ch == '"' {
            // Check for triple quotes
            let is_triple = {
                let mut temp = chars.clone();
                temp.next().map(|(_, c)| c) == Some('"')
                    && temp.peek().map(|(_, c)| *c) == Some('"')
            };
            if is_triple {
                out.push_str("\"\"\"");
                chars.next();
                chars.next();
                while let Some((_, c)) = chars.next() {
                    out.push(c);
                    if c == '"' {
                        let is_close_triple = {
                            let mut temp = chars.clone();
                            temp.next().map(|(_, ch)| ch) == Some('"')
                                && temp.peek().map(|(_, ch)| *ch) == Some('"')
                        };
                        if is_close_triple {
                            out.push_str("\"\"");
                            chars.next();
                            chars.next();
                            break;
                        }
                    }
                }
            } else {
                out.push('"');
                let mut escaped = false;
                while let Some((_, c)) = chars.next() {
                    out.push(c);
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    }
                }
            }
        } else if ch == '#' {
            // Line comment: preserve all characters until newline
            out.push(ch);
            while let Some((_, c)) = chars.next() {
                out.push(c);
                if c == '\n' {
                    break;
                }
            }
        } else if ch == '/' && chars.peek().map(|(_, c)| *c) == Some('*') {
            chars.next(); // consume '*'
            out.push(' ');
            out.push(' ');
            let mut found_end = false;
            while let Some((_, c)) = chars.next() {
                if c == '*' && chars.peek().map(|(_, ch)| *ch) == Some('/') {
                    chars.next(); // consume '/'
                    out.push(' ');
                    out.push(' ');
                    found_end = true;
                    break;
                } else if c == '\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
            if !found_end {
                return Err(DecodeError::new("Unterminated block comment"));
            }
        } else {
            out.push(ch);
        }
    }
    Ok(out)
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
    delimiter_stack: Vec<char>,
    line: u32,
    col: u32,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, offset: 0, delimiter_stack: vec![','], line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.input[self.offset..].chars();
        chars.next();
        chars.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += ch.len_utf16() as u32;
        }
        Some(ch)
    }

    fn skip_trivia(&mut self) {
        let active_delim = *self.delimiter_stack.last().unwrap_or(&',');
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\r' {
                self.advance();
            } else if ch == '\t' && active_delim != '\t' {
                self.advance();
            } else if ch == '#' {
                // Line comment: skip to end of line
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn skip_newlines(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                self.skip_trivia();
            } else {
                break;
            }
        }
    }

    fn parse_document(&mut self) -> DecodeResult<Value> {
        self.skip_trivia();
        self.skip_newlines();
        if self.peek().is_none() {
            return Ok(Value::Object(Map::new()));
        }

        let first_indent = if let Some((indent, offset)) = self.peek_next_non_blank_line_indent()? {
            self.consume_indentation(offset);
            indent
        } else {
            0
        };

        let val = self.parse_block(first_indent)?;

        self.skip_trivia();
        self.skip_newlines();
        if self.peek().is_some() {
            return Err(DecodeError::new("trailing tokens after document"));
        }
        Ok(val)
    }

    fn peek_next_non_blank_line_indent(&self) -> DecodeResult<Option<(usize, usize)>> {
        let mut temp_offset = self.offset;
        loop {
            if temp_offset >= self.input.len() {
                return Ok(None);
            }

            let mut spaces = 0;
            let mut has_tab = false;

            while temp_offset < self.input.len() {
                let ch = self.input[temp_offset..].chars().next().unwrap();
                match ch {
                    ' ' => {
                        spaces += 1;
                        temp_offset += 1;
                    }
                    '\t' => {
                        has_tab = true;
                        temp_offset += 1;
                    }
                    _ => break,
                }
            }

            let rest = &self.input[temp_offset..];
            let next_ch = rest.chars().next();
            if next_ch.is_none()
                || next_ch == Some('\n')
                || next_ch == Some('\r')
                || next_ch == Some('#')
            {
                // Blank/comment line, skip to end of line
                while temp_offset < self.input.len() {
                    let ch = self.input[temp_offset..].chars().next().unwrap();
                    temp_offset += ch.len_utf8();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }

            if has_tab {
                return Err(DecodeError::new("Tabs not allowed in indentation"));
            }

            return Ok(Some((spaces, temp_offset)));
        }
    }

    fn consume_indentation(&mut self, offset_after_indent: usize) {
        let mut temp_offset = self.offset;
        while temp_offset < offset_after_indent {
            let ch = self.input[temp_offset..].chars().next().unwrap();
            temp_offset += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += ch.len_utf16() as u32;
            }
        }
        self.offset = offset_after_indent;
    }

    fn parse_block(&mut self, indent: usize) -> DecodeResult<Value> {
        self.skip_trivia();

        // Check for expanded array item
        if self.peek() == Some('-')
            && (self.peek_next() == Some(' ')
                || self.peek_next() == Some('\n')
                || self.peek_next().is_none())
        {
            return self.parse_expanded_array(indent);
        }

        // Check for object or array header
        if self.is_object_entry_on_line() {
            let colon_idx = self.find_unquoted_colon_on_line().unwrap();
            let key_part = &self.input[self.offset..colon_idx];
            if let Some((_, count, delim, cols, is_root)) = self.try_parse_array_header(key_part) {
                if is_root {
                    self.offset = colon_idx + 1;
                    self.col = 1
                        + (self.offset - self.input[..self.offset].rfind('\n').map_or(0, |i| i + 1))
                            as u32;
                    return self.parse_array_or_table(count, delim, cols, indent, true);
                }
            }
            return self.parse_object(indent);
        }

        // Check for literal empty array `[]`
        if self.peek() == Some('[') && self.peek_next() == Some(']') {
            self.advance(); // '['
            self.advance(); // ']'
            self.skip_trivia();
            if self.peek() == Some('\n') {
                self.advance();
            }
            return Ok(Value::Array(Vec::new()));
        }

        if indent == 0 {
            let val = self.parse_scalar_line()?;
            return Ok(val);
        }

        Err(DecodeError::new("missing colon in key-value context"))
    }

    fn parse_expanded_array(&mut self, indent: usize) -> DecodeResult<Value> {
        let mut items = Vec::new();
        loop {
            self.skip_trivia();
            if self.peek() != Some('-') {
                return Err(DecodeError::new("expected '-'"));
            }
            self.advance(); // consume '-'

            let item = self.parse_expanded_item(indent)?;
            items.push(item);

            if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                if next_i == indent {
                    let rest = &self.input[next_offset..];
                    if rest.starts_with('-')
                        && (rest[1..].starts_with(' ')
                            || rest[1..].starts_with('\n')
                            || rest[1..].is_empty())
                    {
                        self.consume_indentation(next_offset);
                        continue;
                    }
                }
            }
            break;
        }
        Ok(Value::Array(items))
    }

    fn parse_expanded_item(&mut self, parent_indent: usize) -> DecodeResult<Value> {
        self.skip_trivia();

        if self.peek() == Some('[') {
            return self.parse_array_value(parent_indent);
        }

        if self.rest_of_line_is_empty() {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    self.advance();
                    break;
                }
                self.advance();
            }

            if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                if next_i > parent_indent {
                    self.consume_indentation(next_offset);
                    let child = self.parse_block(next_i)?;
                    return Ok(child);
                }
            }
            return Ok(Value::Object(Map::new()));
        }

        if self.is_object_entry_on_line() {
            let mut map = Map::new();
            let field_indent = parent_indent + 2;

            self.parse_object_entry(&mut map, field_indent)?;

            loop {
                if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                    if next_i == field_indent {
                        let rest = &self.input[next_offset..];
                        if find_unquoted_colon_on_line(rest).is_some() {
                            self.consume_indentation(next_offset);
                            self.parse_object_entry(&mut map, field_indent)?;
                            continue;
                        }
                    }
                }
                break;
            }
            Ok(Value::Object(map))
        } else {
            let val = self.parse_scalar_line()?;
            Ok(val)
        }
    }

    fn parse_object(&mut self, indent: usize) -> DecodeResult<Value> {
        let mut map = Map::new();
        loop {
            if !self.is_object_entry_on_line() {
                return Err(DecodeError::new("expected key-value entry in object"));
            }

            self.parse_object_entry(&mut map, indent)?;

            if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                if next_i == indent {
                    let rest = &self.input[next_offset..];
                    if find_unquoted_colon_on_line(rest).is_some() {
                        self.consume_indentation(next_offset);
                        continue;
                    }
                }
            }
            break;
        }
        Ok(Value::Object(map))
    }

    fn parse_object_entry(
        &mut self,
        map: &mut Map<String, Value>,
        parent_indent: usize,
    ) -> DecodeResult<()> {
        let colon_idx = self
            .find_unquoted_colon_on_line()
            .ok_or_else(|| DecodeError::new("missing colon in object entry"))?;

        let key_part = &self.input[self.offset..colon_idx];
        self.offset = colon_idx + 1;
        self.col =
            1 + (self.offset - self.input[..self.offset].rfind('\n').map_or(0, |i| i + 1)) as u32;

        if let Some((name, count, delim, cols, _)) = self.try_parse_array_header(key_part) {
            let value = self.parse_array_or_table(count, delim, cols, parent_indent, true)?;
            map.insert(name, value);
        } else {
            let name = self.parse_key_string(key_part)?;
            let value = self.parse_value_after_colon(parent_indent)?;
            map.insert(name, value);
        }
        Ok(())
    }

    fn parse_value_after_colon(&mut self, parent_indent: usize) -> DecodeResult<Value> {
        self.skip_trivia();

        if self.peek() == Some('[') {
            let line_end =
                self.input[self.offset..].find('\n').map_or(self.input.len(), |i| self.offset + i);
            if !self.input[self.offset..line_end].contains(']') {
                return Err(DecodeError::new("Unterminated inline array"));
            }
        }

        if self.peek() == Some('[') && self.peek_next() == Some(']') {
            self.advance(); // '['
            self.advance(); // ']'
            self.skip_trivia();
            if self.peek() == Some('\n') {
                self.advance();
            }
            return Ok(Value::Array(Vec::new()));
        }

        if self.rest_of_line_is_empty() {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    self.advance();
                    break;
                }
                self.advance();
            }
            if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                if next_i > parent_indent {
                    self.consume_indentation(next_offset);
                    let child = self.parse_block(next_i)?;
                    return Ok(child);
                }
            }
            return Ok(Value::Object(Map::new()));
        }

        let val = self.parse_scalar_line()?;
        Ok(val)
    }

    fn parse_array_value(&mut self, parent_indent: usize) -> DecodeResult<Value> {
        if self.peek() != Some('[') {
            return Err(DecodeError::new("expected '['"));
        }
        self.advance(); // '['

        let mut count_str = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                count_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if count_str.is_empty() {
            return Err(DecodeError::new("expected count"));
        }
        let count = count_str.parse::<usize>().map_err(|e| DecodeError::new(e.to_string()))?;

        let mut delim = None;
        if let Some(ch) = self.peek() {
            if ch != ']' {
                delim = Some(ch);
                self.advance();
            }
        }

        if self.peek() != Some(']') {
            return Err(DecodeError::new("expected ']'"));
        }
        self.advance(); // ']'

        let mut cols = None;
        self.skip_trivia();
        if self.peek() == Some('{') {
            self.advance(); // '{'
            let mut brace_content = String::new();
            loop {
                let Some(ch) = self.advance() else {
                    return Err(DecodeError::new("expected '}'"));
                };
                if ch == '}' {
                    break;
                }
                brace_content.push(ch);
            }
            let active_delim = delim.unwrap_or(',');
            cols = Some(parse_delimited_strings(&brace_content, active_delim)?);
        }

        self.parse_array_or_table(count, delim, cols, parent_indent, false)
    }

    fn parse_array_or_table(
        &mut self,
        count: usize,
        delim: Option<char>,
        cols: Option<Vec<String>>,
        parent_indent: usize,
        colon_consumed: bool,
    ) -> DecodeResult<Value> {
        let active_delim = delim.unwrap_or(',');
        self.delimiter_stack.push(active_delim);
        let res = self.parse_array_or_table_inner(
            count,
            active_delim,
            cols,
            parent_indent,
            colon_consumed,
        );
        self.delimiter_stack.pop();
        res
    }

    fn parse_array_or_table_inner(
        &mut self,
        count: usize,
        active_delim: char,
        cols: Option<Vec<String>>,
        parent_indent: usize,
        colon_consumed: bool,
    ) -> DecodeResult<Value> {
        self.skip_trivia();

        if let Some(columns) = cols {
            if !colon_consumed {
                if self.peek() != Some(':') {
                    return Err(DecodeError::new("expected ':' after tabular header"));
                }
                self.advance(); // consume ':'
            }
            self.skip_trivia();
            if self.peek() == Some('\n') {
                self.advance();
            }

            let mut rows = Vec::new();
            if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                if next_i > parent_indent {
                    self.consume_indentation(next_offset);
                    let row_indent = next_i;
                    loop {
                        let row_vals = self.parse_delimited_row(active_delim)?;
                        if row_vals.len() != columns.len() {
                            return Err(DecodeError::new(format!(
                                "tabular row has {} values but {} columns",
                                row_vals.len(),
                                columns.len()
                            )));
                        }

                        let mut obj = Map::new();
                        for (col, val) in columns.iter().zip(row_vals) {
                            obj.insert(col.clone(), val);
                        }
                        rows.push(Value::Object(obj));

                        if let Some((next_i, next_offset)) =
                            self.peek_next_non_blank_line_indent()?
                        {
                            if next_i == row_indent {
                                self.consume_indentation(next_offset);
                                continue;
                            }
                        }
                        break;
                    }
                }
            }

            if rows.len() != count {
                return Err(DecodeError::new("tabular row count mismatch with header length"));
            }
            return Ok(Value::Array(rows));
        }

        if !colon_consumed {
            if self.peek() != Some(':') {
                return Err(DecodeError::new("expected ':' after array header"));
            }
            self.advance(); // consume ':'
        }
        self.skip_trivia();

        if self.peek() == Some('\n') || self.peek().is_none() {
            if self.peek() == Some('\n') {
                self.advance();
            }
            if let Some((next_i, next_offset)) = self.peek_next_non_blank_line_indent()? {
                if next_i > parent_indent {
                    let rest = &self.input[next_offset..];
                    if rest.starts_with('-')
                        && (rest[1..].starts_with(' ')
                            || rest[1..].starts_with('\n')
                            || rest[1..].is_empty())
                    {
                        self.consume_indentation(next_offset);
                        let val = self.parse_expanded_array(next_i)?;
                        if let Value::Array(ref arr) = val {
                            if arr.len() != count {
                                return Err(DecodeError::new("array length mismatch"));
                            }
                        }
                        return Ok(val);
                    }
                }
            }

            if count == 0 {
                return Ok(Value::Array(Vec::new()));
            }
            return Err(DecodeError::new("array length mismatch"));
        }

        let items = self.parse_inline_items(active_delim)?;
        if items.len() != count {
            return Err(DecodeError::new("array length mismatch"));
        }
        Ok(Value::Array(items))
    }

    fn parse_delimited_row(&mut self, delim: char) -> DecodeResult<Vec<Value>> {
        self.parse_inline_items(delim)
    }

    fn parse_inline_items(&mut self, delim: char) -> DecodeResult<Vec<Value>> {
        let mut items = Vec::new();
        let mut line_str = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                break;
            }
            line_str.push(ch);
            self.advance();
        }

        if line_str.trim_start().starts_with('[') && !line_str.contains(']') {
            return Err(DecodeError::new("Unterminated inline array"));
        }

        let trimmed_line = truncate_comment_from_line(&line_str);
        let mut chars = trimmed_line.chars().peekable();

        loop {
            while let Some(&c) = chars.peek() {
                if c == ' ' || c == '\r' || (c == '\t' && delim != '\t') {
                    chars.next();
                } else {
                    break;
                }
            }

            if chars.peek().is_none() {
                break;
            }

            let val = self.parse_scalar_from_chars(&mut chars, delim)?;
            items.push(val);

            while let Some(&c) = chars.peek() {
                if c == ' ' || c == '\r' || (c == '\t' && delim != '\t') {
                    chars.next();
                } else {
                    break;
                }
            }

            if let Some(&ch) = chars.peek() {
                if ch == delim {
                    chars.next();
                    if chars.peek().is_none() {
                        items.push(Value::String(String::new()));
                    }
                } else {
                    return Err(DecodeError::new("expected delimiter"));
                }
            }
        }
        Ok(items)
    }

    fn parse_escaped_char(chars: &mut std::iter::Peekable<std::str::Chars>) -> DecodeResult<char> {
        let Some(ch) = chars.next() else {
            return Err(DecodeError::new("Unterminated escape sequence"));
        };
        match ch {
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            'u' => {
                let mut hex = String::new();
                for _ in 0..4 {
                    if let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() {
                            hex.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                if hex.len() != 4 {
                    return Err(DecodeError::new("Invalid \\u escape"));
                }
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|e| DecodeError::new(format!("invalid unicode: {e}")))?;
                let c = char::from_u32(code)
                    .ok_or_else(|| DecodeError::new("invalid unicode code point"))?;
                Ok(c)
            }
            other => Err(DecodeError::new(format!("Invalid escape: \\{other}"))),
        }
    }

    fn parse_scalar_from_chars(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        delim: char,
    ) -> DecodeResult<Value> {
        if chars.peek() == Some(&'"') {
            let mut temp = chars.clone();
            temp.next();
            let is_triple = temp.next() == Some('"') && temp.peek() == Some(&'"');
            if is_triple {
                chars.next();
                chars.next();
                chars.next();
                let mut val = String::new();
                loop {
                    let Some(ch) = chars.next() else {
                        return Err(DecodeError::new("Unterminated block string"));
                    };
                    if ch == '"' {
                        let mut temp_close = chars.clone();
                        if temp_close.next() == Some('"') && temp_close.peek() == Some(&'"') {
                            chars.next();
                            chars.next();
                            break;
                        }
                    }
                    val.push(ch);
                }
                return Ok(Value::String(val));
            } else {
                chars.next();
                let mut val = String::new();
                loop {
                    let Some(ch) = chars.next() else {
                        return Err(DecodeError::new("Unterminated quoted string"));
                    };
                    if ch == '\\' {
                        val.push(Self::parse_escaped_char(chars)?);
                    } else if ch == '"' {
                        break;
                    } else {
                        val.push(ch);
                    }
                }
                return Ok(Value::String(val));
            }
        }

        if chars.peek() == Some(&'$') {
            let mut temp = chars.clone();
            temp.next();
            if temp.peek() == Some(&'{') {
                chars.next();
                chars.next();
                let mut raw = String::new();
                loop {
                    let Some(ch) = chars.next() else {
                        return Err(DecodeError::new("Unterminated reference"));
                    };
                    if ch == '}' {
                        break;
                    }
                    raw.push(ch);
                }
                return Ok(Value::String(format!("${{{raw}}}")));
            }
        }

        let mut s = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == delim {
                break;
            }
            s.push(ch);
            chars.next();
        }
        let s = s.trim().to_string();

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
        if let Ok(val) = parse_number(&s) {
            return Ok(val);
        }

        Ok(Value::String(s))
    }

    fn parse_scalar_line(&mut self) -> DecodeResult<Value> {
        let mut line_str = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                break;
            }
            line_str.push(ch);
            self.advance();
        }

        let trimmed_line = truncate_comment_from_line(&line_str);
        let s = trimmed_line.trim();

        if s.starts_with("\"\"\"") {
            if s.ends_with("\"\"\"") && s.len() >= 6 {
                let mut chars = s.chars().peekable();
                return self.parse_scalar_from_chars(&mut chars, '\n');
            } else {
                return Err(DecodeError::new("Unterminated block string"));
            }
        }

        if s.starts_with('"') {
            if s.ends_with('"') && s.len() >= 2 {
                let mut chars = s.chars().peekable();
                return self.parse_scalar_from_chars(&mut chars, '\n');
            } else {
                return Err(DecodeError::new("Unterminated quoted string"));
            }
        }

        if s.starts_with("${") && s.ends_with('}') {
            let mut chars = s.chars().peekable();
            return self.parse_scalar_from_chars(&mut chars, '\n');
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
        if let Ok(val) = parse_number(s) {
            return Ok(val);
        }

        Ok(Value::String(s.to_string()))
    }

    fn is_object_entry_on_line(&self) -> bool {
        find_unquoted_colon_on_line(&self.input[self.offset..]).is_some()
    }

    fn find_unquoted_colon_on_line(&self) -> Option<usize> {
        find_unquoted_colon_on_line(&self.input[self.offset..]).map(|idx| self.offset + idx)
    }

    fn rest_of_line_is_empty(&self) -> bool {
        let line = if let Some(idx) = self.input[self.offset..].find('\n') {
            &self.input[self.offset..self.offset + idx]
        } else {
            &self.input[self.offset..]
        };
        let trimmed = truncate_comment_from_line(line);
        trimmed.trim().is_empty()
    }

    fn parse_key_string(&self, s: &str) -> DecodeResult<String> {
        let s = s.trim();
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            let mut out = String::new();
            let mut chars = s[1..s.len() - 1].chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    out.push(Self::parse_escaped_char(&mut chars)?);
                } else {
                    out.push(ch);
                }
            }
            Ok(out)
        } else {
            Ok(s.to_string())
        }
    }

    fn try_parse_array_header(
        &self,
        key_part: &str,
    ) -> Option<(String, usize, Option<char>, Option<Vec<String>>, bool)> {
        let key_part = key_part.trim();

        if key_part.ends_with('}') {
            let brace_start = key_part.rfind('{')?;
            let brace_content = &key_part[brace_start + 1..key_part.len() - 1];

            let array_part = key_part[..brace_start].trim();
            if !array_part.ends_with(']') {
                return None;
            }
            let bracket_start = array_part.rfind('[')?;
            let bracket_content = &array_part[bracket_start + 1..array_part.len() - 1];

            let (count, delim) = parse_bracket_content(bracket_content)?;
            let col_delim = delim.unwrap_or(',');
            let cols = parse_delimited_strings(brace_content, col_delim).ok()?;

            let name_raw = array_part[..bracket_start].trim().to_string();
            let is_root = name_raw.is_empty();
            let name = self.parse_key_string(&name_raw).ok()?;
            if !name_raw.starts_with('"')
                && (name.contains('[')
                    || name.contains(']')
                    || name.contains('{')
                    || name.contains('}'))
            {
                return None;
            }
            return Some((name, count, delim, Some(cols), is_root));
        }

        if key_part.ends_with(']') {
            let bracket_start = key_part.rfind('[')?;
            let bracket_content = &key_part[bracket_start + 1..key_part.len() - 1];

            let (count, delim) = parse_bracket_content(bracket_content)?;
            let name_raw = key_part[..bracket_start].trim().to_string();
            let is_root = name_raw.is_empty();
            let name = self.parse_key_string(&name_raw).ok()?;
            if !name_raw.starts_with('"')
                && (name.contains('[')
                    || name.contains(']')
                    || name.contains('{')
                    || name.contains('}'))
            {
                return None;
            }
            return Some((name, count, delim, None, is_root));
        }

        None
    }
}

fn find_unquoted_colon_on_line(s: &str) -> Option<usize> {
    let mut in_quote = false;
    let mut escaped = false;
    let mut in_ref = false;
    let mut chars = s.char_indices();
    while let Some((idx, ch)) = chars.next() {
        if ch == '\n' {
            break;
        }
        if in_quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if in_ref {
            if ch == '}' {
                in_ref = false;
            }
        } else {
            if ch == '"' {
                in_quote = true;
            } else if ch == '$' {
                let mut temp = chars.clone();
                if temp.next().map(|(_, c)| c) == Some('{') {
                    in_ref = true;
                }
            } else if ch == '#' {
                break;
            } else if ch == ':' {
                return Some(idx);
            }
        }
    }
    None
}

fn truncate_comment_from_line(line: &str) -> &str {
    let mut in_quote = false;
    let mut escaped = false;
    let mut in_ref = false;
    let mut chars = line.char_indices();
    while let Some((idx, ch)) = chars.next() {
        if in_quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_quote = false;
            }
        } else if in_ref {
            if ch == '}' {
                in_ref = false;
            }
        } else {
            if ch == '"' {
                in_quote = true;
            } else if ch == '$' {
                let mut temp = chars.clone();
                if temp.next().map(|(_, c)| c) == Some('{') {
                    in_ref = true;
                }
            } else if ch == '#' {
                return &line[..idx];
            }
        }
    }
    line
}

fn parse_bracket_content(s: &str) -> Option<(usize, Option<char>)> {
    let s = s.trim_start_matches(|c| c == ' ' || c == '\r');
    if s.is_empty() {
        return None;
    }

    let mut count_str = String::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            count_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if count_str.is_empty() {
        return None;
    }
    let count = count_str.parse::<usize>().ok()?;

    let rest: String = chars.collect();
    let delim = if rest.is_empty() { None } else { Some(rest.chars().next().unwrap()) };

    Some((count, delim))
}

fn parse_delimited_strings(s: &str, delim: char) -> DecodeResult<Vec<String>> {
    let mut parts = Vec::new();
    let mut chars = s.chars().peekable();

    loop {
        while chars.peek().map(|&c| c == ' ' || c == '\r').unwrap_or(false) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }

        let mut item = String::new();
        if chars.peek() == Some(&'"') {
            chars.next(); // consume '"'
            loop {
                let Some(ch) = chars.next() else {
                    return Err(DecodeError::new("Unterminated quoted string in columns"));
                };
                if ch == '\\' {
                    item.push(Parser::parse_escaped_char(&mut chars)?);
                } else if ch == '"' {
                    break;
                } else {
                    item.push(ch);
                }
            }
            while chars.peek().map(|&c| c == ' ').unwrap_or(false) {
                chars.next();
            }
            if let Some(&ch) = chars.peek() {
                if ch == delim {
                    chars.next();
                } else {
                    return Err(DecodeError::new(format!(
                        "expected delimiter {delim} after quoted column"
                    )));
                }
            }
        } else {
            while let Some(&ch) = chars.peek() {
                if ch == delim {
                    chars.next();
                    break;
                }
                if ch == '\n' {
                    break;
                }
                item.push(ch);
                chars.next();
            }
            item = item.trim().to_string();
        }
        parts.push(item);
    }
    Ok(parts)
}

fn parse_number(s: &str) -> DecodeResult<Value> {
    if has_leading_zero(s) {
        return Err(DecodeError::new("leading zero not allowed in number"));
    }

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

    if let Ok(n) = s.parse::<i64>() {
        return Ok(Value::Number(n.into()));
    }
    if let Ok(n) = s.parse::<u64>() {
        return Ok(Value::Number(n.into()));
    }

    let f = s.parse::<f64>().map_err(|_| DecodeError::new(format!("invalid number '{s}'")))?;
    if !f.is_finite() {
        return Err(DecodeError::new(format!("non-finite number '{s}'")));
    }
    if f == 0.0 {
        return Ok(Value::Number(0i64.into()));
    }
    if f.fract() == 0.0 && (i64::MIN as f64..=i64::MAX as f64).contains(&f) {
        #[allow(clippy::cast_possible_truncation)]
        return Ok(Value::Number((f as i64).into()));
    }
    serde_json::Number::from_f64(f)
        .map(Value::Number)
        .ok_or_else(|| DecodeError::new(format!("invalid number '{s}'")))
}

fn has_leading_zero(s: &str) -> bool {
    let body = s.strip_prefix('-').unwrap_or(s);
    if body.starts_with('0') && body.len() > 1 {
        if let Some(c) = body.chars().nth(1) {
            if c.is_ascii_digit() {
                return true;
            }
        }
    }
    false
}
