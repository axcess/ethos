#!/usr/bin/env bash
# Build the double-click Windows package: dist/TheTank-windows.zip
# Needs: rustup target add x86_64-pc-windows-gnu ; apt-get install gcc-mingw-w64-x86-64 zip
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build --release --target x86_64-pc-windows-gnu
out=dist/TheTank-windows
rm -rf "$out" dist/TheTank-windows.zip
mkdir -p "$out/data/climate" "$out/config/presets"
cp target/x86_64-pc-windows-gnu/release/tank.exe "$out/"
cp tools/package/windows/Tank.bat tools/package/windows/README.txt "$out/"
cp data/climate/chicago_daily.csv data/climate/SOURCES.md "$out/data/climate/"
cp config/presets/*.toml "$out/config/presets/"
(cd dist && zip -qr TheTank-windows.zip TheTank-windows)
ls -la dist/TheTank-windows.zip
