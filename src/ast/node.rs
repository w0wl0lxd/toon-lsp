//! AST node types for TOON documents.

use super::Span;
use serde::{Deserialize, Serialize};

/// Array presentation form in the source document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrayForm {
    /// Inline form: [item1, item2, item3]
    Inline,
    /// Expanded form with dashes:
    /// - item1
    /// - item2
    Expanded,
    /// Tabular form with pipes:
    /// | col1 | col2 |
    /// | val1 | val2 |
    Tabular,
}

/// An AST node in a TOON document.
///
/// Each variant carries its span (source location) for error reporting
/// and LSP features like go-to-definition and hover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    /// A TOON document (root node)
    Document {
        /// Child nodes
        children: Vec<AstNode>,
        /// Source span
        span: Span,
    },

    /// An object/table (key-value pairs)
    Object {
        /// Object entries
        entries: Vec<ObjectEntry>,
        /// Source span
        span: Span,
    },

    /// An array
    Array {
        /// Array items
        items: Vec<AstNode>,
        /// Array presentation form
        form: ArrayForm,
        /// Source span
        span: Span,
    },

    /// A string value
    String {
        /// The string content
        value: String,
        /// Source span
        span: Span,
    },

    /// A number value
    Number {
        /// The numeric value
        value: NumberValue,
        /// Source span
        span: Span,
    },

    /// A boolean value
    Bool {
        /// The boolean value
        value: bool,
        /// Source span
        span: Span,
    },

    /// A null value
    Null {
        /// Source span
        span: Span,
    },
}

impl AstNode {
    /// Get the span of this node.
    pub fn span(&self) -> Span {
        match self {
            AstNode::Document { span, .. } => *span,
            AstNode::Object { span, .. } => *span,
            AstNode::Array { span, .. } => *span,
            AstNode::String { span, .. } => *span,
            AstNode::Number { span, .. } => *span,
            AstNode::Bool { span, .. } => *span,
            AstNode::Null { span } => *span,
        }
    }

    /// Get the kind of this node as a string.
    pub fn kind(&self) -> &'static str {
        match self {
            AstNode::Document { .. } => "document",
            AstNode::Object { .. } => "object",
            AstNode::Array { .. } => "array",
            AstNode::String { .. } => "string",
            AstNode::Number { .. } => "number",
            AstNode::Bool { .. } => "bool",
            AstNode::Null { .. } => "null",
        }
    }
}

/// An entry in a TOON object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectEntry {
    /// The key
    pub key: String,
    /// Span of the key
    pub key_span: Span,
    /// The value
    pub value: AstNode,
}

/// A numeric value in TOON.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NumberValue {
    /// Positive integer
    PosInt(u64),
    /// Negative integer
    NegInt(i64),
    /// Floating point
    Float(f64),
}

impl NumberValue {
    /// Convert to f64.
    pub fn as_f64(&self) -> f64 {
        match self {
            NumberValue::PosInt(n) => *n as f64,
            NumberValue::NegInt(n) => *n as f64,
            NumberValue::Float(n) => *n,
        }
    }
}

// AST traversal is provided via lsp::ast_utils module:
// - find_node_at_position() for cursor-based node lookup
// - find_all_key_references() for key occurrence finding
// - flatten_ast() for collecting all document symbols
