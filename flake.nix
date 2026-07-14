{
  description = "toon-lsp - LSP implementation for TOON (Token-Oriented Object Notation)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        # `nix develop` (or direnv's `use flake`) drops you into a shell with
        # `mise` and `rustup` on PATH. Deliberately NOT pinning an rustc/cargo
        # *version* here via nixpkgs: `rust-toolchain.toml` (rustup convention,
        # pinned to `nightly` for this project) is the single source of truth
        # for the Rust version, already used by CI (dtolnay/rust-toolchain)
        # and by contributors without Nix at all. `rustup` just needs to be
        # *present* so its `cargo`/`rustc` proxies can read that file and
        # lazily install the pinned nightly on first use.
        devShells.default = pkgs.mkShellNoCC {
          packages = [
            pkgs.mise
            pkgs.rustup
            pkgs.pkg-config
          ];

          shellHook = ''
            eval "$(mise activate bash)"
          '';
        };
      }
    );
}
