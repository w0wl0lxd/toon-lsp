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

//! # toon-lsp
//!
//! A Language Server Protocol (LSP) implementation for TOON (Token-Oriented Object Notation).
//!
//! TOON is a compact, human-readable encoding of the JSON data model designed for LLM prompts.
//! This crate provides:
//!
//! - **AST with source positions** - Full abstract syntax tree with span information
//! - **Parser** - TOON parser that produces positioned AST nodes
//! - **LSP Server** - Complete language server with diagnostics, symbols, and more
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │   Scanner   │ ──▶ │   Parser    │ ──▶ │     AST     │
//! │  (Lexer)    │     │             │     │ (with Spans)│
//! └─────────────┘     └─────────────┘     └─────────────┘
//!                                                │
//!                                                ▼
//!                                         ┌─────────────┐
//!                                         │ LSP Server  │
//!                                         │ (tower-lsp) │
//!                                         └─────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use toon_lsp::{parse, AstNode};
//!
//! let source = "user:\n  name: Alice\n  age: 30";
//! let ast = parse(source).expect("valid TOON");
//!
//! // Every AST node carries source positions (Span)
//! let AstNode::Document { children, span } = &ast else { return };
//! assert_eq!(span.start.line, 0); // 0-indexed
//!
//! // Walk objects via entries
//! for node in children {
//!     if let AstNode::Object { entries, .. } = node {
//!         assert_eq!(entries[0].key, "user");
//!     }
//! }
//! ```
//!
//! **Error recovery for IDEs** — parse succeeds even with syntax errors:
//!
//! ```rust
//! use toon_lsp::parse_with_errors;
//!
//! let source = "config:\n  debug: true\n  port: 8080";
//! let (ast, errors) = parse_with_errors(source);
//!
//! // Partial AST available for IDE features even with errors
//! assert!(ast.is_some());
//!
//! // Errors carry spans for diagnostic rendering
//! for err in &errors {
//!     let _ = (err.span.start.line, err.kind.clone());
//! }
//! ```

pub mod ast;
pub mod cli;
pub mod lsp;
pub mod parser;

pub use ast::{AstNode, NumberValue, ObjectEntry, Position, Span};
pub use parser::{ParseError, ParseErrorKind, parse, parse_with_errors};
