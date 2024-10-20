#!/bin/bash
ARCH="x86_64"
TARGET="mannager"
ID="com.github.tsuza.mannager-source"
VERSION=$(cargo pkgid | cut -d '@' -f 2)
PROFILE="release"
ASSETS_DIR="assets/linux"
RELEASE_DIR="target/$PROFILE"
BINARY="$RELEASE_DIR/$TARGET"
ARCHIVE_DIR=".temp"
ARCHIVE_NAME="$TARGET-$VERSION-$ARCH-linux.tar.gz"
ICON_NAME="app_icon.png"
ICON_PATH="assets/$ICON_NAME"
FLATPAK_MANIFEST_PATH="$ASSETS_DIR/flatpak/$ID.json"

set -xe

flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --noninteractive --user flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08 org.freedesktop.Sdk.Extension.rust-stable//23.08

flatpak install --noninteractive --user org.freedesktop.appstream-glib
flatpak run --env=G_DEBUG=fatal-criticals org.freedesktop.appstream-glib validate assets/linux/$ID.appdata.xml

python3 -m pip install toml aiohttp
curl -L 'https://github.com/flatpak/flatpak-builder-tools/raw/master/cargo/flatpak-cargo-generator.py' > /tmp/flatpak-cargo-generator.py
python3 /tmp/flatpak-cargo-generator.py Cargo.lock -o $ASSETS_DIR/flatpak/generated-sources.json

flatpak-builder \
  --force-clean \
  --user -y \
  --disable-rofiles-fuse \
  --state-dir "$ARCHIVE_DIR/$TARGET-flatpak-builder" \
  "$ARCHIVE_DIR/$TARGET-build-flatpak" \
  "$FLATPAK_MANIFEST_PATH"

flatpak build-bundle \
  "$ARCHIVE_DIR/$TARGET-build-flatpak" \
  $TARGET.flatpak \
  $ID