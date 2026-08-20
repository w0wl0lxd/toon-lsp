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
//! TOON is a compact encoding of the JSON data model built for LLM prompts.
//!
//! This crate parses TOON into an AST that tracks spans and recovers from errors.
//! The same tree feeds an LSP server built on tower-lsp and a CLI that can encode,
//! decode, check, format, and inspect documents. Run the binary with no arguments
//! to start the server.
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
//! let ast = parse("user:\n  name: Alice\n  age: 30").unwrap();
//! let AstNode::Document { children, .. } = &ast else { return };
//! let AstNode::Object { entries, .. } = &children[0] else { return };
//! assert_eq!(entries[0].key, "user");
//! ```
//!
//! Recover from errors and keep a partial AST for IDE features:
//!
//! ```rust
//! use toon_lsp::parse_with_errors;
//! let (ast, errors) = parse_with_errors("config:\n  debug: true");
//! assert!(ast.is_some());
//! for err in &errors {
//!     eprintln!("L{}: {}", err.span.start.line + 1, err.kind);
//! }
//! ```

pub mod ast;
pub mod cli;
pub mod lsp;
pub mod parser;
pub mod resolve;
pub mod toon;

pub use ast::{AstNode, NumberValue, ObjectEntry, Position, Span};
pub use parser::{ParseError, ParseErrorKind, parse, parse_with_errors};
pub use resolve::{ResolveError, ResolvedRef};
