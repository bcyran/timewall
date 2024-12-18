{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
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
    rev = self.shortRev or self.dirtyShortRev or "dirty";
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

    overlays.default = final: prev: {
      timewall = final.callPackage ./nix/package.nix {inherit rev;};
    };

    packages = forEachSystem (pkgs: rec {
      timewall = pkgs.callPackage ./nix/package.nix {inherit rev;};
      default = timewall;
    });

    homeManagerModules = rec {
      timewall = import ./nix/hm-module.nix self;
      default = timewall;
    };

    formatter = forEachSystem (pkgs: pkgs.alejandra);
  };
}
