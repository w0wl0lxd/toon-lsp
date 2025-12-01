//! TOON Language Server implementation.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::parser;

/// The TOON Language Server.
pub struct ToonLanguageServer {
    client: Client,
}

impl ToonLanguageServer {
    /// Create a new TOON language server.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Validate a document and publish diagnostics.
    async fn validate_document(&self, uri: Url, text: &str) {
        let (_, errors) = parser::parse_with_errors(text);

        let diagnostics: Vec<Diagnostic> = errors
            .into_iter()
            .map(|err| Diagnostic {
                range: Range {
                    start: Position {
                        line: err.span.start.line,
                        character: err.span.start.column,
                    },
                    end: Position {
                        line: err.span.end.line,
                        character: err.span.end.column,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: err.to_string(),
                source: Some("toon-lsp".to_string()),
                ..Default::default()
            })
            .collect();

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
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
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
        self.validate_document(
            params.text_document.uri,
            &params.text_document.text,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.validate_document(params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Clear diagnostics when document is closed
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        // TODO: Implement document symbols from AST
        let _ = params;
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement hover information
        let _ = params;
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // TODO: Implement completions
        let _ = params;
        Ok(None)
    }
}
