#!/bin/bash

# Build
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

# Get version
VERSION=$(grep "^version" Cargo.toml | cut -d "\"" -f 2)

# If main or pre release
if [ $(git branch --show-current) == "main" ];
then
	DEV="";
else
	DEV="prerelease"
fi

# Move binaries
cp target/release/weoo target/release/weoo_$DEVv$VERSION
cp target/x86_64-pc-windows-gnu/release/weoo.exe target/x86_64-pc-windows-gnu/release/weoo_$DEVv$VERSION.exe

# Zip everything
zip -j "weoo_$DEVv$VERSION.zip" target/release/weoo_$DEVv$VERSION target/x86_64-pc-windows-gnu/release/weoo_$DEVv$VERSION.exe Database.json CustomPoi.json
