#!/bin/sh

app_id="com.stremio.App"
cwd="build-aux/flatpak"

python3 $cwd/flatpak-builder-tools/cargo/flatpak-cargo-generator.py Cargo.lock -o $cwd/cargo-sources.json

flatpak-builder --force-clean $cwd/build $app_id.json
flatpak build-export $cwd/repo $cwd/build
flatpak build-bundle $cwd/repo $cwd/$app_id.flatpak $app_id