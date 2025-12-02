//! Tests for find references functionality in the LSP server.

use toon_lsp::lsp::references::find_references_at_position;
use toon_lsp::parser::parse_with_errors;

/// Test basic reference finding
mod basic_references {
    use super::*;

    #[test]
    fn test_references_finds_all_occurrences() {
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "name" key
        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Should return 2 references
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].key_name, "name");
        assert_eq!(refs[0].span.start.line, 0);
        assert_eq!(refs[1].key_name, "name");
        assert_eq!(refs[1].span.start.line, 2);
    }

    #[test]
    fn test_references_exact_match_only() {
        let source = "name: x\nfullname: y";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "name"
        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Should NOT include "fullname"
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key_name, "name");
    }

    #[test]
    fn test_references_empty_when_not_on_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on value "Alice" (col 6)
        let refs = find_references_at_position(&ast, source, 0, 6, true);

        // Should return empty
        assert_eq!(refs.len(), 0);
    }
}

/// Test include_declaration flag behavior
mod declaration_flag {
    use super::*;

    #[test]
    fn test_include_declaration_true() {
        let source = "id: 1\ndata:\n  id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // With include_declaration=true
        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Should include both
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_include_declaration_false() {
        let source = "id: 1\ndata:\n  id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // With include_declaration=false
        let refs = find_references_at_position(&ast, source, 0, 0, false);

        // Should exclude the one at cursor (line 0)
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].span.start.line, 2);
    }

    #[test]
    fn test_include_declaration_false_from_second_ref() {
        let source = "id: 1\ndata:\n  id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on second "id" (line 2, col 2)
        let refs = find_references_at_position(&ast, source, 2, 2, false);

        // Should exclude the one at cursor (line 2)
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].span.start.line, 0);
    }
}

/// Test with nested structures
mod nested_references {
    use super::*;

    #[test]
    fn test_references_in_nested_objects() {
        let source = "data:\n  id: 1\n  nested:\n    id: 2";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "id"
        let refs = find_references_at_position(&ast, source, 1, 2, true);

        // Should find both "id" keys at different nesting levels
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].span.start.line, 1);
        assert_eq!(refs[1].span.start.line, 3);
    }

    #[test]
    fn test_references_all_marked_as_definitions() {
        let source = "name: Alice\nuser:\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Currently all references are marked as definitions
        assert!(refs[0].is_definition);
        assert!(refs[1].is_definition);
    }
}

/// Test reference sorting
mod reference_sorting {
    use super::*;

    #[test]
    fn test_references_sorted_by_position() {
        let source = "name: C\ndata:\n  name: B\nname: A";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let refs = find_references_at_position(&ast, source, 0, 0, true);

        // Should be sorted by line number
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].span.start.line, 0);
        assert_eq!(refs[1].span.start.line, 2);
        assert_eq!(refs[2].span.start.line, 3);
    }
}
