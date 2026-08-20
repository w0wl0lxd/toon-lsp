# TOON — VS Code

Install: search "TOON Language" in the Marketplace, or run `code --install-extension toon-lang.toon-lsp`. You can also install the VSIX from [Releases](https://github.com/w0wl0lxd/toon-lsp/releases).
The extension bundles toon-lsp. You need no PATH setup.

Configure only if you use a custom binary:
```json
{ "toon-lsp.path": "/path/to/toon-lsp" }
```

Verify: open a `.toon` file and check that diagnostics appear.
See [IDE support](../../docs/ide-support.md) for more.
