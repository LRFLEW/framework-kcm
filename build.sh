#!/bin/sh
# Build the daemon and KCM inside Docker; no Rust toolchain needed on the host.
#
# The project is mounted at the same path as on the host, so afterwards
#   sudo cmake --install build
# works directly from the host.
set -eu

cd "$(dirname "$0")"
SRC="$(pwd)"
IMAGE=framework-kcm-build

docker build -t "$IMAGE" docker

mkdir -p build
docker run --rm \
    --user "$(id -u):$(id -g)" \
    -e HOME=/tmp \
    -e CARGO_HOME="$SRC/build/cargo-home" \
    -v "$SRC:$SRC" \
    -w "$SRC" \
    "$IMAGE" \
    sh -c 'cmake -B build -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=RelWithDebInfo "$@" \
           && cmake --build build -j"$(nproc)"' sh "$@"
