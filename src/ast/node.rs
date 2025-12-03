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
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Document { span, .. }
            | Self::Object { span, .. }
            | Self::Array { span, .. }
            | Self::String { span, .. }
            | Self::Number { span, .. }
            | Self::Bool { span, .. }
            | Self::Null { span } => *span,
        }
    }

    /// Get the kind of this node as a string.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Document { .. } => "document",
            Self::Object { .. } => "object",
            Self::Array { .. } => "array",
            Self::String { .. } => "string",
            Self::Number { .. } => "number",
            Self::Bool { .. } => "bool",
            Self::Null { .. } => "null",
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
    /// Convert to f64 (lossy for large integers).
    ///
    /// # Precision Warning
    /// For integers with absolute value > 2^53 (9,007,199,254,740,992),
    /// precision may be lost during conversion. This affects very large
    /// IDs, timestamps in nanoseconds, or cryptographic values.
    ///
    /// Use [`as_u64`] or [`as_i64`] for exact integer access when precision matters.
    #[allow(clippy::cast_precision_loss)]
    pub fn as_f64_lossy(&self) -> f64 {
        match self {
            NumberValue::PosInt(n) => *n as f64,
            NumberValue::NegInt(n) => *n as f64,
            NumberValue::Float(n) => *n,
        }
    }

    /// Get as u64 if this is a positive integer.
    #[must_use]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            NumberValue::PosInt(n) => Some(*n),
            NumberValue::NegInt(_) | NumberValue::Float(_) => None,
        }
    }

    /// Get as i64 if this is an integer (positive or negative).
    ///
    /// Returns `None` for positive integers that exceed `i64::MAX`.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // Safe: checked that n <= i64::MAX before cast
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            NumberValue::PosInt(n) => {
                if *n <= i64::MAX as u64 {
                    Some(*n as i64)
                } else {
                    None
                }
            }
            NumberValue::NegInt(n) => Some(*n),
            NumberValue::Float(_) => None,
        }
    }
}

// AST traversal is provided via lsp::ast_utils module:
// - find_node_at_position() for cursor-based node lookup
// - find_all_key_references() for key occurrence finding
// - flatten_ast() for collecting all document symbols
