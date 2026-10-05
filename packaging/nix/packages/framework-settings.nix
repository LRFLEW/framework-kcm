{
  lib,
  symlinkJoin,

  version,
  framework-gui,
  framework-kcm,
  frameworkd,
}:

symlinkJoin {
  name = "framework-settings-${version}";

  paths = [
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
