{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.framework-kcm;
  packages = import ../packages { inherit pkgs; };
in
{
  options.programs.framework-kcm = {
    enable = lib.mkEnableOption "framework-kcm";
    package = lib.mkPackageOption packages "framework-kcm" { };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.dbus.packages = [ cfg.package ];
    systemd.packages = [ cfg.package ];
    # start service at boot
    systemd.services.frameworkd.wantedBy = [ "multi-user.target" ];
  };
}
