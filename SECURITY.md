# Security Policy

## Reporting a Vulnerability

Do not open a public issue. Report privately:

- **Email**: w0wl0lxd@tuta.com
- **GitHub**: [Private vulnerability report](https://github.com/w0wl0lxd/toon-lsp/security/advisories/new)

Include a description, steps to reproduce, and potential impact. Expect an initial response within a few business days.

## Scope

Reports of particular interest for `toon-lsp`:

- Untrusted TOON, JSON, or YAML input triggers a parser panic, infinite loop, or excessive memory or CPU use.
- Malformed or adversarial documents trigger unsafe LSP behavior such as path traversal via `${env:...}` or `${path}` references.
- Encode or decode round-trips silently corrupt data instead of returning an error.

## Supported Versions

Only the latest published release (0.7.x) receives security fixes.
