//! Language Server Protocol implementation for TOON.
//!
//! This module provides the LSP server that integrates with editors
//! like VS Code, Neovim, and others.

mod ast_utils;
mod capabilities;
pub mod completion;
pub mod diagnostics;
pub mod goto;
pub mod hover;
mod server;
pub mod state;
pub mod symbols;
mod utf16;

pub use ast_utils::{
    NodeAtPosition, NodePathEntry, collect_parent_keys, collect_sibling_keys, find_key_definitions,
    find_node_at_position,
};
pub use completion::{ToonCompletion, get_completions_at_position};
pub use diagnostics::{error_to_diagnostic, errors_to_diagnostics};
pub use goto::{DefinitionLocation, get_definition_at_position};
pub use hover::{HoverInfo, get_hover_at_position};
pub use server::ToonLanguageServer;
pub use state::DocumentState;
pub use symbols::ast_to_document_symbols;
pub use utf16::{position_to_utf8_col, span_to_range, utf8_to_utf16_col, utf16_to_utf8_col};
