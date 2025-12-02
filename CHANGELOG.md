# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/w0wl0lxd/toon-lsp/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/w0wl0lxd/toon-lsp/releases/tag/v0.1.0
