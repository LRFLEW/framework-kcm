{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.framework-gui;
  packages = import ../packages { inherit pkgs; };
in
{
  imports = [ ./frameworkd.nix ];

  options.programs.framework-gui = {
    enable = lib.mkEnableOption "standalone Framework laptop settings application";
    package = lib.mkPackageOption packages "framework-gui" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.frameworkd.enable = true;
  };
}
