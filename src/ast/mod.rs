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

//! Abstract Syntax Tree types for TOON with source position tracking.
//!
//! This module provides AST node types that preserve source locations (spans)
//! for error reporting, syntax highlighting, and IDE features.

mod node;
mod span;

pub use node::{ArrayForm, AstNode, NumberValue, ObjectEntry};
pub use span::{Position, Span};

// AST types fully implement the TOON spec.
// Reference: https://github.com/toon-format/spec/blob/main/SPEC.md
