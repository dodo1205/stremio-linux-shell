#!/bin/sh

package_id="com.stremio.App.Devel"
cwd="build-aux/flatpak"

python3 $cwd/flatpak-builder-tools/cargo/flatpak-cargo-generator.py Cargo.lock -o $cwd/cargo-sources.json

flatpak-builder --force-clean $cwd/build $package_id.json
flatpak build-export $cwd/repo $cwd/build
flatpak build-bundle $cwd/repo $cwd/$package_id.flatpak $package_id