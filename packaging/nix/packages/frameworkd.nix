{
  lib,
  pkg-config,
  rustPlatform,
  udev,

  version,
}:

rustPlatform.buildRustPackage {
  pname = "frameworkd";
  src = ../../../daemon;
  inherit version;

  __structuredAttrs = true;
  strictDeps = true;

  cargoLock = {
    lockFile = ../../../daemon/Cargo.lock;
    outputHashes = {
      "framework_lib-0.6.6" = "sha256-AcATahEiCiXUNC4k9dCW6doGchjdvg6Kc7VjaOYyFGk=";
    };
  };

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ udev ];

  postInstall = ''
    install -Dm644 ${../../../data}/io.github.frameworkkcm.Daemon1.conf \
      "$out/share/dbus-1/system.d/io.github.frameworkkcm.Daemon1.conf"
    install -Dm644 ${../../../data}/io.github.frameworkkcm.policy \
      "$out/share/polkit-1/actions/io.github.frameworkkcm.policy"

    mkdir -p "$out/share/dbus-1/system-services" "$out/lib/systemd/system"
    substitute ${../../../data}/io.github.frameworkkcm.Daemon1.service.in \
      "$out/share/dbus-1/system-services/io.github.frameworkkcm.Daemon1.service" \
      --subst-var-by DAEMON_PATH "$out/bin/frameworkd"
    substitute ${../../../data}/frameworkd.service.in \
      "$out/lib/systemd/system/frameworkd.service" \
      --subst-var-by DAEMON_PATH "$out/bin/frameworkd"
  '';

  meta = {
    description = "DBus daemon for framework configuration";
    homepage = "https://github.com/flamingspaz/framework-settings";
    license = lib.licenses.gpl3Plus;
    mainProgram = "frameworkd";
    platforms = lib.platforms.linux;
  };
}
