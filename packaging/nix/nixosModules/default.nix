rec {
  framework-kcm = import ./framework-kcm.nix;
  framework-gui = import ./framework-gui.nix;
  frameworkd = import ./frameworkd.nix;

  default = framework-kcm;
}
