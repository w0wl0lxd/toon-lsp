# TOON — Eclipse

TOON support in Eclipse uses `toon-lsp` through LSP4E.

## Prerequisites

* Eclipse 2022-03 or later.
* LSP4E from the Eclipse Marketplace.
* `toon-lsp` on your `PATH`. Install it with `cargo install toon-lsp`.

## Setup

LSP4E binds a language server to a *content type*, so you create the content type
first and point LSP4E at it second.

1. Install LSP4E from Help → Eclipse Marketplace.
2. Open Window → Preferences → General → Content Types. Click **Add Root…** and name
   the new type `TOON File`. With it selected, click **Add…** under File Associations
   and enter `*.toon`.
3. Open Window → Preferences → Language Servers and add a language server. Set the
   program to `toon-lsp` and the content type to the **TOON File** type from step 2.
   Do not enter `toon` or `.toon` here: `toon` is the LSP language id and `.toon` is a
   file suffix, and neither is a content type.
4. Restart Eclipse and open a `.toon` file.

## About `plugin.xml`

`plugin.xml` in this directory is a reference for a packaged Eclipse plugin that would
register the same content type as `org.toon.contenttype`. It is not a buildable plugin
project on its own — there is no manifest, no build files and no published JAR — so do
not expect installing LSP4E to provide `TOON File`. Step 2 is the supported route.

## More info

See [IDE support](../../docs/ide-support.md) for all features and usage.
