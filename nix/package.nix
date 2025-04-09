{
  lib,
  rustPlatform,
  installShellFiles,
  pkg-config,
  libheif,
  rev ? "dirty",
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
  cargoVersion = cargoToml.package.version;
in
  rustPlatform.buildRustPackage rec {
    pname = "timewall";
    version = "${cargoVersion}-${rev}";

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

    nativeBuildInputs = [
      installShellFiles
      pkg-config
    ];

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
      changelog = "https://github.com/bcyran/timewall/releases/tag/${cargoVersion}";
      license = lib.licenses.mit;
      mainProgram = "timewall";
      maintainers = with lib.maintainers; [bcyran];
    };
  }
