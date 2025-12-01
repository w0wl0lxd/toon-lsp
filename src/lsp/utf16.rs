//! UTF-16 position conversion utilities for LSP compliance.
//!
//! LSP uses UTF-16 code units for character positions, while Rust strings
//! use UTF-8. This module provides conversion functions between the two.

use crate::ast::Span;
use tower_lsp::lsp_types::{Position, Range};

/// Convert a UTF-8 column offset to UTF-16 code units.
///
/// Characters outside the Basic Multilingual Plane (BMP) require 2 UTF-16
/// code units (surrogate pairs) but may use up to 4 UTF-8 bytes.
///
/// # Arguments
/// * `line_text` - The text of the line (UTF-8)
/// * `utf8_column` - Column offset in UTF-8 bytes
///
/// # Returns
/// Column offset in UTF-16 code units
pub fn utf8_to_utf16_col(line_text: &str, utf8_column: u32) -> u32 {
    let utf8_col = utf8_column as usize;
    if utf8_col >= line_text.len() {
        // If column is beyond line end, count all chars
        return line_text.chars().map(char_utf16_len).sum();
    }

    line_text[..utf8_col].chars().map(char_utf16_len).sum()
}

/// Convert a UTF-16 column offset to UTF-8 bytes.
///
/// # Arguments
/// * `line_text` - The text of the line (UTF-8)
/// * `utf16_column` - Column offset in UTF-16 code units
///
/// # Returns
/// Column offset in UTF-8 bytes
pub fn utf16_to_utf8_col(line_text: &str, utf16_column: u32) -> u32 {
    let mut utf16_count = 0u32;
    let mut utf8_offset = 0u32;

    for ch in line_text.chars() {
        if utf16_count >= utf16_column {
            break;
        }
        utf16_count += char_utf16_len(ch);
        utf8_offset += ch.len_utf8() as u32;
    }

    utf8_offset
}

/// Get the UTF-16 length of a character.
#[inline]
fn char_utf16_len(ch: char) -> u32 {
    if ch as u32 > 0xFFFF { 2 } else { 1 }
}

/// Convert a Span to an LSP Range, converting UTF-8 columns to UTF-16.
///
/// # Arguments
/// * `span` - The source span with UTF-8 positions
/// * `source` - The full source text for line extraction
///
/// # Returns
/// LSP Range with UTF-16 character positions
pub fn span_to_range(span: &Span, source: &str) -> Range {
    let lines: Vec<&str> = source.lines().collect();

    let start_col = if (span.start.line as usize) < lines.len() {
        utf8_to_utf16_col(lines[span.start.line as usize], span.start.column)
    } else {
        span.start.column
    };

    let end_col = if (span.end.line as usize) < lines.len() {
        utf8_to_utf16_col(lines[span.end.line as usize], span.end.column)
    } else {
        span.end.column
    };

    Range {
        start: Position {
            line: span.start.line,
            character: start_col,
        },
        end: Position {
            line: span.end.line,
            character: end_col,
        },
    }
}

/// Convert an LSP Position to a UTF-8 byte offset within a line.
///
/// # Arguments
/// * `line_text` - The text of the line
/// * `position` - LSP position with UTF-16 character offset
///
/// # Returns
/// Column in UTF-8 bytes
pub fn position_to_utf8_col(line_text: &str, position: &Position) -> u32 {
    utf16_to_utf8_col(line_text, position.character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_conversion() {
        let line = "hello world";
        assert_eq!(utf8_to_utf16_col(line, 5), 5);
        assert_eq!(utf16_to_utf8_col(line, 5), 5);
    }

    #[test]
    fn test_multibyte_utf8() {
        // "hello" + emoji (4 UTF-8 bytes, 2 UTF-16 units) + "world"
        let line = "hello\u{1F600}world"; // 1F600 is a grinning face emoji

        // Position after "hello" (5 UTF-8 bytes, 5 UTF-16 units)
        assert_eq!(utf8_to_utf16_col(line, 5), 5);

        // Position after emoji (5 + 4 = 9 UTF-8 bytes, 5 + 2 = 7 UTF-16 units)
        assert_eq!(utf8_to_utf16_col(line, 9), 7);

        // Reverse conversion
        assert_eq!(utf16_to_utf8_col(line, 5), 5);
        assert_eq!(utf16_to_utf8_col(line, 7), 9);
    }

    #[test]
    fn test_bmp_characters() {
        // Characters in BMP (1 UTF-16 unit each)
        let line = "cafe\u{0301}"; // "cafe" + combining acute accent
        assert_eq!(utf8_to_utf16_col(line, 4), 4);
        // The combining accent is 2 UTF-8 bytes but 1 UTF-16 unit
        assert_eq!(utf8_to_utf16_col(line, 6), 5);
    }

    #[test]
    fn test_empty_line() {
        let line = "";
        assert_eq!(utf8_to_utf16_col(line, 0), 0);
        assert_eq!(utf16_to_utf8_col(line, 0), 0);
    }

    #[test]
    fn test_column_beyond_line() {
        let line = "short";
        // Column beyond line end returns total char count
        assert_eq!(utf8_to_utf16_col(line, 100), 5);
    }
}
