# toon-lsp

A Language Server Protocol (LSP) implementation for [TOON](https://github.com/toon-format/toon) (Token-Oriented Object Notation).

## Overview

TOON is a compact, human-readable encoding of the JSON data model designed for LLM prompts. This project provides:

- **Full AST with source positions** - Parse TOON into an abstract syntax tree with span information
- **LSP Server** - Complete language server for IDE integration
- **Error recovery** - Partial parsing for better IDE experience

## Features

### Parser
- [x] Lexer/Scanner with position tracking
- [ ] Full TOON spec parser
- [ ] Error recovery for partial documents
- [ ] AST with complete span information

### LSP Features
- [ ] Diagnostics (syntax errors)
- [ ] Document symbols (outline)
- [ ] Hover information
- [ ] Go to definition
- [ ] Completions
- [ ] Formatting

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
use toon_lsp::{parse, AstNode};

let source = r#"
name: Alice
age: 30
tags:
  - developer
  - rust
"#;

let ast = parse(source)?;
println!("Parsed: {:?}", ast);
```

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Scanner   │ ──▶ │   Parser    │ ──▶ │     AST     │
│  (Lexer)    │     │             │     │ (with Spans)│
└─────────────┘     └─────────────┘     └─────────────┘
                                               │
                                               ▼
                                        ┌─────────────┐
                                        │ LSP Server  │
                                        │ (tower-lsp) │
                                        └─────────────┘
```

## Related Projects

- [toon-format/toon](https://github.com/toon-format/toon) - Official TOON specification
- [toon-format/toon-rust](https://github.com/toon-format/toon-rust) - Rust implementation (serde-based)
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - LSP framework used by this project

## License

MIT
