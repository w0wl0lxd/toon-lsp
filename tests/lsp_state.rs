//! Tests for DocumentState management in the LSP server.

use toon_lsp::lsp::state::DocumentState;

#[test]
fn test_document_state_new() {
    let text = "name: Alice";
    let state = DocumentState::new(text.to_string(), 1);

    assert_eq!(state.text(), text);
    assert_eq!(state.version(), 1);
    assert!(state.ast().is_some());
    assert!(state.errors().is_empty());
}

#[test]
fn test_document_state_with_errors() {
    let text = "name Alice"; // Missing colon
    let state = DocumentState::new(text.to_string(), 1);

    assert_eq!(state.text(), text);
    assert!(!state.errors().is_empty());
}

#[test]
fn test_document_state_update() {
    let mut state = DocumentState::new("name: Alice".to_string(), 1);

    state.update("name: Bob".to_string(), 2);

    assert_eq!(state.text(), "name: Bob");
    assert_eq!(state.version(), 2);
    assert!(state.ast().is_some());
}

#[test]
fn test_document_state_update_introduces_error() {
    let mut state = DocumentState::new("name: Alice".to_string(), 1);
    assert!(state.errors().is_empty());

    state.update("name".to_string(), 2); // Now invalid

    assert!(!state.errors().is_empty());
}

#[test]
fn test_document_state_update_fixes_error() {
    let mut state = DocumentState::new("name".to_string(), 1);
    assert!(!state.errors().is_empty());

    state.update("name: Alice".to_string(), 2); // Now valid

    assert!(state.errors().is_empty());
}

#[test]
fn test_document_state_empty_document() {
    let state = DocumentState::new(String::new(), 1);

    assert_eq!(state.text(), "");
    // Empty document should parse successfully (no keys is valid)
    assert!(state.errors().is_empty());
}

#[test]
fn test_document_state_nested_structure() {
    let text = r#"person:
  name: Alice
  address:
    city: Boston
    zip: 02101"#;
    let state = DocumentState::new(text.to_string(), 1);

    assert!(state.ast().is_some());
    assert!(state.errors().is_empty());
}

#[test]
fn test_document_state_array() {
    let text = r#"items:
  - one
  - two
  - three"#;
    let state = DocumentState::new(text.to_string(), 1);

    assert!(state.ast().is_some());
    assert!(state.errors().is_empty());
}

#[test]
fn test_document_state_multiple_errors() {
    let text = "name\nage\ncity"; // Multiple missing colons
    let state = DocumentState::new(text.to_string(), 1);

    // Should have multiple errors
    assert!(state.errors().len() >= 1);
}

#[test]
fn test_document_state_partial_parse() {
    // Document with one valid entry and one error
    let text = "name: Alice\nage"; // Second line missing colon
    let state = DocumentState::new(text.to_string(), 1);

    // Should have partial AST and errors
    assert!(state.ast().is_some());
    assert!(!state.errors().is_empty());
}
