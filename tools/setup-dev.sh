#!/bin/sh
# Downloads what the execution-plane prototype needs into .dev/ (ignored by Git):
#   - the Firecracker release binary, verified against the checksum published with the release;
#   - a guest kernel built by the Firecracker project for its own CI.
#
#   tools/setup-dev.sh
#
# Requirements: Linux on x86_64 with KVM (/dev/kvm readable and writable by your user), curl.
# Then build the root file system with tools/build-rootfs.sh.
set -eu

FIRECRACKER_VERSION="${FIRECRACKER_VERSION:-v1.17.0}"
KERNEL_URL="${KERNEL_URL:-http://spec.ccfc.min.s3.amazonaws.com/firecracker-ci/v1.15/x86_64/vmlinux-6.1.155}"

dev="$(cd "$(dirname "$0")/.." && pwd)/.dev"
mkdir -p "$dev" && cd "$dev"

[ -r /dev/kvm ] && [ -w /dev/kvm ] || { echo "error: /dev/kvm is not accessible to $(id -un)" >&2; exit 1; }

archive="firecracker-$FIRECRACKER_VERSION-x86_64.tgz"
base="https://github.com/firecracker-microvm/firecracker/releases/download/$FIRECRACKER_VERSION"
echo "downloading Firecracker $FIRECRACKER_VERSION…"
curl -fsSL -o "$archive" "$base/$archive"
curl -fsSL -o "$archive.sha256.txt" "$base/$archive.sha256.txt"
sha256sum -c "$archive.sha256.txt"
tar -xzf "$archive"
ln -sf "release-$FIRECRACKER_VERSION-x86_64/firecracker-$FIRECRACKER_VERSION-x86_64" firecracker

echo "downloading the guest kernel…"
curl -fsSL -o vmlinux "$KERNEL_URL"

./firecracker --version | head -n 1
echo "done: now run tools/build-rootfs.sh"
