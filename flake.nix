{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f (import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        }));
  in {
    devShells = forEachSystem (pkgs: {
      default = pkgs.mkShell {
        packages = with pkgs; [
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          cargo-llvm-cov
          just
        ];

        buildInputs = with pkgs; [
          libheif
        ];
      };
    });

    packages = forEachSystem (pkgs: rec {
      timewall = pkgs.callPackage ./package.nix {};
      default = timewall;
    });
  };
}
