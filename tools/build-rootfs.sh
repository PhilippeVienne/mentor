#!/bin/sh
# Builds the root file system of a microVM from a container image.
#
#   tools/build-rootfs.sh [image] [output]      defaults: alpine:3.22, .dev/rootfs.ext4
#
# The image is exported with Docker, the guest agent is added as /sbin/mentor-guest (the init process), and
# the tree is written into an ext4 image. No root privilege is needed: fakeroot records root ownership.
#
# Prototype: in the target design this runs inside a build microVM, never on the host (doc/architecture.md §3).
set -eu

image="${1:-alpine:3.22}"
output="${2:-.dev/rootfs.ext4}"
root="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "building mentor-guest (static, musl)…"
cargo build --quiet --release --target x86_64-unknown-linux-musl -p mentor-guest --manifest-path "$root/Cargo.toml"

echo "exporting $image…"
container="$(docker create "$image" /bin/true)"
trap 'docker rm -f "$container" >/dev/null 2>&1; rm -rf "$work"' EXIT
docker export "$container" > "$work/image.tar"

mkdir -p "$(dirname "$output")"
fakeroot sh -eu -c '
    mkdir "$1/rootfs"
    tar -xf "$1/image.tar" -C "$1/rootfs"
    install -D -m 0755 "$2" "$1/rootfs/sbin/mentor-guest"
    mkdir -p "$1/rootfs/workspace" "$1/rootfs/proc" "$1/rootfs/sys" "$1/rootfs/dev" "$1/rootfs/run" "$1/rootfs/tmp" "$1/rootfs/root"
    size_kb=$(du -sk "$1/rootfs" | cut -f1)
    rm -f "$3"
    mke2fs -q -t ext4 -d "$1/rootfs" -L mentor-rootfs "$3" "$((size_kb * 12 / 10 + 16384))K"
' sh "$work" "$root/target/x86_64-unknown-linux-musl/release/mentor-guest" "$output"
echo "wrote $output ($(du -h "$output" | cut -f1))"
