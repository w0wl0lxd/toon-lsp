//! Tests for document symbols functionality in the LSP server.

use toon_lsp::lsp::symbols::ast_to_document_symbols;
use toon_lsp::parser::parse_with_errors;
use tower_lsp::lsp_types::SymbolKind;

/// Test document_symbol with flat object
mod flat_object {
    use super::*;

    #[test]
    fn test_flat_object_symbols() {
        let source = "name: Alice\nage: 30\ncity: Paris";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 3);

        // Check first symbol
        assert_eq!(symbols[0].name, "name");
        assert_eq!(symbols[0].kind, SymbolKind::KEY);
        assert!(symbols[0].children.is_none());

        // Check second symbol
        assert_eq!(symbols[1].name, "age");
        assert_eq!(symbols[1].kind, SymbolKind::KEY);

        // Check third symbol
        assert_eq!(symbols[2].name, "city");
        assert_eq!(symbols[2].kind, SymbolKind::KEY);
    }

    #[test]
    fn test_flat_object_symbol_ranges() {
        let source = "name: Alice\nage: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        // First symbol on line 0
        assert_eq!(symbols[0].range.start.line, 0);
        assert_eq!(symbols[0].selection_range.start.line, 0);

        // Second symbol on line 1
        assert_eq!(symbols[1].range.start.line, 1);
        assert_eq!(symbols[1].selection_range.start.line, 1);
    }

    #[test]
    fn test_single_key_symbol() {
        let source = "key: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "key");
    }
}

/// Test document_symbol with nested objects
mod nested_objects {
    use super::*;

    #[test]
    fn test_nested_object_symbols() {
        // TOON syntax: key followed by colon, then indented children
        let source = "person:\n  name: Alice\n  age: 30";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        // Should have one top-level symbol
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "person");
        assert_eq!(symbols[0].kind, SymbolKind::OBJECT);

        // Should have children
        let children = symbols[0].children.as_ref().expect("should have children");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "name");
        assert_eq!(children[1].name, "age");
    }

    #[test]
    fn test_deeply_nested_symbols() {
        // Three levels of nesting with proper TOON syntax
        let source = "level1:\n  level2:\n    level3: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        // Check nesting
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "level1");

        let level2 = &symbols[0].children.as_ref().unwrap()[0];
        assert_eq!(level2.name, "level2");

        let level3 = &level2.children.as_ref().unwrap()[0];
        assert_eq!(level3.name, "level3");
    }

    #[test]
    fn test_mixed_nesting_levels() {
        // Mix of flat and nested entries
        let source = "flat: value\nnested:\n  child: value\nanother: value";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "flat");
        assert!(symbols[0].children.is_none());

        assert_eq!(symbols[1].name, "nested");
        assert!(symbols[1].children.is_some());

        assert_eq!(symbols[2].name, "another");
        assert!(symbols[2].children.is_none());
    }
}

/// Test document_symbol with arrays
mod arrays {
    use super::*;

    #[test]
    fn test_array_symbol() {
        // TOON expanded array syntax
        let source = "items:\n  - first\n  - second\n  - third";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "items");
        assert_eq!(symbols[0].kind, SymbolKind::ARRAY);
    }

    #[test]
    fn test_array_of_objects() {
        // Array of objects with expanded syntax
        let source = "users:\n  -\n    name: Alice\n  -\n    name: Bob";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "users");
        assert_eq!(symbols[0].kind, SymbolKind::ARRAY);

        // Array children should contain objects with names
        let children = symbols[0].children.as_ref().expect("should have children");
        assert!(children.len() >= 2);
    }

    #[test]
    fn test_inline_array() {
        // TOON inline array syntax: key[count]: value1,value2,value3
        let source = "tags[3]: one,two,three";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse");

        let symbols = ast_to_document_symbols(&ast, source);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "tags");
        assert_eq!(symbols[0].kind, SymbolKind::ARRAY);
    }
}

/// Test document_symbol with empty document
mod empty_document {
    use super::*;

    #[test]
    fn test_empty_document_no_symbols() {
        let source = "";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse empty");

        let symbols = ast_to_document_symbols(&ast, source);

        assert!(symbols.is_empty());
    }

    #[test]
    fn test_whitespace_only_no_symbols() {
        let source = "   \n  \n   ";
        let (ast, _) = parse_with_errors(source);
        let ast = ast.expect("should parse whitespace");

        let symbols = ast_to_document_symbols(&ast, source);

        assert!(symbols.is_empty());
    }

    #[test]
    fn test_comments_only_no_symbols() {
        let source = "# comment\n# another comment";
        let (ast, _) = parse_with_errors(source);

        // Even if parsing fails or produces no AST, symbols should be empty
        if let Some(ast) = ast {
            let symbols = ast_to_document_symbols(&ast, source);
            assert!(symbols.is_empty());
        }
    }
}
