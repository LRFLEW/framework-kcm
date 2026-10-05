{ pkgs ? import <nixpkgs> { } }:

let
  version = "0.1.2";
in
rec {
  framework-gui = pkgs.callPackage ./framework-gui.nix { inherit version frameworkd; };
  framework-kcm = pkgs.callPackage ./framework-kcm.nix { inherit version frameworkd; };
  frameworkd = pkgs.callPackage ./frameworkd.nix { inherit version; };
  framework-settings = pkgs.callPackage ./framework-settings.nix {
    inherit version framework-gui framework-kcm frameworkd;
  };

  default = framework-settings;
  framework-kcmd = pkgs.lib.warnOnInstantiate "framework-kcmd has been renamed to frameworkd" frameworkd;
}
