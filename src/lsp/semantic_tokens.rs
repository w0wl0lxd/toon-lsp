// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024-2025 w0wl0lxd
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Semantic token types and modifiers for TOON language server.
//!
//! This module defines the token types and modifiers used for semantic
//! highlighting in the LSP. These map to the semantic token legend declared
//! in the server capabilities.
//!
//! # Token Types
//! - Property: Object keys
//! - String: String values
//! - Number: Numeric values
//! - Keyword: true, false, null
//! - Operator: colons, brackets
//!
//! # Token Modifiers
//! - DEFINITION: First occurrence of a key
//! - READONLY: Immutable literals

use crate::ast::Span;

/// Token type for semantic highlighting.
///
/// Each variant corresponds to an index in the semantic token legend
/// declared in server capabilities. The order and indices MUST match
/// exactly with the legend array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ToonTokenType {
    /// Object keys
    Property = 0,
    /// String values
    String = 1,
    /// Numeric values
    Number = 2,
    /// Boolean and null keywords (true, false, null)
    Keyword = 3,
    /// Operators (colons, brackets)
    Operator = 4,
}

impl ToonTokenType {
    /// Convert token type to u32 index for LSP protocol.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Convert `ToonTokenType` to `u32` for LSP protocol.
///
/// This conversion is used when encoding tokens for transmission
/// to the LSP client. The u32 value corresponds to an index in the
/// semantic token legend declared in server capabilities.
impl From<ToonTokenType> for u32 {
    fn from(token_type: ToonTokenType) -> Self {
        token_type.as_u32()
    }
}

bitflags::bitflags! {
    /// Token modifiers for semantic highlighting.
    ///
    /// Modifiers can be combined using bitwise operations to indicate
    /// multiple properties of a token.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ToonTokenModifier: u32 {
        /// First occurrence of a key (definition site)
        const DEFINITION = 1 << 0;
        /// Immutable literal value
        const READONLY = 1 << 1;
    }
}

impl ToonTokenModifier {
    /// Convert modifiers to u32 bitmask for LSP protocol.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.bits()
    }
}

/// A semantic token with position and classification.
///
/// Used to provide syntax highlighting information to the LSP client.
/// Positions are 0-based line/column in UTF-8 encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticToken {
    /// 0-based line number
    pub line: u32,
    /// 0-based UTF-8 column
    pub start_col: u32,
    /// Token length in UTF-8 bytes
    pub length: u32,
    /// Token classification
    pub token_type: ToonTokenType,
    /// Token modifiers (combined bitflags)
    pub modifiers: ToonTokenModifier,
}

impl SemanticToken {
    /// Create a new semantic token.
    #[must_use]
    pub const fn new(
        line: u32,
        start_col: u32,
        length: u32,
        token_type: ToonTokenType,
        modifiers: ToonTokenModifier,
    ) -> Self {
        Self {
            line,
            start_col,
            length,
            token_type,
            modifiers,
        }
    }

    /// Create a semantic token from a span.
    #[must_use]
    pub fn from_span(span: &Span, token_type: ToonTokenType, modifiers: ToonTokenModifier) -> Self {
        let length = span.end.offset - span.start.offset;
        Self {
            line: span.start.line,
            start_col: span.start.column,
            length,
            token_type,
            modifiers,
        }
    }

    /// Check if this token has a specific modifier.
    #[must_use]
    pub const fn has_modifier(&self, modifier: ToonTokenModifier) -> bool {
        self.modifiers.contains(modifier)
    }
}

/// Collect semantic tokens from an AST.
///
/// Traverses the AST and generates semantic tokens for all syntax elements
/// including object keys, values, and literals. Tokens contain position
/// information and classification for syntax highlighting.
///
/// # Arguments
/// * `ast` - The root AST node to traverse
/// * `text` - The source text (used for position calculations)
///
/// # Returns
/// Vector of semantic tokens in document order (top-to-bottom, left-to-right)
#[must_use]
pub fn collect_semantic_tokens(ast: &crate::ast::AstNode) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    visit_node(ast, &mut tokens);
    tokens
}

