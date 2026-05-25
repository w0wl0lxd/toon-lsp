# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2] - 2025-05-25

### Added
- **Workspace symbol search**: `workspace/symbol` handler with query filtering across all open documents
- **Code actions**: `textDocument/codeAction` with extract-to-variable support
- **Code lens**: `textDocument/codeLens` showing reference counts on keys
- **Document highlights**: `textDocument/documentHighlight` for occurrence highlighting
- **Document links**: `textDocument/documentLink` for URL detection in document text
- **Folding ranges**: `textDocument/foldingRange` for code folding support
- **Inlay hints**: `textDocument/inlayHint` for type/value annotations
- **Linked editing ranges**: `textDocument/linkedEditingRange` for simultaneous key editing
- **Selection ranges**: `textDocument/selectionRange` for smart selection expansion
- All new LSP capabilities declared in server capabilities negotiation

### Changed
- Refactored codebase for idiomatic Rust patterns:
  - AST: added `#[inline]` attributes, `const fn` methods, `Position::ZERO` constant
  - Parser: eliminated `.clone()` on tokens, used iterator combinators, simplified control flow
  - Scanner: `&str` slices instead of `String`, `#[inline]` on hot paths
  - LSP server: `with_ast` helper to flatten deeply nested `if let` chains, `DocRef` type alias
  - Replaced manual loops with iterator combinators across LSP modules

### Fixed
- `Span::merge` producing inconsistent `Position` values by independently computing min/max of line, column, and offset fields — now uses offset-based position selection
- `parse_unquoted_string` inserting unwanted spaces before colons and commas in unquoted values
- Test comment syntax corrections in `lsp_capabilities` and other test modules

### Dependencies
- Bump `bytes` from 1.11.0 to 1.11.1 (dependabot #4)

## [0.3.1] - 2025-05-24

### Changed
- Added Mermaid architecture diagram and modernized code examples in README
- Added CI, release, and Rust version badges

## [0.3.0] - 2025-05-23

### Added
- CLI schema support with 6 commands: check, convert, decode, diagnose, encode, format
- Pre-commit hooks for auto-formatting and clippy
- Tabular array syntax support
- Inline array syntax support
- Expanded security tests for resource limits

### Changed
- Improved error recovery in parser for IDE use
- Fixed CI warnings and applied formatting
- Removed tabs support per TOON spec

### Fixed
- Resolved CI failures for deprecated API and dependency policy

## [0.2.0] - 2025-12-15

### Added
- Snapshot testing with insta for parser and scanner output validation
- Property-based testing with proptest
- CLI integration tests with assert_cmd

### Changed
- LSP capabilities: expanded semantic tokens, improved completion context detection
- Extended parser test coverage to 27 tests with edge cases

## [0.1.1] - 2025-12-01

### Changed
- Improved library examples with idiomatic Rust patterns
- Fixed CI workflow for cargo-deny action

## [0.1.0] - 2025-12-01

### Added
- Full TOON parser with position tracking (Span on all AST nodes)
- Scanner/lexer with 45 tests
- Recursive descent parser with 27 tests
- Security limits: maximum nesting depth (128), document size (10MB), array items (100k), object entries (10k)
- Error types for security limits: `MaxDepthExceeded`, `DocumentTooLarge`, `TooManyArrayItems`, `TooManyObjectEntries`
- `#[must_use]` attributes on pure functions in AST module
- `const fn` for `AstNode::kind()` method
- Async parsing via `spawn_blocking` for better LSP responsiveness
- LSP server with 9 features:
  - Real-time diagnostics
  - Document symbols (outline)
  - Hover information
  - Smart completions
  - Go-to-definition for duplicate keys
  - Semantic tokens (syntax highlighting)
  - Find references
  - Rename symbol
  - Document formatting
- Error recovery for IDE use (`parse_with_errors`)
- All array forms: expanded, inline, tabular
- UTF-16/UTF-8 position conversion for LSP compliance
- 247+ tests with snapshot testing (insta)

### Technical
- Built with tower-lsp 0.20
- Async runtime: tokio
- Zero unsafe code
- AGPL-3.0-only license with commercial licensing available

[unreleased]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.3.2...HEAD
[0.3.2]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/w0wl0lxd/toon-lsp/releases/tag/v0.1.0
