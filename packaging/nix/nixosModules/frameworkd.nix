{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.frameworkd;
  packages = import ../packages { inherit pkgs; };
in
{
  options.services.frameworkd = {
    enable = lib.mkEnableOption "Framework laptop hardware service";
    package = lib.mkPackageOption packages "frameworkd" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.dbus.packages = [ cfg.package ];
    security.polkit.enable = true;
    systemd.packages = [ cfg.package ];
    # Reapply write-only hardware settings after reboot.
    systemd.services.frameworkd.wantedBy = [ "multi-user.target" ];
  };
}
