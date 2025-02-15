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
          cache_fallback = lib.mkOption {
            type = lib.types.bool;
            default = true;
            description = ''
              Whether to fallback to the last known location if GeoClue 2 fails to return
              a location.
            '';
          };
          prefer = lib.mkOption {
            type = lib.types.bool;
            default = false;
            description = "Prefer GeoClue 2 over manual location configuration.";
          };
          timeout = lib.mkOption {
            type = lib.types.int;
            default = 1000;
            description = ''
              Time in milliseconds to wait for GeoClue 2 to return a location.
              After this time `timewall` will fallback to manual location configuration
              or fail, depending on the `prefer` option.
            '';
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
          overlap = lib.mkOption {
            type = lib.types.int;
            default = 0;
            description = ''
              Time overlap in milliseconds between spawning a new setter command and
              terminating the previous one. This is useful for long running setters that
              don't terminate immediately after setting the wallpaper.
            '';
          };
          quiet = lib.mkOption {
            type = lib.types.bool;
            default = true;
            description = "Whether to suppress the setter command output.";
          };
        };

        daemon = {
          update_interval_seconds = lib.mkOption {
            type = lib.types.int;
            default = 600;
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
        inherit (cfg.config) daemon;
        inherit (cfg.config) geoclue;
      }
      // lib.optionalAttrs (cfg.config.location.lat != null && cfg.config.location.lon != null) {
        inherit (cfg.config) location;
      }
      // lib.optionalAttrs (cfg.config.setter.command != null) {
        inherit (cfg.config) setter;
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
