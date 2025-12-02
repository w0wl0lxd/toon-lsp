# `toon-lsp`

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0--only-blue.svg)](LICENSE)
[![Commercial License](https://img.shields.io/badge/License-Commercial-green.svg)](LICENSING.md)

A Language Server Protocol (LSP) implementation for [TOON](https://github.com/toon-format/toon) `(Token-Oriented Object Notation)`.

## Overview

`TOON` is a compact, human-readable encoding of the JSON data model designed for LLM prompts. This project provides:

- **Full AST with source positions** - Parse TOON into an abstract syntax tree with span information
- **LSP Server** - Complete language server for IDE integration with 9 LSP features
- **Error recovery** - Partial parsing for better IDE experience
- **Comprehensive tests** - 248 tests covering scanner, parser, and all LSP features

## Features

### Parser
- [x] Lexer/Scanner with position tracking
- [x] Full TOON spec parser (objects, arrays, primitives)
- [x] Expanded arrays (dash-prefixed items)
- [x] Inline arrays (`key[count]: val1,val2,val3`)
- [x] Tabular arrays (`key[count]{col1,col2}:`)
- [x] Error recovery for partial documents
- [x] AST with complete span information

### LSP Features
- [x] Diagnostics (syntax errors with recovery)
- [x] Document symbols (outline with hierarchy)
- [x] Hover information (type and path display)
- [x] Go to definition (duplicate key navigation)
- [x] Completions (sibling keys, boolean values)
- [x] Semantic tokens (syntax highlighting)
- [x] Find references (key usage navigation)
- [x] Rename symbol (refactor keys across document)
- [x] Formatting (configurable indentation)

## Installation

```bash
cargo install toon-lsp
```

Or build from source:

```bash
git clone https://github.com/w0wl0lxd/toon-lsp
cd toon-lsp
cargo build --release
```

## Usage

### As an LSP Server

The binary communicates over stdio:

```bash
toon-lsp
```

### As a Library

```rust
use toon_lsp::{parse, AstNode, ObjectEntry, ParseError};

fn main() -> Result<(), ParseError> {
    let source = "user:\n  name: Alice\n  age: 30\n  roles[2]:\n    - admin\n    - developer";

    let ast = parse(source)?;

    // Every AST node carries source positions (Span) - 0-indexed
    let AstNode::Document { children, span } = &ast else {
        return Ok(());
    };
    println!("Document: lines {}-{}", span.start.line + 1, span.end.line + 1);

    // Walk the AST - objects contain key-value entries
    for node in children {
        if let AstNode::Object { entries, .. } = node {
            print_entries(entries, 0);
        }
    }
    Ok(())
}

fn print_entries(entries: &[ObjectEntry], depth: usize) {
    let indent = "  ".repeat(depth);
    for entry in entries {
        let loc = &entry.key_span.start;
        println!("{}{}: (line {}, col {})", indent, entry.key, loc.line + 1, loc.column + 1);

        // Recursively handle nested objects
        if let AstNode::Object { entries: nested, .. } = &entry.value {
            print_entries(nested, depth + 1);
        }
    }
}
```

**Error recovery for IDEs** — parse succeeds even with syntax errors:

```rust
use toon_lsp::parse_with_errors;

let (ast, errors) = parse_with_errors(source);

// Partial AST available even with errors (for IDE features)
if let Some(ast) = ast {
    // Provide completions, symbols, hover despite errors
}

// Errors have spans for diagnostic squiggles
for err in &errors {
    eprintln!("{}:{}: {}", err.span.start.line, err.span.start.column, err.kind);
}
```

## Architecture

```
Scanner ──▶ Parser ──▶ AST ──▶ LSP Server
(Lexer)              (Spans)   (tower-lsp)
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
               Diagnostics    Symbols      Semantic Tokens
               Hover          References   Formatting
               Completion     Rename       Go-to-Definition
```

## Development

```bash
cargo build                      # Build the project
cargo test                       # Run all 248 tests
cargo clippy -- -D warnings      # Lint with warnings as errors
cargo fmt --check                # Check formatting
RUST_LOG=debug cargo run         # Run LSP server with debug logging
```

## Related Projects

- [toon-format/toon](https://github.com/toon-format/toon) - Official TOON specification
- [toon-format/toon-rust](https://github.com/toon-format/toon-rust) - Rust implementation (serde-based)
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - LSP framework used by this project

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

All contributions require a DCO sign-off (`git commit -s`).

## License

**Dual Licensed**: AGPL-3.0-only OR Commercial

- **Open Source**: [AGPL-3.0](LICENSE) - Free for open source and personal use
- **Commercial**: Available for proprietary use - See [LICENSING.md](LICENSING.md)

| Use Case | License |
|----------|---------|
| Personal/internal development | Free (AGPL) |
| Open source project (AGPL-compatible) | Free (AGPL) |
| Proprietary IDE/editor embedding | Commercial required |
| Cloud IDE / SaaS platform | Commercial required |

Contact: w0wl0lxd@tuta.com | [GitHub Discussions](https://github.com/w0wl0lxd/toon-lsp/discussions)
