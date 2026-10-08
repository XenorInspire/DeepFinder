#!/usr/bin/env bash
set -e

cd /workspace

readonly TARGET=x86_64-unknown-linux-gnu
readonly RELEASE_DIR="$CARGO_TARGET_DIR/$TARGET/release"

VERSION="$(cargo metadata --locked --no-deps --format-version 1 \
    | python3 -c 'import json, sys; print(next(package["version"] for package in json.load(sys.stdin)["packages"] if package["name"] == "deepfinder"))')"
export VERSION

# Compress package documentation without modifying the read-only source tree.
readonly PACKAGE_RESOURCES_DIR=/tmp/deepfinder-package-resources
mkdir -p \
    "$PACKAGE_RESOURCES_DIR/usr/share/doc/deepfinder" \
    "$PACKAGE_RESOURCES_DIR/usr/share/man/man1"

gzip -9 -c /workspace/resources/usr/share/doc/deepfinder/changelog \
    > "$PACKAGE_RESOURCES_DIR/usr/share/doc/deepfinder/changelog.gz"
gzip -9 -c /workspace/resources/usr/share/man/man1/deepfinder.1 \
    > "$PACKAGE_RESOURCES_DIR/usr/share/man/man1/deepfinder.1.gz"

# Build each mutually exclusive feature separately and strip the packaged copy.

cargo build \
    --locked \
    --release \
    --target "$TARGET"

install -s -D -m 0755 "$RELEASE_DIR/deepfinder" "/tmp/deepfinder"

nfpm package \
    --config /workspace/packaging/nfpm.yaml \
    --packager deb \
    --target "/output/deepfinder_${VERSION}_amd64.deb"

nfpm package \
    --config /workspace/packaging/nfpm.yaml \
    --packager rpm \
    --target "/output/deepfinder_${VERSION}_x86_64.rpm"