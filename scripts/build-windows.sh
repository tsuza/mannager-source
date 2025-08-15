#!/bin/bash
EXE_NAME="mannager.exe"
TARGET="x86_64-pc-windows-msvc"
VERSION=$(cargo pkgid | cut -d '@' -f 2)

# build binary
rustup target add $TARGET
cargo build --release --target=$TARGET
cp -fp target/$TARGET/release/$EXE_NAME target/release