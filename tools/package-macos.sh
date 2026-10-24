#!/bin/sh
# Host-only packaging. Does not prepare USB media or include firmware utilities.
set -eu
cd "$(dirname "$0")/.."
test "$(uname -s)" = Darwin || { echo 'This packaging helper requires macOS.' >&2; exit 1; }
cargo build --locked --release -p mazda-desktop
app_path="$PWD/target/macos/Mazda OS Simulator.app"
mkdir -p "$app_path/Contents/MacOS"
cp target/release/mazda-desktop "$app_path/Contents/MacOS/mazda-desktop"
cat > "$app_path/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>mazda-desktop</string>
<key>CFBundleIdentifier</key><string>dev.llkj.mazda-os.simulator</string>
<key>CFBundleName</key><string>Mazda OS Simulator</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.2.0</string>
<key>CFBundleVersion</key><string>2</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
# Finder/FileProvider can attach metadata that invalidates a bundle signature.
# This directory contains only this helper's generated local development app.
xattr -cr "$app_path"
codesign --force --sign - "$app_path"
printf 'Built local development app: %s\n' "$app_path"
