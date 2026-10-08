#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
"$ROOT/scripts/build-tiny-ffi.sh"
swift build --package-path "$ROOT/macos" -c release
BIN="$(swift build --package-path "$ROOT/macos" -c release --show-bin-path)"
APP="$ROOT/macos/.build/Tiny Dev.app"
mkdir -p "$APP/Contents/MacOS"
cp "$BIN/Tiny" "$APP/Contents/MacOS/Tiny"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.mxrsv.tiny.dev</string>
<key>CFBundleName</key><string>Tiny Dev</string>
<key>CFBundleDisplayName</key><string>Tiny Dev</string>
<key>CFBundleExecutable</key><string>Tiny</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
PLIST
codesign --force --sign - "$APP"
codesign --verify --deep --strict "$APP"
printf 'Built and verified: %s\n' "$APP"
