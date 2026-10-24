#!/bin/sh
# Host-only packaging. Does not prepare USB media or include firmware utilities.
set -eu
cd "$(dirname "$0")/.."
test "$(uname -s)" = Darwin || { echo 'This packaging helper requires macOS.' >&2; exit 1; }
case "${1:-}" in
    '') app_name='Mazda OS Simulator'; bundle_id='dev.llkj.mazda-os.simulator' ;;
    --preview|--stock) app_name='Mazda Connect Typography Preview'; bundle_id='dev.llkj.mazda-os.stock-preview' ;;
    *) echo 'Usage: sh tools/package-macos.sh [--stock]' >&2; exit 2 ;;
esac
asset_dir="$PWD/target/owner-firmware/render-assets"
test -f "$asset_dir/manifest.json" || { echo 'Import the owner firmware first; see docs/STOCK_UI_PORT.md.' >&2; exit 1; }
cargo build --locked --release -p mazda-desktop
app_path="$PWD/target/macos/$app_name.app"
mkdir -p "$app_path/Contents/MacOS" "$app_path/Contents/Resources"
cp target/release/mazda-desktop "$app_path/Contents/MacOS/mazda-desktop"
cat > "$app_path/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>mazda-desktop</string>
<key>CFBundleIdentifier</key><string>$bundle_id</string>
<key>CFBundleName</key><string>$app_name</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.11.1</string>
<key>CFBundleVersion</key><string>12</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
cp LICENSE "$app_path/Contents/Resources/mazda-os-LICENSE.txt"
cp crates/mazda-ui/assets/fonts/OFL.txt "$app_path/Contents/Resources/Inter-OFL.txt"
mkdir -p "$app_path/Contents/Resources/firmware-assets"
cp "$asset_dir/"*.rgba "$asset_dir/manifest.json" "$app_path/Contents/Resources/firmware-assets/"
cp "$asset_dir/clock-zones.txt" "$app_path/Contents/Resources/firmware-assets/"
cp "$asset_dir/legal.txt" "$app_path/Contents/Resources/firmware-assets/"
cp "$asset_dir/languages.bin" "$app_path/Contents/Resources/firmware-assets/"
cat > "$app_path/Contents/Resources/OWNER_ARTWORK_NOTICE.txt" <<'NOTICE'
This local development app contains artwork extracted from the owner's supplied
Mazda NA 70.00.100A firmware. That artwork is not covered by this project's MIT
license and is not included in the public repository. Keep this package local;
it is not a firmware update or a public redistributable release.
NOTICE
# Finder/FileProvider can attach metadata that invalidates a bundle signature.
# This directory contains only this helper's generated local development app.
xattr -cr "$app_path"
codesign --force --sign - "$app_path"
printf 'Built local development app: %s\n' "$app_path"
