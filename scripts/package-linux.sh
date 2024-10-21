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

build() {
    echo "Building Rust project with $PROFILE as the profile..."
    cargo build --profile $PROFILE

    if [ $? -ne 0 ]; then
        echo "Rust build failed."
        exit 1
    fi
}

setup_folder() {
    echo "Copying binary and assets..."

    appdir="$ARCHIVE_DIR/appdir"

    mkdir -p $appdir

    install -Dm755 $BINARY -t $appdir/usr/bin
    install -Dm644 $ASSETS_DIR/$ID.appdata.xml -t $appdir/usr/share/metainfo
    install -Dm644 $ASSETS_DIR/$ID.desktop -t $appdir/usr/share/applications
}

generate_icons() {
    echo "Generating icons..."

    appdir="$ARCHIVE_DIR/appdir"

    conv_opts="-colors 256 -background none -density 300"

    for size in "16" "24" "32" "48" "64" "96" "128" "256" "512"; do
      path="$appdir/usr/share/icons/hicolor/${size}x${size}/apps"
      mkdir -p "$path"
      magick "$ICON_PATH" $conv_opts -resize "!${size}x${size}" "$path/$ID.png"
    done
}

package() {
  archive_path="$ARCHIVE_DIR/$ARCHIVE_NAME"
  appdir="$ARCHIVE_DIR/appdir"

  tar czvf $archive_path -C $appdir .

  echo "Packaged archive: $archive_path"
}

create_appimage() {
    # Download the AppImage tool
    pushd $ARCHIVE_DIR > /dev/null

    pwd

    echo "Downloading AppImageTool..."
    wget -c https://github.com/$(wget -q https://github.com/probonopd/go-appimage/releases/expanded_assets/continuous -O - | grep "appimagetool-.*-x86_64.AppImage" | head -n 1 | cut -d '"' -f 2)
    appimage_file=$(ls appimagetool-*.AppImage)
    chmod +x "$appimage_file"


    # Create the AppImage
    echo "Creating AppImage..."
    ./"$appimage_file" deploy "appdir/usr/share/applications/$ID.desktop"
    VERSION="$VERSION" ./"$appimage_file" "appdir"

    if [ $? -ne 0 ]; then
        echo "AppImage creation failed."

        popd

        exit 1
    fi

    # Check if the file exists and delete it
    if [ -f "$appimage_file" ]; then
        echo "Deleting $appimage_file..."
        rm "$appimage_file"
    fi

    popd
}

build_flatpak() {
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

    flatpak build-finish "$ARCHIVE_DIR/$TARGET-build-flatpak"
    
    flatpak build-export \
        "$ARCHIVE_DIR/$TARGET-build-flatpak-repo" \
        "$ARCHIVE_DIR/$TARGET-build-flatpak"

    flatpak build-bundle \
        "$ARCHIVE_DIR/$TARGET-build-flatpak-repo" \
        "$ARCHIVE_DIR/$TARGET-$VERSION-$ARCH.flatpak" \
        "$ID"
    
    if [ $? -ne 0 ]; then
        echo "Flatpak build failed."
        exit 1
    fi
}

# Main script execution
main() {
    # set -x
    build
    setup_folder
    generate_icons
    package
    # build_flatpak
    create_appimage

    echo "Done!"
}

main