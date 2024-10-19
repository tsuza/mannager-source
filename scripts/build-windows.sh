#!/bin/bash
EXE_NAME="mannager.exe"
TARGET="x86_64-pc-windows-msvc"
APP_VERSION=$(cat VERSION).0

# update package version on Cargo.toml
cargo install cargo-edit
cargo set-version $APP_VERSION

# build binary
rustup target add $TARGET
cargo build --release --target=$TARGET
cp -fp target/$TARGET/release/$EXE_NAME target/release