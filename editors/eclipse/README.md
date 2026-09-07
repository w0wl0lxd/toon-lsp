# TOON — Eclipse

TOON support in Eclipse uses `toon-lsp` through LSP4E.

## Prerequisites

* Eclipse 2022-03 or later.
* LSP4E from the Eclipse Marketplace.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.

## Setup

Install LSP4E and add the language server:

1. Install LSP4E from Help → Eclipse Marketplace.
2. Open Window → Preferences → Language Servers.
3. Add a language server. Set the program to `toon-lsp`. For the content type, select
   **TOON File** — the content type this plugin registers as `org.toon.contenttype` for
   the `toon` file extension (`plugin.xml`). Do not enter `toon` or `.toon` directly:
   `toon` is the LSP language id and `.toon` is a file suffix, neither of which is a
   content type.
4. Restart Eclipse and open a `.toon` file.

There is no prebuilt plugin JAR. No release workflow builds or publishes one, so install
through LSP4E as above, or build the plugin from `plugin.xml` yourself.

## More info

See [IDE support](../../docs/ide-support.md) for all features and usage.
