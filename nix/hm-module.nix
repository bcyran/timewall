self: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (pkgs.stdenv.hostPlatform) system;
  cfg = config.services.timewall;
  package = self.packages.${system}.default;
  configFormat = pkgs.formats.toml {};
in {
  options = {
    services.timewall = {
      enable = lib.mkEnableOption "timewall";

      package = lib.mkPackageOption self.packages.${system} "timewall" {
        default = "default";
        pkgsText = "timewall.packages.\${pkgs.system}";
      };

      systemdTarget = lib.mkOption {
        type = lib.types.str;
        default = "graphical-session.target";
        description = "Systemd target to bind to.";
      };

      wallpaperPath = lib.mkOption {
        type = with lib.types; nullOr path;
        default = null;
        description = ''
          Path to the HEIF/HEIC dynamic wallpaper file to set.
          If not set, you need to manually set the wallpaper by running `timewall set`!
          Otherwise the timewall daemon will fail.
        '';
      };

      config = {
        geoclue = {
          enable = lib.mkOption {
            type = lib.types.bool;
            default = true;
            description = "Enable GeoClue 2 for automatic location detection.";
          };
          prefer = lib.mkOption {
            type = lib.types.bool;
            default = false;
            description = "Prefer GeoClue 2 over manual location configuration.";
          };
        };

        location = {
          lat = lib.mkOption {
            type = with lib.types; nullOr (either float int);
            default = null;
            description = "Your geographical latitude.";
          };
          lon = lib.mkOption {
            type = with lib.types; nullOr (either float int);
            default = null;
            description = "You geographical longitude.";
          };
        };

        setter = {
          command = lib.mkOption {
            type = with lib.types; nullOr (listOf str);
            default = null;
            description = ''
              Command to set the wallpaper. Use "%f" as a placeholder for the file path.
              The command is NOT passed through a shell.
            '';
            example = ["sww" "img" "%f"];
          };
        };

        daemon = {
          update_interval_seconds = lib.mkOption {
            type = with lib.types; nullOr int;
            default = null;
            description = "Interval between wallpaper updates in seconds.";
          };
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = (cfg.config.location.lat != null) == (cfg.config.location.lon != null);
        message = "Both `latitude and `longitude` must be set for timewall";
      }
    ];

    home.packages = [cfg.package];

    xdg.configFile."timewall/config.toml".source = configFormat.generate "config.toml" (
      {
        inherit (cfg.config) geoclue;
      }
      // lib.optionalAttrs (cfg.config.location.lat != null && cfg.config.location.lon != null) {
        inherit (cfg.config) location;
      }
      // lib.optionalAttrs (cfg.config.setter.command != null) {
        inherit (cfg.config) setter;
      }
      // lib.optionalAttrs (cfg.config.daemon.update_interval_seconds != null) {
        inherit (cfg.config) daemon;
      }
    );

    systemd.user.services.timewall = {
      Unit = {
        Description = "Dynamic wallpapers daemon";
        PartOf = ["graphical-session.target"];
        X-Restart-Triggers =
          lib.mkIf (cfg.config != {})
          ["${config.xdg.configFile."timewall/config.toml".source}"];
      };
      Service = {
        ExecStart = builtins.concatStringsSep " " [
          "${lib.getExe package} set --daemon"
          (
            if cfg.wallpaperPath != null
            then builtins.toString cfg.wallpaperPath
            else ""
          )
        ];
        Restart = "on-failure";
      };
      Install.WantedBy = [cfg.systemdTarget];
    };
  };
}
