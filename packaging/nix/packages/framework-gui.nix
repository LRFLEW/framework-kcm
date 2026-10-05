{
  cmake,
  lib,
  qt6,
  stdenv,

  version,
  frameworkd,
}:

stdenv.mkDerivation {
  pname = "framework-gui";
  inherit version;
  src = ../../..;

  __structuredAttrs = true;
  strictDeps = true;

  nativeBuildInputs = [
    cmake
    qt6.wrapQtAppsHook
  ];

  buildInputs = with qt6; [
    qtbase
    qtdeclarative

  ];

  cmakeFlags = [
    (lib.cmakeBool "BUILD_KCM" false)
    (lib.cmakeBool "BUILD_DAEMON" false)
    (lib.cmakeFeature "DAEMON_PATH" (lib.getExe frameworkd))
  ];

  installPhase = ''
    runHook preInstall
    cmake --install . --component gui
    runHook postInstall
  '';

  meta = {
    description = "Standalone Qt settings application for Framework laptops";
    homepage = "https://github.com/flamingspaz/framework-settings";
    license = lib.licenses.gpl3Plus;
    mainProgram = "framework-settings";
    platforms = lib.platforms.linux;
  };
}
