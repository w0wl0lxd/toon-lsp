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
    ///
    /// # Example
    /// ```rust
    /// use toon_lsp::{parse, AstNode};
    ///
    /// let ast = parse("name: Alice").unwrap();
    /// if let AstNode::Document { children, .. } = ast {
    ///     assert!(!children.is_empty());
    /// }
    /// ```
    #[inline]
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
    ///
    /// # Example
    /// ```rust
    /// use toon_lsp::{parse, AstNode};
    ///
    /// let ast = parse("name: Alice").unwrap();
    /// if let AstNode::Document { children, .. } = &ast {
    ///     if let Some(first) = children.first() {
    ///         assert_eq!(first.kind(), "object");
    ///     }
    /// }
    /// ```
    #[inline]
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
    /// Convert to f64.
    ///
    /// # Example
    /// ```rust
    /// use toon_lsp::NumberValue;
    ///
    /// assert_eq!(NumberValue::PosInt(42).as_f64(), 42.0);
    /// assert_eq!(NumberValue::NegInt(-10).as_f64(), -10.0);
    /// assert_eq!(NumberValue::Float(3.14).as_f64(), 3.14);
    /// ```
    #[inline]
    pub fn as_f64(self) -> f64 {
        match self {
            Self::PosInt(n) => n as f64,
            Self::NegInt(n) => n as f64,
            Self::Float(n) => n,
        }
    }
}

