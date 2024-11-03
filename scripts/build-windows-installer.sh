#!/bin/bash
VERSION=$(cargo pkgid | cut -d '@' -f 2)
ID="org.tsuza.mannager"
APP_CREATOR="tsuza"
APP_NAME="MANNager"
DESCRIPTION="Source Engine Dedicated Server Manager"
WXS_FILE="wix/main.wxs"
UPGRADE_GUID="{fb605fdc-b683-4c8e-8981-edffd210f437}" # https://www.guidgenerator.com/
ICON_PATH="assets/windows"
WEBSITE_URL="https://www.github.com/tsuza/mannager-source"
BINARY_PATH="target/release"

# Generate an ICO
conv_opts="-colors 256 -background none -density 300"
convert $conv_opts -define icon:auto-resize=256,64,48,32,16 "assets/app_icon.png" "$ICON_PATH/$APP_NAME.ico"
chmod 777 "$ICON_PATH/$APP_NAME.ico"

# build the binary
scripts/build-windows.sh

# Automatically generate a license.rtf from the repo's license, to avoid unnecessary duplication
sudo apt install pandoc
pandoc -f markdown -s LICENSE -o wix/LICENSE.rtf

# install latest wix
dotnet tool install --global wix

# add required wix extension
wix extension add WixToolset.UI.wixext

# build the installer
wix build \
    -pdbtype none \
    -arch x64 \
    -d id=$ID \
    -d app_name=$APP_NAME \
    -d app_creator=$APP_CREATOR \
    -d version=$VERSION \
    -d description=$DESCRIPTION \
    -d upgrade_guid=$UPGRADE_GUID \
    -d path_to_icon=$ICON_PATH \
    -d website_url=$WEBSITE_URL \
    -d binary_path=$BINARY_PATH$ \
    $WXS_FILE \ 
    -o target/release/mannager-installer.msi \
    -ext WixToolset.UI.wixext