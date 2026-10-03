{ pkgs ? import <nixpkgs> { } }:

let
  version = "0.1.1";

  framework-kcmd = import ./framework-kcmd.nix { inherit pkgs version; };

  framework-kcm = import ./framework-kcm.nix {
    inherit pkgs version framework-kcmd;
  };
in
{
  default = framework-kcm;
  inherit framework-kcm framework-kcmd;
}
