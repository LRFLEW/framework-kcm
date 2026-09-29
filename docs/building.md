# Building from source

Requirements: Rust (cargo), CMake, extra-cmake-modules, Qt 6, KF6
(KCMUtils, I18n, CoreAddons), Kirigami, gettext and libudev.

## With Docker

No Rust toolchain needed on the host. The image is Arch-based, so the
result links against the same libraries as an up-to-date Arch system.

```sh
./build.sh                          # extra args go to cmake, e.g. ./build.sh -DBUILD_DAEMON=OFF
sudo cmake --install build
```

The project is mounted at the same path inside the container, so
`cmake --install` works from the host afterwards.

## Natively (Arch Linux)

```sh
sudo pacman -S --needed rust cmake extra-cmake-modules kcmutils ki18n kirigami

cmake -B build -DCMAKE_INSTALL_PREFIX=/usr
cmake --build build
sudo cmake --install build
```

On Debian or Ubuntu, add `-DKDE_INSTALL_USE_QT_SYS_PATHS=ON` so the plugin
lands in the multiarch Qt directory.

## After installing

```sh
sudo systemctl daemon-reload
sudo systemctl reload dbus          # pick up the new bus policy
sudo systemctl enable --now framework-kcmd

kcmshell6 kcm_framework             # or System Settings → System → Framework Laptop
```

If you later switch to the release packages, remove the files listed in
`build/install_manifest.txt` first, or the package manager will report
conflicting files.

## Useful options

- `-DBUILD_DAEMON=OFF`: build only the KCM, for example to work on the QML.
- The daemon is built with `cargo build --release --locked`, so after
  changing Rust dependencies run `cargo update` in `daemon/` to refresh
  `Cargo.lock`.

## Debugging the service

Talk to it directly:

```sh
busctl introspect io.github.frameworkkcm.Daemon1 /io/github/frameworkkcm/Daemon1
busctl call io.github.frameworkkcm.Daemon1 /io/github/frameworkkcm/Daemon1 \
    io.github.frameworkkcm.Daemon1 GetThermal
busctl call io.github.frameworkkcm.Daemon1 /io/github/frameworkkcm/Daemon1 \
    io.github.frameworkkcm.Daemon1 SetChargeLimit i 80
journalctl -u framework-kcmd
```

Run it in the foreground with more logging (stop the service first):

```sh
sudo systemctl stop framework-kcmd
sudo RUST_LOG=debug build/cargo/release/framework-kcmd
```

See [architecture.md](architecture.md) for the full D-Bus API.
