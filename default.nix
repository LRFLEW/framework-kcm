{ pkgs ? import <nixpkgs> { } }:
{
  packages = import ./packaging/nix/packages { inherit pkgs; };
  nixosModules = import ./packaging/nix/nixosModules;
}
