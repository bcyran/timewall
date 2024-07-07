{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain = let
            rust = prev.rust-bin;
          in
            if builtins.pathExists ./rust-toolchain.toml
            then rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain
            then rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default.override {
                extensions = ["rust-src" "rustfmt"];
              };
        })
      ];
      pkgs = import nixpkgs {
        inherit overlays system;
      };
    in
      with pkgs; {
        devShells.default = mkShell {
          packages = [
            (rustToolchain.override {extensions = ["llvm-tools-preview"];})
            cargo-edit
            cargo-watch
            cargo-llvm-cov
            rust-analyzer
            just
          ];
          buildInputs = [
            libheif
          ];
        };
      });
}
