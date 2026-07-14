# Security Policy

## Reporting a Vulnerability

Please report security vulnerabilities privately rather than opening a public issue.

- **Email**: w0wl0lxd@tuta.com
- **GitHub**: [Private vulnerability report](https://github.com/w0wl0lxd/toon-lsp/security/advisories/new)

Include a description of the vulnerability, steps to reproduce, and its potential
impact. You should receive an initial response within a few business days.

## Scope

Security reports of particular interest for `toon-lsp`:

- Parser/scanner input that causes a panic, infinite loop, or excessive
  memory/CPU use on untrusted TOON, JSON, or YAML input.
- The LSP server behaving unsafely on malformed or adversarial documents
  (e.g. path traversal via `${env:...}`/`${path}` reference resolution).
- Any encode/decode round-trip that silently corrupts data rather than
  erroring.

## Supported Versions

Only the latest published release (`0.6.x`) is supported with security fixes.
