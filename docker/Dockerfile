# Build environment for framework-kcm. Arch-based so the result links against
# the same Qt/KF6/hidapi versions as an up-to-date Arch host.
FROM archlinux:latest

RUN pacman -Syu --noconfirm --needed \
        base-devel git pkgconf \
        rust \
        cmake extra-cmake-modules \
        qt6-base qt6-declarative \
        kcoreaddons ki18n kcmutils kirigami \
        hidapi libusb systemd-libs \
    && pacman -Scc --noconfirm
