//! Tests for LSP server capability declarations
//!
//! Verifies that the server correctly advertises advanced LSP features
//! including semantic tokens, references, rename, and formatting.

use toon_lsp::lsp::ToonLanguageServer;
use tower_lsp::LanguageServer;
use tower_lsp::lsp_types::{InitializeParams, OneOf, SemanticTokenModifier, SemanticTokenType};

/// Helper to get server capabilities from initialization
async fn get_server_capabilities() -> tower_lsp::lsp_types::ServerCapabilities {
    // Create a test server with LspService
    let (service, _socket) =
        tower_lsp::LspService::build(|client| ToonLanguageServer::new(client)).finish();

    // Call initialize through the service
    let params = InitializeParams::default();
    let result = service.inner().initialize(params).await.unwrap();
    result.capabilities
}

#[tokio::test]
async fn test_semantic_tokens_capability_declared() {
    let caps = get_server_capabilities().await;

    // T001: Verify semantic tokens capability exists
    assert!(caps.semantic_tokens_provider.is_some(), "semantic_tokens_provider must be declared");

    let semantic_tokens = caps.semantic_tokens_provider.unwrap();

    // Verify legend exists
    let legend = match semantic_tokens {
        tower_lsp::lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(opts) => {
            opts.legend
        }
        tower_lsp::lsp_types::SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(opts) => {
            opts.semantic_tokens_options.legend
        }
    };

    // Verify token types: property (0), string (1), number (2), keyword (3), operator (4)
    assert_eq!(legend.token_types.len(), 5, "Must have 5 token types");
    assert_eq!(legend.token_types[0], SemanticTokenType::PROPERTY);
    assert_eq!(legend.token_types[1], SemanticTokenType::STRING);
    assert_eq!(legend.token_types[2], SemanticTokenType::NUMBER);
    assert_eq!(legend.token_types[3], SemanticTokenType::KEYWORD);
    assert_eq!(legend.token_types[4], SemanticTokenType::OPERATOR);

    // Verify token modifiers: definition (bit 0), readonly (bit 1)
    assert_eq!(legend.token_modifiers.len(), 2, "Must have 2 token modifiers");
    assert_eq!(legend.token_modifiers[0], SemanticTokenModifier::DEFINITION);
    assert_eq!(legend.token_modifiers[1], SemanticTokenModifier::READONLY);
}

#[tokio::test]
async fn test_semantic_tokens_full_capability() {
    let caps = get_server_capabilities().await;

    let semantic_tokens = caps.semantic_tokens_provider.unwrap();

    // Verify full capability is declared
    let has_full = match semantic_tokens {
        tower_lsp::lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(opts) => {
            opts.full.is_some()
        }
        tower_lsp::lsp_types::SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(opts) => {
            opts.semantic_tokens_options.full.is_some()
        }
    };

    assert!(has_full, "semantic_tokens must declare 'full' capability");
}

#[tokio::test]
async fn test_semantic_tokens_range_capability() {
    let caps = get_server_capabilities().await;

    let semantic_tokens = caps.semantic_tokens_provider.unwrap();

    // Verify range capability is declared
    let has_range = match semantic_tokens {
        tower_lsp::lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(opts) => {
            opts.range.is_some()
        }
        tower_lsp::lsp_types::SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(opts) => {
            opts.semantic_tokens_options.range.is_some()
        }
    };

    assert!(has_range, "semantic_tokens must declare 'range' capability");
}

#[tokio::test]
async fn test_references_provider_declared() {
    let caps = get_server_capabilities().await;

    // T002: Verify references provider is enabled
    assert!(caps.references_provider.is_some(), "references_provider must be declared");

    // Should be simple boolean true
    match caps.references_provider.unwrap() {
        OneOf::Left(enabled) => assert!(enabled, "references_provider must be enabled"),
        OneOf::Right(_) => panic!("references_provider should be simple boolean"),
    }
}

#[tokio::test]
async fn test_rename_provider_declared() {
    let caps = get_server_capabilities().await;

    // T003: Verify rename provider is enabled
    assert!(caps.rename_provider.is_some(), "rename_provider must be declared");

    // Should use RenameOptions with prepare_provider
    match caps.rename_provider.unwrap() {
        OneOf::Left(_) => panic!("rename_provider should use RenameOptions"),
        OneOf::Right(opts) => {
            assert_eq!(
                opts.prepare_provider,
                Some(true),
                "rename_provider must have prepare_provider enabled"
            );
        }
    }
}

#[tokio::test]
async fn test_document_formatting_provider_declared() {
    let caps = get_server_capabilities().await;

    // T004: Verify document formatting provider is enabled
    assert!(
        caps.document_formatting_provider.is_some(),
        "document_formatting_provider must be declared"
    );

    // Should be simple boolean true
    match caps.document_formatting_provider.unwrap() {
        OneOf::Left(enabled) => assert!(enabled, "document_formatting_provider must be enabled"),
        OneOf::Right(_) => panic!("document_formatting_provider should be simple boolean"),
    }
}

#[tokio::test]
async fn test_all_existing_capabilities_preserved() {
    let caps = get_server_capabilities().await;

    // Ensure existing capabilities are not lost
    assert!(caps.text_document_sync.is_some(), "text_document_sync must exist");
    assert!(caps.hover_provider.is_some(), "hover_provider must exist");
    assert!(caps.completion_provider.is_some(), "completion_provider must exist");
    assert!(caps.definition_provider.is_some(), "definition_provider must exist");
    assert!(caps.document_symbol_provider.is_some(), "document_symbol_provider must exist");
}
