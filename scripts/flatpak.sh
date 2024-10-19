#!/bin/bash
set -xe

flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --noninteractive --user flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08 org.freedesktop.Sdk.Extension.rust-stable//23.08

flatpak install --noninteractive --user org.freedesktop.appstream-glib
flatpak run --env=G_DEBUG=fatal-criticals org.freedesktop.appstream-glib validate assets/linux/org.tsuza.mannager-source.appdata.xml

python3 -m pip install toml aiohttp
curl -L 'https://github.com/flatpak/flatpak-builder-tools/raw/master/cargo/flatpak-cargo-generator.py' > /tmp/flatpak-cargo-generator.py
python3 /tmp/flatpak-cargo-generator.py Cargo.lock -o assets/flatpak/generated-sources.json

if [ "${CI}" != "yes" ] ; then
  flatpak-builder \
    --install --force-clean --user -y \
    --disable-rofiles-fuse --state-dir /var/tmp/mannager-source-flatpak-builder \
    /var/tmp/mannager-source-flatpak-repo \
    assets/flatpak/org.tsuza.mannager-source.json
fi