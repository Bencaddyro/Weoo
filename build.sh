#!/bin/bash

# Build
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

# Get version
VERSION=$(grep "^version" Cargo.toml | cut -d "\"" -f 2)

# Move binaries
cp target/release/weoo_nav_tool target/release/weoo_nav_tool_v$VERSION
cp target/x86_64-pc-windows-gnu/release/weoo_nav_tool.exe target/x86_64-pc-windows-gnu/release/weoo_nav_tool_v$VERSION.exe

# Zip everything
zip -j "weoo_nav_tool_v$VERSION.zip" target/release/weoo_nav_tool_v$VERSION target/x86_64-pc-windows-gnu/release/weoo_nav_tool_v$VERSION.exe Database.json CustomPoi.json
