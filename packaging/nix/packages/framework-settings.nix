{
  lib,
  stdenvNoCC,

  version,
  framework-gui,
  framework-kcm,
  frameworkd,
}:

stdenvNoCC.mkDerivation {
  pname = "framework-settings";
  inherit version;

  __structuredAttrs = true;
  strictDeps = true;

  dontUnpack = true;
  dontBuild = true;

  installPhase = ''
    mkdir -p "$out"
  '';

  propagatedBuildInputs = [
    framework-gui
    framework-kcm
    frameworkd
  ];

  meta = {
    description = "Framework Laptop settings GUI, KDE module, and system service";
    homepage = "https://github.com/flamingspaz/framework-settings";
    license = lib.licenses.gpl3Plus;
    platforms = lib.platforms.linux;
  };
}
