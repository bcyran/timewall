{
  lib,
  rustPlatform,
  installShellFiles,
  libheif,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage rec {
    inherit (cargoToml.package) version;
    pname = "timewall";

    src = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ../.)) (
        lib.fileset.unions [
          ../src
          ../build.rs
          ../Cargo.toml
          ../Cargo.lock
        ]
      );
    };

    cargoLock.lockFile = ../Cargo.lock;

    nativeBuildInputs = [installShellFiles];

    buildInputs = [libheif];

    SHELL_COMPLETIONS_DIR = "completions";

    preBuild = ''
      mkdir ${SHELL_COMPLETIONS_DIR}
    '';

    postInstall = ''
      installShellCompletion \
        --bash ${SHELL_COMPLETIONS_DIR}/timewall.bash \
        --zsh ${SHELL_COMPLETIONS_DIR}/_timewall \
        --fish ${SHELL_COMPLETIONS_DIR}/timewall.fish
    '';

    meta = {
      description = "Apple dynamic HEIF wallpapers on GNU/Linux";
      homepage = "https://github.com/bcyran/timewall";
      changelog = "https://github.com/bcyran/timewall/releases/tag/${version}";
      license = lib.licenses.mit;
      mainProgram = "timewall";
      maintainers = with lib.maintainers; [bcyran];
    };
  }
