//! TOON Language Server implementation.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::completion::get_completions_at_position;
use super::diagnostics::errors_to_diagnostics;
use super::formatting::{ToonFormattingOptions, format_document};
use super::goto::get_definition_at_position;
use super::hover::get_hover_at_position;
use super::references::find_references_at_position;
use super::rename::{prepare_rename, rename_key};
use super::state::DocumentState;
use super::symbols::ast_to_document_symbols;
use super::utf16::{span_to_range, utf8_to_utf16_col};

/// The TOON Language Server.
///
/// Manages document state and provides LSP features for TOON files.
pub struct ToonLanguageServer {
    /// The LSP client for sending notifications
    client: Client,
    /// Open documents with their parsed state
    /// Uses nested locking: outer lock for document lifecycle, inner for content
    documents: Arc<RwLock<HashMap<Url, Arc<RwLock<DocumentState>>>>>,
}

impl ToonLanguageServer {
    /// Create a new TOON language server.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a document's state by URI.
    async fn get_document(&self, uri: &Url) -> Option<Arc<RwLock<DocumentState>>> {
        let docs = self.documents.read().await;
        docs.get(uri).cloned()
    }

    /// Publish diagnostics for a document.
    async fn publish_diagnostics(&self, uri: Url, doc: &DocumentState) {
        let diagnostics = errors_to_diagnostics(doc.errors(), doc.text());
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
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

                // T001: Semantic tokens for syntax highlighting
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::PROPERTY, // 0 - object keys
                                    SemanticTokenType::STRING,   // 1 - string values
                                    SemanticTokenType::NUMBER,   // 2 - number values
                                    SemanticTokenType::KEYWORD,  // 3 - true/false/null
                                    SemanticTokenType::OPERATOR, // 4 - : = | >
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DEFINITION, // bit 0 - key definitions
                                    SemanticTokenModifier::READONLY,   // bit 1 - immutable values
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(true),
                            ..Default::default()
                        },
                    ),
                ),

                // T002: Find all references to keys
                references_provider: Some(OneOf::Left(true)),

                // T003: Rename symbols with prepare support
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),

                // T004: Format TOON documents
                document_formatting_provider: Some(OneOf::Left(true)),

                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "toon-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "TOON Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Create document state
        let doc_state = DocumentState::new(text, version);

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

        // Update document state
        if let Some(doc_arc) = self.get_document(&uri).await {
            {
                let mut doc = doc_arc.write().await;
                doc.update(text, version);
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
        let uri = params.text_document.uri;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                let symbols = ast_to_document_symbols(ast, doc.text());
                return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
            }
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Convert UTF-16 column to UTF-8
                let utf8_col = doc.utf8_col_at(position.line, position.character);

                if let Some(hover_info) =
                    get_hover_at_position(ast, doc.text(), position.line, utf8_col)
                {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_info.contents,
                        }),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc_arc) = self.get_document(&uri).await {
            let doc = doc_arc.read().await;
            if let Some(ast) = doc.ast() {
                // Convert UTF-16 column to UTF-8
                let utf8_col = doc.utf8_col_at(position.line, position.character);

                let completions =
                    get_completions_at_position(ast, doc.text(), position.line, utf8_col);

                if !completions.is_empty() {
                    let items: Vec<CompletionItem> =
                        completions.into_iter().map(Into::into).collect();
                    return Ok(Some(CompletionResponse::Array(items)));
                }
            }
        }

        Ok(None)
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
                                    start: Position {
                                        line: loc.line,
                                        character: start_utf16,
                                    },
                                    end: Position {
                                        line: loc.line,
                                        character: end_utf16,
                                    },
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
                            TextEdit {
                                range,
                                new_text: edit.new_text,
                            }
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
                let all_tokens =
                    crate::lsp::semantic_tokens::collect_semantic_tokens(ast);

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
                    .map(|l| utf8_to_utf16_col(l, l.len() as u32))
                    .unwrap_or(0);

                return Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    new_text: formatted,
                }]));
            }
        }

        Ok(None)
    }
}
