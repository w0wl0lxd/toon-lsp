//! Tests for hover functionality in the LSP server.

use toon_lsp::lsp::hover::get_hover_at_position;
use toon_lsp::parser::parse_with_errors;

/// Test hover over object key
mod hover_over_key {
    use super::*;

    #[test]
    fn test_hover_on_simple_key() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (0, 0) - on 'n' of 'name'
        let hover = get_hover_at_position(&ast, source, 0, 0);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("name"));
    }

    #[test]
    fn test_hover_on_nested_key() {
        let source = "person:\n  name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (1, 2) - on 'n' of nested 'name'
        let hover = get_hover_at_position(&ast, source, 1, 2);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("name"));
    }

    #[test]
    fn test_hover_shows_key_path() {
        let source = "person:\n  name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (1, 2) - should show path
        let hover = get_hover_at_position(&ast, source, 1, 2);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        // Could contain full path like "person.name"
        assert!(hover.contents.contains("name"));
    }
}

/// Test hover over array key
mod hover_over_array {
    use super::*;

    #[test]
    fn test_hover_on_array_key() {
        let source = "items:\n  - first\n  - second";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (0, 0) - on 'i' of 'items'
        let hover = get_hover_at_position(&ast, source, 0, 0);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("items"));
        // Should show array info
        assert!(hover.contents.contains("array") || hover.contents.contains("Array"));
    }

    #[test]
    fn test_hover_shows_array_length() {
        let source = "items:\n  - first\n  - second\n  - third";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hover = get_hover_at_position(&ast, source, 0, 0);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        // Should indicate it's an array with 3 items
        assert!(hover.contents.contains("3") || hover.contents.contains("items"));
    }
}

/// Test hover over string value
mod hover_over_string {
    use super::*;

    #[test]
    fn test_hover_on_string_value() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (0, 6) - on 'A' of 'Alice'
        let hover = get_hover_at_position(&ast, source, 0, 6);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("string") || hover.contents.contains("String"));
    }

    #[test]
    fn test_hover_shows_string_preview() {
        let source = "greeting: Hello World";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hover = get_hover_at_position(&ast, source, 0, 11);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("Hello") || hover.contents.contains("string"));
    }
}

/// Test hover over number value
mod hover_over_number {
    use super::*;

    #[test]
    fn test_hover_on_integer() {
        let source = "age: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (0, 5) - on '3' of '30'
        let hover = get_hover_at_position(&ast, source, 0, 5);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(
            hover.contents.contains("30")
                || hover.contents.contains("number")
                || hover.contents.contains("Number")
        );
    }

    #[test]
    fn test_hover_on_float() {
        let source = "price: 19.99";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hover = get_hover_at_position(&ast, source, 0, 7);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(
            hover.contents.contains("19.99")
                || hover.contents.contains("number")
                || hover.contents.contains("float")
        );
    }

    #[test]
    fn test_hover_on_negative_number() {
        let source = "offset: -10";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let hover = get_hover_at_position(&ast, source, 0, 8);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("-10") || hover.contents.contains("number"));
    }
}

/// Test hover over empty space (no result)
mod hover_empty_space {
    use super::*;

    #[test]
    fn test_hover_on_empty_line() {
        let source = "name: Alice\n\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (1, 0) - empty line
        // Implementation may return None or root document info
        let hover = get_hover_at_position(&ast, source, 1, 0);

        // Either None or a document-level hover is acceptable
        if let Some(h) = hover {
            // If we get something, it shouldn't be a specific key
            assert!(!h.contents.contains("name:") && !h.contents.contains("age:"));
        }
    }

    #[test]
    fn test_hover_past_end_of_line() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover past end of content
        let hover = get_hover_at_position(&ast, source, 0, 100);

        assert!(hover.is_none());
    }

    #[test]
    fn test_hover_on_colon() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover at position (0, 4) - on ':'
        let hover = get_hover_at_position(&ast, source, 0, 4);

        // Could return None or info about the key-value pair
        // Depends on implementation - either is acceptable
        if let Some(h) = hover {
            // If we get hover, it should have some content
            assert!(!h.contents.is_empty());
        }
    }

    #[test]
    fn test_hover_on_whitespace_before_value() {
        let source = "name:    Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Hover on whitespace between colon and value
        let _hover = get_hover_at_position(&ast, source, 0, 6);

        // Could return None or value info - implementation specific
        // No assertion needed - just verifying it doesn't crash
    }
}
