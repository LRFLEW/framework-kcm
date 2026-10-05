{
  cmake,
  kdePackages,
  lib,
  stdenv,

  version,
  frameworkd,
}:

stdenv.mkDerivation {
  pname = "framework-kcm";
  inherit version;
  src = ../../..;
  dontWrapQtApps = true;

  __structuredAttrs = true;
  strictDeps = true;

  nativeBuildInputs = [
    cmake
    kdePackages.extra-cmake-modules
  ];

  buildInputs = with kdePackages; [
    qtbase
    qtdeclarative

    kcoreaddons
    ki18n
    kcmutils
  ];

  cmakeFlags = [
    (lib.cmakeBool "BUILD_DAEMON" false)
    (lib.cmakeFeature "DAEMON_PATH" (lib.getExe frameworkd))
    (lib.cmakeBool "KDE_INSTALL_USE_QT_SYS_PATHS" true)
  ];

  meta = {
    description = "Framework configuration in KDE settings";
    homepage = "https://github.com/flamingspaz/framework-kcm";
    license = lib.licenses.gpl3Plus;
    platforms = lib.platforms.linux;
  };
}