/// Recursively visit AST nodes and collect semantic tokens.
///
/// Traverses the AST in depth-first order, generating tokens for all
/// syntax elements. Each node type contributes different token types:
/// - Object entries: Property tokens for keys with DEFINITION modifier
/// - String/Number/Bool/Null: Literal tokens with READONLY modifier
/// - Arrays and Documents: Recursively process children
///
/// # Arguments
///
/// * `node` - The AST node to visit
/// * `tokens` - Accumulator vector for collected tokens
///
/// # Examples
///
/// ```ignore
/// let mut tokens = Vec::new();
/// visit_node(&ast, &mut tokens);
/// // tokens now contains all semantic tokens from ast
/// ```
fn visit_node(node: &crate::ast::AstNode, tokens: &mut Vec<SemanticToken>) {
    use crate::ast::AstNode;

    match node {
        AstNode::Document { children, .. } => {
            for child in children {
                visit_node(child, tokens);
            }
        }
        AstNode::Object { entries, .. } => {
            for entry in entries {
                // Add Property token for key with DEFINITION modifier
                tokens.push(SemanticToken::from_span(
                    &entry.key_span,
                    ToonTokenType::Property,
                    ToonTokenModifier::DEFINITION,
                ));
                // Visit value
                visit_node(&entry.value, tokens);
            }
        }
        AstNode::Array { items, .. } => {
            for item in items {
                visit_node(item, tokens);
            }
        }
        AstNode::String { span, .. } => {
            tokens.push(SemanticToken::from_span(
                span,
                ToonTokenType::String,
                ToonTokenModifier::READONLY,
            ));
        }
        AstNode::Number { span, .. } => {
            tokens.push(SemanticToken::from_span(
                span,
                ToonTokenType::Number,
                ToonTokenModifier::READONLY,
            ));
        }
        AstNode::Bool { span, .. } => {
            tokens.push(SemanticToken::from_span(
                span,
                ToonTokenType::Keyword,
                ToonTokenModifier::READONLY,
            ));
        }
        AstNode::Null { span } => {
            tokens.push(SemanticToken::from_span(
                span,
                ToonTokenType::Keyword,
                ToonTokenModifier::READONLY,
            ));
        }
    }
}

