# TOON — Eclipse

TOON support in Eclipse uses `toon-lsp` through LSP4E.

## Prerequisites

* Eclipse 2022-03 or later.
* LSP4E from the Eclipse Marketplace.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.

## Setup

Install LSP4E and add the language server. Or install the plugin JAR.

Option 1: LSP4E Marketplace setup:

1. Install LSP4E from Help → Eclipse Marketplace.
2. Open Window → Preferences → Language Servers.
3. Add a language server. Set the program to `toon-lsp` and the content type to `.toon`.
4. Restart Eclipse and open a `.toon` file.

Option 2: Plugin JAR:

1. Download the plugin JAR from [Releases](https://github.com/w0wl0lxd/toon-lsp/releases).
2. Copy the JAR into the `dropins` folder of your Eclipse install.
3. Restart Eclipse.

## More info

See [IDE support](../../docs/ide-support.md) for all features and usage.
