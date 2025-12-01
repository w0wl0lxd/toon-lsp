//! Abstract Syntax Tree types for TOON with source position tracking.
//!
//! This module provides AST node types that preserve source locations (spans)
//! for error reporting, syntax highlighting, and IDE features.

mod node;
mod span;

pub use node::AstNode;
pub use span::{Position, Span};

// TODO: Implement AST types based on TOON spec
// Reference: https://github.com/toon-format/spec/blob/main/SPEC.md