/// Encode semantic tokens into LSP relative format.
///
/// Converts absolute position tokens into the LSP delta-encoded format.
/// Each token is represented as a tower_lsp::lsp_types::SemanticToken with:
/// - delta_line: Line offset from previous token (or 0 for first token)
/// - delta_start: If same line, offset from previous token's start; if different line, absolute column
/// - length: Token length in UTF-16 code units
/// - token_type: Index into SemanticTokensLegend.tokenTypes
/// - token_modifiers_bitset: Bitmask of modifier indices
///
/// The encoding uses relative positions for efficiency:
/// - First token: deltaLine and deltaStartChar from document start
/// - Same line tokens: deltaLine=0, deltaStartChar=offset from previous token
/// - New line tokens: deltaLine=line difference, deltaStartChar=absolute column
///
/// # Arguments
/// * `tokens` - Semantic tokens with absolute positions
/// * `text` - The source text (used for UTF-16 conversion)
///
/// # Returns
/// Vector of LSP SemanticToken structs in delta-encoded format
#[must_use]
pub fn encode_tokens(
    tokens: &[SemanticToken],
    text: &str,
) -> Vec<tower_lsp::lsp_types::SemanticToken> {
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;

    // Split text into lines for UTF-16 conversion
    let lines: Vec<&str> = text.lines().collect();

    // Note: tokens are already in document order from depth-first AST traversal
    for token in tokens {
        let delta_line = token.line - prev_line;

        // Convert UTF-8 positions to UTF-16 for LSP compliance
        let line_idx = token.line as usize;
        let line_text = if line_idx < lines.len() {
            lines[line_idx]
        } else {
            ""
        };

        let utf16_col = super::utf16::utf8_to_utf16_col(line_text, token.start_col);
        let utf16_length =
            super::utf16::utf8_to_utf16_col(line_text, token.start_col + token.length) - utf16_col;

        let utf16_delta_col = if delta_line == 0 {
            // Same line: compute delta in UTF-16 space
            let prev_utf16_col = super::utf16::utf8_to_utf16_col(line_text, prev_col);
            utf16_col - prev_utf16_col
        } else {
            // New line: use absolute UTF-16 column
            utf16_col
        };

        result.push(tower_lsp::lsp_types::SemanticToken {
            delta_line,
            delta_start: utf16_delta_col,
            length: utf16_length,
            token_type: token.token_type.as_u32(),
            token_modifiers_bitset: token.modifiers.bits(),
        });

        prev_line = token.line;
        prev_col = token.start_col;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Position;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_token_type_indices() {
        // Verify token type indices match LSP legend order
        assert_eq!(ToonTokenType::Property.as_u32(), 0);
        assert_eq!(ToonTokenType::String.as_u32(), 1);
        assert_eq!(ToonTokenType::Number.as_u32(), 2);
        assert_eq!(ToonTokenType::Keyword.as_u32(), 3);
        assert_eq!(ToonTokenType::Operator.as_u32(), 4);
    }

    #[test]
    fn test_token_type_conversion() {
        // Test From<ToonTokenType> for u32
        let property: u32 = ToonTokenType::Property.into();
        assert_eq!(property, 0);

        let keyword: u32 = ToonTokenType::Keyword.into();
        assert_eq!(keyword, 3);
    }

    #[test]
    fn test_token_modifier_definition() {
        // Test DEFINITION modifier
        let def = ToonTokenModifier::DEFINITION;
        assert_eq!(def.bits(), 1);
        assert_eq!(def.as_u32(), 1);
    }

    #[test]
    fn test_token_modifier_readonly() {
        // Test READONLY modifier
        let readonly = ToonTokenModifier::READONLY;
        assert_eq!(readonly.bits(), 2);
        assert_eq!(readonly.as_u32(), 2);
    }

    #[test]
    fn test_token_modifier_combined() {
        // Test combining modifiers with bitwise OR
        let combined = ToonTokenModifier::DEFINITION | ToonTokenModifier::READONLY;
        assert_eq!(combined.bits(), 3);
        assert_eq!(combined.as_u32(), 3);

        // Test contains
        assert!(combined.contains(ToonTokenModifier::DEFINITION));
        assert!(combined.contains(ToonTokenModifier::READONLY));
    }

    #[test]
    fn test_token_modifier_empty() {
        // Test empty modifiers
        let empty = ToonTokenModifier::empty();
        assert_eq!(empty.bits(), 0);
        assert_eq!(empty.as_u32(), 0);
        assert!(!empty.contains(ToonTokenModifier::DEFINITION));
    }

    #[test]
    fn test_semantic_token_new() {
        // Test creating a semantic token
        let token = SemanticToken::new(
            5,
            10,
            4,
            ToonTokenType::Property,
            ToonTokenModifier::DEFINITION,
        );

        assert_eq!(token.line, 5);
        assert_eq!(token.start_col, 10);
        assert_eq!(token.length, 4);
        assert_eq!(token.token_type, ToonTokenType::Property);
        assert_eq!(token.modifiers, ToonTokenModifier::DEFINITION);
    }

    #[test]
    fn test_semantic_token_from_span() {
        // Test creating semantic token from span
        let start = Position::new(2, 5, 25);
        let end = Position::new(2, 9, 29);
        let span = Span::new(start, end);

        let token =
            SemanticToken::from_span(&span, ToonTokenType::String, ToonTokenModifier::READONLY);

        assert_eq!(token.line, 2);
        assert_eq!(token.start_col, 5);
        assert_eq!(token.length, 4); // 29 - 25
        assert_eq!(token.token_type, ToonTokenType::String);
        assert_eq!(token.modifiers, ToonTokenModifier::READONLY);
    }

    #[test]
    fn test_semantic_token_has_modifier() {
        // Test modifier checking
        let token = SemanticToken::new(
            0,
            0,
            5,
            ToonTokenType::Property,
            ToonTokenModifier::DEFINITION | ToonTokenModifier::READONLY,
        );

        assert!(token.has_modifier(ToonTokenModifier::DEFINITION));
        assert!(token.has_modifier(ToonTokenModifier::READONLY));
    }

    #[test]
    fn test_semantic_token_no_modifier() {
        // Test token without specific modifier
        let token = SemanticToken::new(0, 0, 5, ToonTokenType::Number, ToonTokenModifier::empty());

        assert!(!token.has_modifier(ToonTokenModifier::DEFINITION));
        assert!(!token.has_modifier(ToonTokenModifier::READONLY));
    }

    // ===================================================================
    // TDD RED PHASE: Semantic Token Collection Tests (T010-T017)
    // These tests are EXPECTED to FAIL until implementation in T018-T023
    // ===================================================================

    use crate::parser::parse_with_errors;

    /// T010: Test semantic tokens for object keys
    ///
    /// Validates that object keys are correctly identified as Property tokens.
    /// Should detect all keys at the top level of an object.
    #[test]
    fn test_semantic_tokens_object_keys() {
        let source = "name: Alice\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse valid TOON");

        let tokens = collect_semantic_tokens(&ast);

        // Should find 2 Property tokens: "name" and "age"
        let properties: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == ToonTokenType::Property)
            .collect();

        assert_eq!(
            properties.len(),
            2,
            "Expected 2 property tokens (name, age), found {}",
            properties.len()
        );

        // Verify first property token
        assert_eq!(properties[0].line, 0, "name should be on line 0");
        assert_eq!(properties[0].start_col, 0, "name should start at column 0");
        assert_eq!(properties[0].length, 4, "name is 4 characters");

        // Verify second property token
        assert_eq!(properties[1].line, 1, "age should be on line 1");
        assert_eq!(properties[1].start_col, 0, "age should start at column 0");
        assert_eq!(properties[1].length, 3, "age is 3 characters");
    }

    /// T011: Test semantic tokens for string values
    ///
    /// Validates that string literals are correctly classified as String tokens
    /// with READONLY modifier (immutable literals).
    #[test]
    fn test_semantic_tokens_string_values() {
        let source = "name: Alice\nmessage: Hello World";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse valid TOON");

        let tokens = collect_semantic_tokens(&ast);

        // Should find 2 String tokens: "Alice" and "Hello World"
        let strings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == ToonTokenType::String)
            .collect();

        assert_eq!(
            strings.len(),
            2,
            "Expected 2 string tokens (Alice, Hello World), found {}",
            strings.len()
        );

        // Verify first string
        assert_eq!(strings[0].line, 0, "Alice should be on line 0");
        assert_eq!(strings[0].length, 5, "Alice is 5 characters");
        assert!(
            strings[0].has_modifier(ToonTokenModifier::READONLY),
            "String literals should be marked READONLY"
        );

        // Verify second string
        assert_eq!(strings[1].line, 1, "Hello World should be on line 1");
        assert_eq!(strings[1].length, 11, "Hello World is 11 characters");
        assert!(
            strings[1].has_modifier(ToonTokenModifier::READONLY),
            "String literals should be marked READONLY"
        );
    }

    /// T012: Test semantic tokens for number values
    ///
    /// Validates that numeric literals (integer, float, negative) are correctly
    /// classified as Number tokens with READONLY modifier.
    #[test]
    fn test_semantic_tokens_number_values() {
        let source = "count: 42\nprice: 19.99\nnegative: -5";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse valid TOON");

        let tokens = collect_semantic_tokens(&ast);

        // Should find 3 Number tokens: 42, 19.99, -5
        let numbers: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == ToonTokenType::Number)
            .collect();

        assert_eq!(
            numbers.len(),
            3,
            "Expected 3 number tokens (42, 19.99, -5), found {}",
            numbers.len()
        );

        // Verify integer
        assert_eq!(numbers[0].line, 0, "42 should be on line 0");
        assert_eq!(numbers[0].length, 2, "42 is 2 characters");
        assert!(
            numbers[0].has_modifier(ToonTokenModifier::READONLY),
            "Number literals should be marked READONLY"
        );

        // Verify float
        assert_eq!(numbers[1].line, 1, "19.99 should be on line 1");
        assert_eq!(numbers[1].length, 5, "19.99 is 5 characters");

        // Verify negative
        assert_eq!(numbers[2].line, 2, "-5 should be on line 2");
        assert_eq!(numbers[2].length, 2, "-5 is 2 characters");
    }

    /// T013: Test semantic tokens for boolean and null keywords
    ///
    /// Validates that true, false, and null are correctly classified as
    /// Keyword tokens with READONLY modifier.
    #[test]
    fn test_semantic_tokens_boolean_null() {
        let source = "active: true\ndeleted: false\ndata: null";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse valid TOON");

        let tokens = collect_semantic_tokens(&ast);

        // Should find 3 Keyword tokens: true, false, null
        let keywords: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == ToonTokenType::Keyword)
            .collect();

        assert_eq!(
            keywords.len(),
            3,
            "Expected 3 keyword tokens (true, false, null), found {}",
            keywords.len()
        );

        // Verify true keyword
        assert_eq!(keywords[0].line, 0, "true should be on line 0");
        assert_eq!(keywords[0].length, 4, "true is 4 characters");
        assert!(
            keywords[0].has_modifier(ToonTokenModifier::READONLY),
            "Keywords should be marked READONLY"
        );

        // Verify false keyword
        assert_eq!(keywords[1].line, 1, "false should be on line 1");
        assert_eq!(keywords[1].length, 5, "false is 5 characters");
        assert!(
            keywords[1].has_modifier(ToonTokenModifier::READONLY),
            "Keywords should be marked READONLY"
        );

        // Verify null keyword
        assert_eq!(keywords[2].line, 2, "null should be on line 2");
        assert_eq!(keywords[2].length, 4, "null is 4 characters");
        assert!(
            keywords[2].has_modifier(ToonTokenModifier::READONLY),
            "Keywords should be marked READONLY"
        );
    }

    /// T014: Test semantic tokens for nested objects and arrays
    ///
    /// Validates that tokens are correctly collected from nested structures,
    /// including keys at different indentation levels and values within arrays.
    #[test]
    fn test_semantic_tokens_nested() {
        let source = "user:\n  name: Bob\n  tags:\n    - admin\n    - user";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse valid TOON");

        let tokens = collect_semantic_tokens(&ast);

        // Should find 3 property tokens: "user", "name", "tags"
        let properties: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == ToonTokenType::Property)
            .collect();

        assert_eq!(
            properties.len(),
            3,
            "Expected 3 property tokens (user, name, tags), found {}",
            properties.len()
        );

        // Should find 3 string tokens: "Bob", "admin", "user"
        let strings: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == ToonTokenType::String)
            .collect();

        assert_eq!(
            strings.len(),
            3,
            "Expected 3 string tokens (Bob, admin, user), found {}",
            strings.len()
        );

        // Verify nested property positions
        assert_eq!(properties[0].line, 0, "user should be on line 0");
        assert_eq!(properties[1].line, 1, "name should be on line 1");
        assert_eq!(
            properties[1].start_col, 2,
            "name should be indented 2 spaces"
        );
        assert_eq!(properties[2].line, 2, "tags should be on line 2");
        assert_eq!(
            properties[2].start_col, 2,
            "tags should be indented 2 spaces"
        );
    }

    /// T015: Test encode_tokens produces correct relative positions
    ///
    /// Validates the LSP delta-encoding algorithm:
    /// - delta_line, delta_start, length, token_type, token_modifiers_bitset
    /// - Same line: delta_line=0, delta_start=offset from previous
    /// - New line: delta_line=difference, delta_start=absolute (reset)
    #[test]
    fn test_encode_tokens_relative_positions() {
        let source = "foo: bar\nbaz: 123";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse valid TOON");

        let tokens = collect_semantic_tokens(&ast);
        let encoded = encode_tokens(&tokens, source);

        // Should have at least 4 tokens: "foo", "bar", "baz", "123"
        assert!(
            encoded.len() >= 4,
            "Should have at least 4 tokens (foo, bar, baz, 123)"
        );

        // First token: "foo" at line 0, col 0
        assert_eq!(
            encoded[0].delta_line, 0,
            "First token delta_line should be 0"
        );
        assert_eq!(
            encoded[0].delta_start, 0,
            "First token delta_start should be 0"
        );

        // Second token on same line: "bar" at col 5
        if encoded.len() >= 2 {
            assert_eq!(
                encoded[1].delta_line, 0,
                "Same line token delta_line should be 0"
            );
            assert!(
                encoded[1].delta_start > 0,
                "Same line token delta_start should be offset from previous"
            );
        }

        // Third token on new line: "baz" at line 1, col 0
        if encoded.len() >= 3 {
            assert_eq!(
                encoded[2].delta_line, 1,
                "New line token delta_line should be 1"
            );
            assert_eq!(
                encoded[2].delta_start, 0,
                "New line token delta_start resets to absolute position"
            );
        }
    }

    /// T016: Test semantic tokens handles documents with parse errors
    ///
    /// Validates graceful degradation - even with parse errors, should
    /// return tokens for successfully parsed portions of the document.
    /// Uses unclosed bracket to force parse error.
    #[test]
    fn test_semantic_tokens_with_parse_errors() {
        // Use unclosed array bracket to force parse error
        let source = "valid: ok\narray: [\nmore: stuff";
        let (ast, errors) = parse_with_errors(source);

        // If we got an AST despite errors, should still tokenize valid parts
        // This tests graceful degradation
        if let Some(ast) = ast {
            let _tokens = collect_semantic_tokens(&ast);

            // Should tokenize without panicking on parse errors
            // (No assertion needed - test passes if no panic occurs)
        } else {
            // If parser returned no AST due to errors, that's also acceptable
            // The test validates we don't panic on error cases
            assert!(
                !errors.is_empty(),
                "If no AST returned, should have parse errors"
            );
        }
    }

    /// T017: Test semantic tokens handles empty documents
    ///
    /// Validates edge case: empty document should produce empty token list
    /// without panicking or returning errors.
    #[test]
    fn test_semantic_tokens_empty_document() {
        let source = "";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("Empty document should parse successfully");

        let tokens = collect_semantic_tokens(&ast);

        assert!(
            tokens.is_empty(),
            "Empty document should produce no tokens, found {}",
            tokens.len()
        );
    }

    /// T017b: Test semantic tokens handles whitespace-only documents
    ///
    /// Validates edge case: document with only whitespace/comments should
    /// produce empty token list.
    #[test]
    fn test_semantic_tokens_whitespace_only() {
        let source = "   \n  \n\t\n";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("Whitespace-only document should parse");

        let tokens = collect_semantic_tokens(&ast);

        assert!(
            tokens.is_empty(),
            "Whitespace-only document should produce no tokens, found {}",
            tokens.len()
        );
    }
}
