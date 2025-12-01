//! Language Server Protocol implementation for TOON.
//!
//! This module provides the LSP server that integrates with editors
//! like VS Code, Neovim, and others.

mod server;
mod capabilities;

pub use server::ToonLanguageServer;
