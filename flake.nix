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

    # libheif 1.19.7 is not in nixpkgs-unstable yet
    libheif-overlay = final: prev: {
      libheif = prev.libheif.overrideAttrs (_self: _super: rec {
        version = "1.19.7";
        src = final.fetchFromGitHub {
          owner = "strukturag";
          repo = "libheif";
          rev = "v${version}";
          hash = "sha256-FXq6AOq1tUM05++fkzowApbLnlgeS5ZJ+UmypHrF11g=";
        };
      });
    };

    forEachSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f (import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            libheif-overlay
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

        nativeBuildInputs = with pkgs; [
          pkg-config
          rustPlatform.bindgenHook
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
