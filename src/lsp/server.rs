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

//! TOON Language Server implementation.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::ast::AstNode;

use super::code_actions::collect_code_actions;
use super::code_lens::collect_code_lenses;
use super::completion::get_completions_at_position;
use super::diagnostics::errors_to_diagnostics;
use super::document_highlight::collect_document_highlights;
use super::document_links::collect_document_links;
use super::folding::collect_folding_ranges;
use super::formatting::{ToonFormattingOptions, format_document};
use super::goto::get_definition_at_position;
use super::hover::get_hover_at_position;
use super::inlay_hints::collect_inlay_hints;
use super::linked_editing::collect_linked_editing_ranges;
use super::references::find_references_at_position;
use super::rename::{prepare_rename, rename_key};
use super::selection_ranges::get_selection_ranges;
use super::state::DocumentState;
use super::symbols::ast_to_document_symbols;
use super::utf16::{span_to_range, utf8_to_utf16_col};
use super::workspace_symbols::collect_workspace_symbols;

/// Type alias for a shared reference to a document state.
type DocRef = Arc<RwLock<DocumentState>>;

/// The TOON Language Server.
pub struct ToonLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, DocRef>>>,
}

impl ToonLanguageServer {
    /// Create a new TOON language server.
    pub fn new(client: Client) -> Self {
        Self { client, documents: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Get a document's state by URI.
    async fn get_document(&self, uri: &Url) -> Option<DocRef> {
        self.documents.read().await.get(uri).cloned()
    }

    /// Try to access document AST and text for a given URI.
    async fn with_ast<F, R>(&self, uri: &Url, f: F) -> Option<R>
    where
        F: FnOnce(&AstNode, &str) -> Option<R>,
    {
        let doc = self.get_document(uri).await?;
        let doc = doc.read().await;
        let ast = doc.ast()?;
        f(ast, doc.text())
    }

    /// Publish diagnostics for a document.
    async fn publish_diagnostics(&self, uri: Url, doc: &DocumentState) {
        let diagnostics = errors_to_diagnostics(doc.errors(), doc.text());
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ToonLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::OPERATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(true),
                            ..Default::default()
                        },
                    ),
                ),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(false) }),
                document_highlight_provider: Some(OneOf::Left(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                linked_editing_range_provider: Some(LinkedEditingRangeServerCapabilities::Simple(
                    true,
                )),
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "toon-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "TOON Language Server initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Parse on blocking thread pool to avoid blocking async runtime
        let text_clone = text.clone();
        let parse_result =
            tokio::task::spawn_blocking(move || crate::parser::parse_with_errors(&text_clone))
                .await
                .expect("parsing task panicked");

        let (ast, errors) = parse_result;

        // Create document state with pre-parsed data
        let mut doc_state = DocumentState::new(String::new(), 0);
        doc_state.update_parsed(text, version, ast, errors);

        // Store in documents map
        {
            let mut docs = self.documents.write().await;
            docs.insert(uri.clone(), Arc::new(RwLock::new(doc_state)));
        }

        // Publish diagnostics
        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            self.publish_diagnostics(uri, &doc).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Get full text from changes (we use FULL sync)
        let text = match params.content_changes.into_iter().next() {
            Some(change) => change.text,
            None => return,
        };

        // Parse on blocking thread pool to avoid blocking async runtime
        let text_clone = text.clone();
        let parse_result =
            tokio::task::spawn_blocking(move || crate::parser::parse_with_errors(&text_clone))
                .await
                .expect("parsing task panicked");

        let (ast, errors) = parse_result;

        // Update document state with pre-parsed data
        if let Some(doc_arc) = self.get_document(&uri).await {
            {
                let mut doc = doc_arc.write().await;
                doc.update_parsed(text, version, ast, errors);
            }

            // Publish diagnostics after releasing write lock
            let doc = doc_arc.read().await;
            self.publish_diagnostics(uri, &doc).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document state
        {
            let mut docs = self.documents.write().await;
            docs.remove(&uri);
        }

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let result = self
            .with_ast(&params.text_document.uri, |ast, text| {
                Some(DocumentSymbolResponse::Nested(ast_to_document_symbols(ast, text)))
            })
            .await;
        Ok(result)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        Ok(self.with_ast(uri, |ast, text| {
            let utf8_col = crate::lsp::utf16::utf16_to_utf8_col(
                text.lines().nth(pos.line as usize).unwrap_or(""),
                pos.character,
            );
            get_hover_at_position(ast, text, pos.line, utf8_col).map(|hover_info| Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_info.contents,
                }),
                range: None,
            })
        }).await)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        Ok(self.with_ast(uri, |ast, text| {
            let utf8_col = crate::lsp::utf16::utf16_to_utf8_col(
                text.lines().nth(pos.line as usize).unwrap_or(""),
                pos.character,
            );
            let completions = get_completions_at_position(ast, text, pos.line, utf8_col);
            if completions.is_empty() {
                None
            } else {
                Some(CompletionResponse::Array(completions.into_iter().map(Into::into).collect()))
            }
        }).await)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Convert UTF-16 column to UTF-8
                let utf8_col = doc.utf8_col_at(position.line, position.character);

                let locations =
                    get_definition_at_position(ast, doc.text(), position.line, utf8_col);

                if !locations.is_empty() {
                    let lsp_locations: Vec<Location> = locations
                        .into_iter()
                        .map(|loc| {
                            // Convert UTF-8 columns back to UTF-16
                            let loc_line_text = doc.get_line(loc.line).unwrap_or("");
                            let start_utf16 = utf8_to_utf16_col(loc_line_text, loc.start_col);
                            let end_utf16 = utf8_to_utf16_col(loc_line_text, loc.end_col);

                            Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position { line: loc.line, character: start_utf16 },
                                    end: Position { line: loc.line, character: end_utf16 },
                                },
                            }
                        })
                        .collect();

                    return Ok(Some(GotoDefinitionResponse::Array(lsp_locations)));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Convert UTF-16 column to UTF-8
                let utf8_col = doc.utf8_col_at(position.line, position.character);

                let refs = find_references_at_position(
                    ast,
                    doc.text(),
                    position.line,
                    utf8_col,
                    include_declaration,
                );

                if !refs.is_empty() {
                    // Convert KeyReference to LSP Location with UTF-16 positions
                    let locations: Vec<Location> = refs
                        .into_iter()
                        .map(|key_ref| {
                            // Convert UTF-8 columns back to UTF-16
                            let loc_line_text = doc.get_line(key_ref.span.start.line).unwrap_or("");
                            let start_utf16 =
                                utf8_to_utf16_col(loc_line_text, key_ref.span.start.column);
                            let end_utf16 =
                                utf8_to_utf16_col(loc_line_text, key_ref.span.end.column);

                            Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: key_ref.span.start.line,
                                        character: start_utf16,
                                    },
                                    end: Position {
                                        line: key_ref.span.end.line,
                                        character: end_utf16,
                                    },
                                },
                            }
                        })
                        .collect();

                    return Ok(Some(locations));
                }
            }
        }

        Ok(None)
    }

    /// Validate if rename is possible at the cursor position.
    ///
    /// Returns the range and placeholder text for the key to be renamed.
    /// Returns None if the cursor is not on a renameable symbol (key).
    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Convert UTF-16 column to UTF-8
                let utf8_col = doc.utf8_col_at(position.line, position.character);

                if let Some(prepare_result) =
                    prepare_rename(ast, doc.text(), position.line, utf8_col)
                {
                    // Convert span to UTF-16 range for LSP
                    let range = span_to_range(&prepare_result.range, doc.text());

                    return Ok(Some(PrepareRenameResponse::Range(range)));
                }
            }
        }

        Ok(None)
    }

    /// Rename all occurrences of a symbol (key) in the document.
    ///
    /// Returns WorkspaceEdit containing text edits for all occurrences of the key.
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Convert UTF-16 column to UTF-8
                let utf8_col = doc.utf8_col_at(position.line, position.character);

                let edits = rename_key(ast, doc.text(), position.line, utf8_col, &new_name);

                if !edits.is_empty() {
                    // Convert RenameEdit to LSP TextEdit with UTF-16 positions
                    let text_edits: Vec<TextEdit> = edits
                        .into_iter()
                        .map(|edit| {
                            let range = span_to_range(&edit.span, doc.text());
                            TextEdit { range, new_text: edit.new_text }
                        })
                        .collect();

                    // Create WorkspaceEdit
                    let mut changes = HashMap::new();
                    changes.insert(uri, text_edits);

                    return Ok(Some(WorkspaceEdit {
                        changes: Some(changes),
                        document_changes: None,
                        change_annotations: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Handle semantic tokens request for full document.
    ///
    /// Returns semantic tokens for the entire document, providing
    /// syntax highlighting information for all tokens.
    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                let tokens = crate::lsp::semantic_tokens::collect_semantic_tokens(ast);
                let encoded = crate::lsp::semantic_tokens::encode_tokens(&tokens, doc.text());

                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: encoded,
                })));
            }
        }

        Ok(None)
    }

    /// Handle semantic tokens request for a specific range.
    ///
    /// Returns semantic tokens for only the specified range of the document.
    /// Filters out tokens outside the requested range.
    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let uri = params.text_document.uri;
        let range = params.range;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Collect all tokens
                let all_tokens = crate::lsp::semantic_tokens::collect_semantic_tokens(ast);

                // Filter tokens within the requested range
                let filtered_tokens: Vec<_> = all_tokens
                    .into_iter()
                    .filter(|token| {
                        // Token is in range if it starts after range.start and ends before range.end
                        let token_line = token.line;
                        let token_start = token.start_col;
                        let token_end = token.start_col + token.length;

                        // Check if token overlaps with requested range
                        if token_line < range.start.line || token_line > range.end.line {
                            return false;
                        }

                        if token_line == range.start.line && token_end <= range.start.character {
                            return false;
                        }

                        if token_line == range.end.line && token_start >= range.end.character {
                            return false;
                        }

                        true
                    })
                    .collect();

                let encoded =
                    crate::lsp::semantic_tokens::encode_tokens(&filtered_tokens, doc.text());

                return Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
                    result_id: None,
                    data: encoded,
                })));
            }
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let mut all_symbols = Vec::new();

        let docs = self.documents.read().await;
        for (uri, doc_arc) in docs.iter() {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                let matching = collect_workspace_symbols(ast, uri)
                    .into_iter()
                    .filter(|ws| query.is_empty() || ws.name.to_lowercase().contains(&query))
                    .map(|ws| {
                        let location = match ws.location {
                            OneOf::Left(loc) => loc,
                            OneOf::Right(_) => Location { uri: uri.clone(), range: Range::default() },
                        };
                        SymbolInformation {
                            name: ws.name,
                            kind: ws.kind,
                            tags: ws.tags,
                            location,
                            container_name: ws.container_name,
                            #[allow(deprecated)]
                            deprecated: None,
                        }
                    });
                all_symbols.extend(matching);
            }
        }
        drop(docs);

        if all_symbols.is_empty() { Ok(None) } else { Ok(Some(all_symbols)) }
    }

    async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> Result<Option<Vec<FoldingRange>>> {
        Ok(self.with_ast(&params.text_document.uri, |ast, _text| {
            let ranges = collect_folding_ranges(ast);
            if ranges.is_empty() { None } else { Some(ranges) }
        }).await)
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        Ok(self.with_ast(&params.text_document.uri, |ast, text| {
            let actions = collect_code_actions(ast, text, &params.text_document.uri, params.range, &params.context.diagnostics);
            if actions.is_empty() {
                None
            } else {
                Some(actions.into_iter().map(CodeActionOrCommand::CodeAction).collect())
            }
        }).await)
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        Ok(self.with_ast(&params.text_document.uri, |ast, text| {
            let positions: Vec<(u32, u32)> = params
                .positions
                .iter()
                .map(|p| (p.line, crate::lsp::utf16::utf16_to_utf8_col(
                    text.lines().nth(p.line as usize).unwrap_or(""),
                    p.character,
                )))
                .collect();
            let ranges = get_selection_ranges(ast, text, &positions);
            let result: Vec<SelectionRange> = ranges.into_iter().flatten().collect();
            if result.is_empty() { None } else { Some(result) }
        }).await)
    }

    async fn document_link(
        &self,
        params: DocumentLinkParams,
    ) -> Result<Option<Vec<DocumentLink>>> {
        Ok(self.with_ast(&params.text_document.uri, |ast, text| {
            let links = collect_document_links(ast, text);
            if links.is_empty() { None } else { Some(links) }
        }).await)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        Ok(self.with_ast(uri, |ast, text| {
            let utf8_col = crate::lsp::utf16::utf16_to_utf8_col(
                text.lines().nth(pos.line as usize).unwrap_or(""),
                pos.character,
            );
            let highlights = collect_document_highlights(ast, text, pos.line, utf8_col);
            if highlights.is_empty() { None } else { Some(highlights) }
        }).await)
    }

    async fn inlay_hint(
        &self,
        params: InlayHintParams,
    ) -> Result<Option<Vec<InlayHint>>> {
        Ok(self.with_ast(&params.text_document.uri, |ast, text| {
            let hints = collect_inlay_hints(ast, text, Some(params.range));
            if hints.is_empty() { None } else { Some(hints) }
        }).await)
    }

    async fn code_lens(
        &self,
        params: CodeLensParams,
    ) -> Result<Option<Vec<CodeLens>>> {
        Ok(self.with_ast(&params.text_document.uri, |ast, text| {
            let lenses = collect_code_lenses(ast, text, &params.text_document.uri);
            if lenses.is_empty() { None } else { Some(lenses) }
        }).await)
    }

    async fn linked_editing_range(
        &self,
        params: LinkedEditingRangeParams,
    ) -> Result<Option<LinkedEditingRanges>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        Ok(self.with_ast(uri, |ast, text| {
            let utf8_col = crate::lsp::utf16::utf16_to_utf8_col(
                text.lines().nth(pos.line as usize).unwrap_or(""),
                pos.character,
            );
            collect_linked_editing_ranges(ast, text, pos.line, utf8_col)
        }).await)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let options = ToonFormattingOptions::from(&params.options);

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;

            // Skip formatting if document has parse errors
            if !doc.errors().is_empty() {
                return Ok(None);
            }

            if let Some(ast) = doc.ast()
                && let Some(formatted) = format_document(ast, options)
            {
                // Return single TextEdit replacing entire document
                let end_line = doc.text().lines().count().saturating_sub(1) as u32;
                let end_col = doc
                    .text()
                    .lines()
                    .nth(end_line as usize)
                    .map_or(0, |l| utf8_to_utf16_col(l, l.len() as u32));

                return Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: end_line, character: end_col },
                    },
                    new_text: formatted,
                }]));
            }
        }

        Ok(None)
    }
}
