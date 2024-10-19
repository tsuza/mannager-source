#!/bin/bash
WXS_FILE="wix/main.wxs"
APP_VERSION=$(cat VERSION)

# build the binary
scripts/build-windows.sh

# install latest wix
dotnet tool install --global wix

# add required wix extension
wix extension add WixToolset.UI.wixext

# build the installer
wix build -pdbtype none -arch x64 -d PackageVersion=$APP_VERSION $WXS_FILE -o target/release/mannager-installer.msi -ext WixToolset.UI.wixext