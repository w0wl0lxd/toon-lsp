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

//! Language Server Protocol implementation for TOON.
//!
//! This module provides the LSP server that integrates with editors
//! like VS Code, Neovim, and others.

mod ast_utils;
mod capabilities;
pub mod code_actions;
pub mod code_lens;
pub mod completion;
pub mod diagnostics;
pub mod document_highlight;
pub mod document_links;
pub mod folding;
pub mod formatting;
pub mod goto;
pub mod hover;
pub mod inlay_hints;
pub mod linked_editing;
pub mod references;
pub mod rename;
pub mod selection_ranges;
pub mod semantic_tokens;
mod server;
pub mod state;
pub mod symbols;
mod utf16;
pub mod workspace_symbols;

pub use ast_utils::{
    NodeAtPosition, NodePathEntry, collect_all_keys, collect_parent_keys, collect_sibling_keys,
    find_key_definitions, find_node_at_position,
};
pub use code_actions::collect_code_actions;
pub use code_lens::collect_code_lenses;
pub use completion::{ToonCompletion, get_completions_at_position};
pub use diagnostics::{error_to_diagnostic, errors_to_diagnostics};
pub use document_highlight::collect_document_highlights;
pub use document_links::collect_document_links;
pub use folding::collect_folding_ranges;
pub use formatting::{ToonFormattingOptions, format_document};
pub use goto::{DefinitionLocation, get_definition_at_position};
pub use hover::{HoverInfo, get_hover_at_position};
pub use inlay_hints::collect_inlay_hints;
pub use linked_editing::collect_linked_editing_ranges;
pub use references::{KeyReference, find_references_at_position};
pub use rename::{PrepareRenameResult, RenameEdit, prepare_rename, rename_key};
pub use selection_ranges::get_selection_ranges;
pub use semantic_tokens::{SemanticToken, ToonTokenModifier, ToonTokenType};
pub use server::ToonLanguageServer;
pub use state::DocumentState;
pub use symbols::ast_to_document_symbols;
pub use utf16::{position_to_utf8_col, span_to_range, utf8_to_utf16_col, utf16_to_utf8_col};
pub use workspace_symbols::collect_workspace_symbols;
