//! TOON parser that produces AST with source positions.
//!
//! This module provides the core parsing functionality:
//! - Scanner (lexer) for tokenizing TOON input
//! - Parser for building AST from tokens
//! - Error types with position information

mod scanner;
mod error;

pub use error::ParseError;

use crate::ast::AstNode;

/// Parse TOON source into an AST.
///
/// # Arguments
/// * `source` - The TOON source text to parse
///
/// # Returns
/// * `Ok(AstNode)` - The root AST node on success
/// * `Err(ParseError)` - Parse error with position information
///
/// # Example
/// ```rust,ignore
/// use toon_lsp::parse;
///
/// let ast = parse("name: Alice\nage: 30")?;
/// ```
pub fn parse(source: &str) -> Result<AstNode, ParseError> {
    // TODO: Implement parser based on toon-rust scanner/parser architecture
    // Reference implementation: https://github.com/toon-format/toon-rust/tree/main/src/decode
    let _ = source;
    todo!("Implement TOON parser")
}

/// Parse TOON source and collect all errors (for IDE use).
///
/// Unlike `parse()`, this function attempts to recover from errors
/// and returns as much of the AST as possible along with all errors.
pub fn parse_with_errors(source: &str) -> (Option<AstNode>, Vec<ParseError>) {
    // TODO: Implement error-recovering parser
    let _ = source;
    todo!("Implement error-recovering parser")
}
