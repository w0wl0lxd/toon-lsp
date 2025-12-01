//! Tests for go-to-definition functionality in the LSP server.

use toon_lsp::lsp::goto::get_definition_at_position;
use toon_lsp::parser::parse_with_errors;

/// Test goto with unique key
mod unique_key {
    use super::*;

    #[test]
    fn test_goto_unique_key_returns_single_location() {
        let source = "name: Alice\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "name" key
        let locations = get_definition_at_position(&ast, source, 0, 2);

        // Should return exactly one location for unique key
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].line, 0);
    }

    #[test]
    fn test_goto_unique_key_correct_range() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let locations = get_definition_at_position(&ast, source, 0, 0);

        assert_eq!(locations.len(), 1);
        // Range should cover "name"
        assert_eq!(locations[0].start_col, 0);
        assert_eq!(locations[0].end_col, 4); // "name" is 4 chars
    }
}

/// Test goto with duplicate keys
mod duplicate_keys {
    use super::*;

    #[test]
    fn test_goto_duplicate_keys_returns_all_locations() {
        // TOON allows duplicate keys (like JSON)
        let source = "name: Alice\nname: Bob\nname: Charlie";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on first "name" key
        let locations = get_definition_at_position(&ast, source, 0, 2);

        // Should return all three locations
        assert_eq!(locations.len(), 3);
    }

    #[test]
    fn test_goto_duplicate_keys_correct_lines() {
        let source = "name: Alice\nname: Bob\nname: Charlie";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let locations = get_definition_at_position(&ast, source, 0, 2);

        // Check all lines are represented
        let lines: Vec<u32> = locations.iter().map(|l| l.line).collect();
        assert!(lines.contains(&0));
        assert!(lines.contains(&1));
        assert!(lines.contains(&2));
    }

    #[test]
    fn test_goto_from_any_duplicate_finds_all() {
        let source = "name: Alice\nname: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on second "name" key (line 1)
        let locations = get_definition_at_position(&ast, source, 1, 2);

        // Should still find both
        assert_eq!(locations.len(), 2);
    }
}

/// Test goto with no key at position
mod no_key_at_position {
    use super::*;

    #[test]
    fn test_goto_on_value_returns_empty() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on "Alice" value
        let locations = get_definition_at_position(&ast, source, 0, 7);

        // No definition for values
        assert!(locations.is_empty());
    }

    #[test]
    fn test_goto_on_empty_space_returns_empty() {
        let source = "name: Alice\n\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on empty line
        let locations = get_definition_at_position(&ast, source, 1, 0);

        assert!(locations.is_empty());
    }

    #[test]
    fn test_goto_past_end_of_document() {
        let source = "name: Alice";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position past end
        let locations = get_definition_at_position(&ast, source, 10, 0);

        assert!(locations.is_empty());
    }

    #[test]
    fn test_goto_in_empty_document() {
        let source = "";
        let (ast, _) = parse_with_errors(source);

        if let Some(ast) = ast {
            let locations = get_definition_at_position(&ast, source, 0, 0);
            assert!(locations.is_empty());
        }
    }
}

/// Additional edge cases
mod edge_cases {
    use super::*;

    #[test]
    fn test_goto_nested_object_key() {
        let source = "person:\n  name: Alice\n  name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on nested "name" key
        let locations = get_definition_at_position(&ast, source, 1, 3);

        // Should find both "name" keys within nested object
        assert_eq!(locations.len(), 2);
    }

    #[test]
    fn test_goto_does_not_cross_scope() {
        let source = "person:\n  name: Alice\ncompany:\n  name: Acme";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        // Position on person's "name" key
        let locations = get_definition_at_position(&ast, source, 1, 3);

        // Should only find the "name" in person, not company
        // (definitions are scoped to containing object)
        assert_eq!(locations.len(), 1);
    }
}
