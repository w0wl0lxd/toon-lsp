//! Tests for rename symbol functionality in the LSP server.

use toon_lsp::lsp::rename::{prepare_rename, rename_key};
use toon_lsp::parser::parse_with_errors;

/// Test prepare_rename validation
mod prepare_rename_tests {
    use super::*;

    #[test]
    fn test_prepare_rename_validates_cursor_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "name" key
        let result = prepare_rename(&ast, source, 0, 1);

        assert!(result.is_some(), "Should validate when on key");
        let result = result.unwrap();
        assert_eq!(result.placeholder, "name");
        assert_eq!(result.range.start.column, 0);
        assert_eq!(result.range.end.column, 4);
    }

    #[test]
    fn test_prepare_rename_rejects_cursor_on_value() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "Alice" (value)
        let result = prepare_rename(&ast, source, 0, 7);

        assert!(result.is_none(), "Should reject when on value");
    }

    #[test]
    fn test_prepare_rename_returns_correct_placeholder() {
        let source = "username: bob123";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let result = prepare_rename(&ast, source, 0, 4).unwrap();

        assert_eq!(result.placeholder, "username");
    }

    #[test]
    fn test_prepare_rename_nested_key() {
        let source = "user:\n  name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on nested "name" key (line 1, col 2)
        let result = prepare_rename(&ast, source, 1, 2);

        assert!(result.is_some());
        assert_eq!(result.unwrap().placeholder, "name");
    }
}

/// Test rename_key functionality
mod rename_key_tests {
    use super::*;

    #[test]
    fn test_rename_updates_all_occurrences() {
        let source = "id: 1\ndata:\n  id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "id"
        let edits = rename_key(&ast, source, 0, 0, "identifier");

        // Should rename both occurrences
        assert_eq!(edits.len(), 2, "Should have 2 edits");
        assert_eq!(edits[0].new_text, "identifier");
        assert_eq!(edits[0].span.start.line, 0);
        assert_eq!(edits[1].new_text, "identifier");
        assert_eq!(edits[1].span.start.line, 2);
    }

    #[test]
    fn test_rename_single_occurrence() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let edits = rename_key(&ast, source, 0, 0, "new_key");

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "new_key");
    }

    #[test]
    fn test_rename_empty_when_not_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on value
        let edits = rename_key(&ast, source, 0, 7, "newname");

        assert_eq!(edits.len(), 0, "Should return empty when not on key");
    }

    #[test]
    fn test_rename_preserves_document_structure() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let edits = rename_key(&ast, source, 0, 0, "new_key");

        // Apply the edit
        let new_text = apply_edits(source, &edits);
        assert_eq!(new_text, "new_key: value");

        // Verify it still parses
        let (new_ast, errors) = parse_with_errors(&new_text);
        assert!(errors.is_empty(), "Renamed document should parse");
        assert!(new_ast.is_some());
    }

    #[test]
    fn test_rename_with_special_characters() {
        let source = "my_key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let edits = rename_key(&ast, source, 0, 0, "my_new_key");

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].new_text, "my_new_key");
        assert_eq!(edits[0].span.start.column, 0);
        assert_eq!(edits[0].span.end.column, 6); // "my_key" is 6 chars
    }

    #[test]
    fn test_rename_creating_duplicate_keys() {
        // This test verifies that rename_key returns edits even if
        // they would create duplicate keys (LSP handler can warn)
        let source = "name: Alice\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Rename "age" to "name" (creates duplicate)
        let edits = rename_key(&ast, source, 1, 0, "name");

        assert_eq!(edits.len(), 1, "Should still return edit");
        assert_eq!(edits[0].new_text, "name");
    }

    /// Helper to apply edits to source text
    fn apply_edits(source: &str, edits: &[toon_lsp::lsp::rename::RenameEdit]) -> String {
        let mut sorted_edits = edits.to_vec();
        sorted_edits.sort_by(|a, b| b.span.start.offset.cmp(&a.span.start.offset));

        let mut result = source.to_string();
        for edit in sorted_edits {
            let start = edit.span.start.offset as usize;
            let end = edit.span.end.offset as usize;
            result.replace_range(start..end, &edit.new_text);
        }

        result
    }
}

/// Test rename with nested structures
mod nested_rename_tests {
    use super::*;

    #[test]
    fn test_rename_in_nested_objects() {
        let source = "data:\n  id: 1\n  nested:\n    id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "id"
        let edits = rename_key(&ast, source, 1, 2, "identifier");

        // Should rename both "id" keys
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].span.start.line, 1);
        assert_eq!(edits[1].span.start.line, 3);
    }

    #[test]
    fn test_rename_does_not_affect_different_keys() {
        let source = "name: Alice\nusername: bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Rename "name" should not affect "username"
        let edits = rename_key(&ast, source, 0, 0, "fullname");

        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].span.start.line, 0);
    }
}

/// Test rename sorting
mod rename_sorting_tests {
    use super::*;

    #[test]
    fn test_rename_edits_sorted_by_position() {
        let source = "name: C\ndata:\n  name: B\nname: A";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let edits = rename_key(&ast, source, 0, 0, "identifier");

        // Edits should be sorted by line/column
        assert_eq!(edits.len(), 3);
        assert_eq!(edits[0].span.start.line, 0);
        assert_eq!(edits[1].span.start.line, 2);
        assert_eq!(edits[2].span.start.line, 3);
    }
}