// AST traversal is provided via lsp::ast_utils module:
// - find_node_at_position() for cursor-based node lookup
// - find_all_key_references() for key occurrence finding
// - flatten_ast() for collecting all document symbols

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    fn test_span() -> Span {
        Span::new(Position::new(0, 0, 0), Position::new(0, 5, 5))
    }

    fn test_span_2() -> Span {
        Span::new(Position::new(1, 0, 10), Position::new(1, 10, 20))
    }

    #[test]
    fn test_ast_node_kind_document() {
        let node = AstNode::Document { children: vec![], span: test_span() };
        assert_eq!(node.kind(), "document");
    }

    #[test]
    fn test_ast_node_kind_object() {
        let node = AstNode::Object { entries: vec![], span: test_span() };
        assert_eq!(node.kind(), "object");
    }

    #[test]
    fn test_ast_node_kind_array() {
        let node = AstNode::Array { items: vec![], form: ArrayForm::Inline, span: test_span() };
        assert_eq!(node.kind(), "array");
    }

    #[test]
    fn test_ast_node_kind_string() {
        let node = AstNode::String { value: String::new(), span: test_span() };
        assert_eq!(node.kind(), "string");
    }

    #[test]
    fn test_ast_node_kind_number() {
        let node = AstNode::Number { value: NumberValue::PosInt(42), span: test_span() };
        assert_eq!(node.kind(), "number");
    }

    #[test]
    fn test_ast_node_kind_bool() {
        let node = AstNode::Bool { value: true, span: test_span() };
        assert_eq!(node.kind(), "bool");
    }

    #[test]
    fn test_ast_node_kind_null() {
        let node = AstNode::Null { span: test_span() };
        assert_eq!(node.kind(), "null");
    }

    #[test]
    fn test_ast_node_span_document() {
        let node = AstNode::Document { children: vec![], span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_ast_node_span_object() {
        let node = AstNode::Object { entries: vec![], span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_ast_node_span_array() {
        let node = AstNode::Array { items: vec![], form: ArrayForm::Expanded, span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_ast_node_span_string() {
        let node = AstNode::String { value: "hello".to_string(), span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_ast_node_span_number() {
        let node = AstNode::Number { value: NumberValue::PosInt(42), span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_ast_node_span_bool() {
        let node = AstNode::Bool { value: false, span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_ast_node_span_null() {
        let node = AstNode::Null { span: test_span() };
        assert_eq!(node.span(), test_span());
    }

    #[test]
    fn test_number_value_as_f64_pos_int() {
        let val = NumberValue::PosInt(42);
        assert!((val.as_f64() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_number_value_as_f64_neg_int() {
        let val = NumberValue::NegInt(-17);
        assert!((val.as_f64() - -17.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_number_value_as_f64_float() {
        let val = NumberValue::Float(2.5);
        assert!((val.as_f64() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_number_value_as_f64_large_pos_int() {
        let val = NumberValue::PosInt(u64::MAX);
        assert!((val.as_f64() - (u64::MAX as f64)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_number_value_as_f64_large_neg_int() {
        let val = NumberValue::NegInt(i64::MIN);
        assert!((val.as_f64() - (i64::MIN as f64)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_number_value_as_f64_negative_float() {
        let val = NumberValue::Float(-1.5);
        assert!((val.as_f64() - -1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_number_value_as_f64_zero() {
        assert_eq!(NumberValue::PosInt(0).as_f64(), 0.0);
        assert_eq!(NumberValue::NegInt(0).as_f64(), 0.0);
        assert_eq!(NumberValue::Float(0.0).as_f64(), 0.0);
    }

    #[test]
    fn test_array_form_variants() {
        let forms = [ArrayForm::Inline, ArrayForm::Expanded, ArrayForm::Tabular];
        for form in forms {
            let node = AstNode::Array { items: vec![], form, span: test_span() };
            assert_eq!(node.kind(), "array");
        }
    }

    #[test]
    fn test_document_with_children() {
        let child = AstNode::Null { span: test_span_2() };
        let doc = AstNode::Document { children: vec![child], span: test_span() };
        let extracted_span = doc.span();
        assert_eq!(extracted_span, test_span());
    }

    #[test]
    fn test_object_with_entries() {
        let entry = ObjectEntry {
            key: "test".to_string(),
            key_span: test_span_2(),
            value: AstNode::Null { span: test_span() },
        };
        let obj = AstNode::Object { entries: vec![entry], span: test_span() };
        let extracted_span = obj.span();
        assert_eq!(extracted_span, test_span());
    }

    #[test]
    fn test_array_with_items() {
        let item = AstNode::Null { span: test_span_2() };
        let arr = AstNode::Array { items: vec![item], form: ArrayForm::Inline, span: test_span() };
        let extracted_span = arr.span();
        assert_eq!(extracted_span, test_span());
    }

    #[test]
    fn test_string_value() {
        let node = AstNode::String { value: "hello world".to_string(), span: test_span() };
        let extracted_span = node.span();
        assert_eq!(extracted_span, test_span());
    }

    #[test]
    fn test_number_variants() {
        let pos = AstNode::Number { value: NumberValue::PosInt(100), span: test_span() };
        let neg = AstNode::Number { value: NumberValue::NegInt(-50), span: test_span() };
        let float = AstNode::Number { value: NumberValue::Float(1.5), span: test_span() };

        assert_eq!(pos.kind(), "number");
        assert_eq!(neg.kind(), "number");
        assert_eq!(float.kind(), "number");
    }

    #[test]
    fn test_bool_variants() {
        let t = AstNode::Bool { value: true, span: test_span() };
        let f = AstNode::Bool { value: false, span: test_span() };

        assert_eq!(t.kind(), "bool");
        assert_eq!(f.kind(), "bool");
    }

    #[test]
    fn test_eq_derive() {
        let node1 = AstNode::Null { span: test_span() };
        let node2 = AstNode::Null { span: test_span() };
        assert_eq!(node1, node2);
    }

    #[test]
    fn test_eq_different_spans() {
        let node1 = AstNode::Null { span: test_span() };
        let node2 = AstNode::Null { span: test_span_2() };
        assert_ne!(node1, node2);
    }

    #[test]
    fn test_number_value_eq() {
        assert_eq!(NumberValue::PosInt(42), NumberValue::PosInt(42));
        assert_eq!(NumberValue::NegInt(-10), NumberValue::NegInt(-10));
        assert_eq!(NumberValue::Float(3.14), NumberValue::Float(3.14));
    }

    #[test]
    fn test_number_value_ne() {
        assert_ne!(NumberValue::PosInt(42), NumberValue::NegInt(-42));
        assert_ne!(NumberValue::PosInt(42), NumberValue::Float(42.0));
        assert_ne!(NumberValue::NegInt(-10), NumberValue::Float(-10.0));
    }

    #[test]
    fn test_clone() {
        let node = AstNode::String { value: "test".to_string(), span: test_span() };
        let cloned = node.clone();
        assert_eq!(node, cloned);
    }

    #[test]
    fn test_debug_format() {
        let node = AstNode::Null { span: test_span() };
        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("Null"));
    }
}
