{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.moxidle;
in
{
  options.services.moxidle = {
    enable = lib.mkEnableOption "moxidle, feature rich idle daemon";
    package = lib.mkPackageOption pkgs "moxidle" { };
    settings = lib.mkOption {
      type = lib.types.attrs;
      default = { };
      description = "Configuration for moxidle";
    };
  };

  config = lib.mkIf cfg.enable {
    xdg.configFile = {
      "mox/moxidle/default.nix" = lib.mkIf (cfg.settings != { }) {
        text = lib.generators.toPretty { } cfg.settings;
      };
    };

    systemd.user.services.moxidle = {
      Install = {
        WantedBy = [ config.wayland.systemd.target ];
      };
      Unit = {
        Description = "moxidle idle manager";
        PartOf = [ config.wayland.systemd.target ];
        After = [ config.wayland.systemd.target ];
        ConditionEnvironment = "WAYLAND_DISPLAY";
        X-Restart-Triggers = [
          (lib.mkIf (cfg.settings != { }) config.xdg.configFile."mox/moxidle/default.nix".source)
        ];
      };
      Service = {
        ExecStart = "${lib.getExe cfg.package}";
        Restart = "always";
        RestartSec = "10";
      };
    };

    home.packages = [ cfg.package ];
  };
}
