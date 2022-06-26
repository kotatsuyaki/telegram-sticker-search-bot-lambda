{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, utils, fenix, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        target = "aarch64-unknown-linux-gnu";
        toolchain = with fenix.packages.${system};
          combine [
            minimal.rustc
            minimal.cargo
            targets.${target}.latest.rust-std
          ];
        rust-src = with fenix.packages.${system};
          complete.rust-src;
      in
      {
        devShell = with pkgs; mkShell {
          RUST_SRC_PATH = "${rust-src}/lib/rustlib/src/rust/library";
          buildInputs = [
            toolchain
            rustfmt
            zig

            fenix.packages.${system}.rust-analyzer
            rnix-lsp
            yaml-language-server
            taplo-cli
            nodePackages.vscode-langservers-extracted
          ];
        };
      });
}
