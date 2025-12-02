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
//! ```rust,ignore
//! use toon_lsp::{parse, AstNode};
//!
//! let source = r#"
//! name: Alice
//! age: 30
//! "#;
//!
//! let ast = parse(source)?;
//! for node in ast.iter() {
//!     println!("{:?} at {:?}", node.kind(), node.span());
//! }
//! ```

pub mod ast;
pub mod lsp;
pub mod parser;

pub use ast::{AstNode, NumberValue, ObjectEntry, Position, Span};
pub use parser::{ParseError, ParseErrorKind, parse, parse_with_errors};
