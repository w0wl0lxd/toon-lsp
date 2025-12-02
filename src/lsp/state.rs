//! Document state management for the LSP server.
//!
//! This module provides the `DocumentState` struct that tracks open documents,
//! their parsed AST, and any parse errors.

use crate::ast::AstNode;
use crate::parser::{ParseError, parse_with_errors};

/// Represents an open TOON document tracked by the language server.
///
/// Each `DocumentState` maintains:
/// - The current document text
/// - The LSP document version
/// - The parsed AST (if parsing succeeded partially or fully)
/// - Any parse errors from the last parse
///
/// The state is updated synchronously on document changes, keeping the
/// AST and errors always in sync with the text.
#[derive(Debug, Clone)]
pub struct DocumentState {
    /// Current document content (UTF-8)
    text: String,
    /// LSP document version (incremented on each change)
    version: i32,
    /// Parsed AST (None only if parsing failed completely)
    ast: Option<AstNode>,
    /// Parse errors from the last parse
    errors: Vec<ParseError>,
}

impl DocumentState {
    /// Create a new document state by parsing the given text.
    ///
    /// # Arguments
    /// * `text` - The document content
    /// * `version` - The LSP document version
    ///
    /// # Returns
    /// A new `DocumentState` with parsed AST and any errors
    pub fn new(text: String, version: i32) -> Self {
        let (ast, errors) = parse_with_errors(&text);
        Self {
            text,
            version,
            ast,
            errors,
        }
    }

    /// Update the document with new text and version.
    ///
    /// Re-parses the document synchronously and updates the AST and errors.
    ///
    /// # Arguments
    /// * `text` - The new document content
    /// * `version` - The new LSP document version
    pub fn update(&mut self, text: String, version: i32) {
        let (ast, errors) = parse_with_errors(&text);
        self.text = text;
        self.version = version;
        self.ast = ast;
        self.errors = errors;
    }

    /// Get the current document text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the LSP document version.
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Get the parsed AST, if available.
    pub fn ast(&self) -> Option<&AstNode> {
        self.ast.as_ref()
    }

    /// Get the parse errors from the last parse.
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Check if the document has any parse errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get lines of the document for position conversion.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.text.lines()
    }

    /// Get a specific line by 0-indexed line number.
    pub fn get_line(&self, line: u32) -> Option<&str> {
        self.text.lines().nth(line as usize)
    }

    /// Convert LSP UTF-16 position to UTF-8 column for internal use.
    ///
    /// LSP uses UTF-16 column offsets, but our internal AST uses UTF-8.
    /// This helper centralizes the conversion logic that previously appeared
    /// in multiple LSP handlers.
    ///
    /// # Arguments
    /// * `line` - The 0-indexed line number
    /// * `utf16_col` - The UTF-16 column offset (as received from LSP)
    ///
    /// # Returns
    /// The UTF-8 column offset for the given position
    pub fn utf8_col_at(&self, line: u32, utf16_col: u32) -> u32 {
        let line_text = self.get_line(line).unwrap_or("");
        crate::lsp::utf16::utf16_to_utf8_col(line_text, utf16_col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_valid_state() {
        let state = DocumentState::new("key: value".to_string(), 1);
        assert_eq!(state.version(), 1);
        assert!(state.ast().is_some());
        assert!(!state.has_errors());
    }

    #[test]
    fn test_update_changes_content() {
        let mut state = DocumentState::new("old: value".to_string(), 1);
        state.update("new: value".to_string(), 2);
        assert_eq!(state.text(), "new: value");
        assert_eq!(state.version(), 2);
    }

    #[test]
    fn test_get_line() {
        let state = DocumentState::new("line0\nline1\nline2".to_string(), 1);
        assert_eq!(state.get_line(0), Some("line0"));
        assert_eq!(state.get_line(1), Some("line1"));
        assert_eq!(state.get_line(2), Some("line2"));
        assert_eq!(state.get_line(3), None);
    }
}
