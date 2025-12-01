//! TOON Language Server implementation.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::completion::get_completions_at_position;
use super::diagnostics::errors_to_diagnostics;
use super::goto::get_definition_at_position;
use super::hover::get_hover_at_position;
use super::state::DocumentState;
use super::symbols::ast_to_document_symbols;
use super::utf16::{utf8_to_utf16_col, utf16_to_utf8_col};

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
        let uri = params.text_document.uri.clone();
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
        let uri = params.text_document.uri.clone();
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
                let line_text = doc.get_line(position.line).unwrap_or("");
                let utf8_col = utf16_to_utf8_col(line_text, position.character);

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
                let line_text = doc.get_line(position.line).unwrap_or("");
                let utf8_col = utf16_to_utf8_col(line_text, position.character);

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
                let line_text = doc.get_line(position.line).unwrap_or("");
                let utf8_col = utf16_to_utf8_col(line_text, position.character);

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
}
