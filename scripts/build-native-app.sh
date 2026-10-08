#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# --smoke-hooks compiles development-only checks (SmokeHooks.swift); the default build excludes them.
SMOKE_FLAGS=()
case "${1:-}" in
  "") ;;
  --smoke-hooks) SMOKE_FLAGS=(-Xswiftc -DTINY_SMOKE_HOOKS) ;;
  *) printf 'Usage: %s [--smoke-hooks]\n' "$0" >&2; exit 2 ;;
esac
"$ROOT/scripts/build-tiny-ffi.sh"
swift build --package-path "$ROOT/macos" -c release ${SMOKE_FLAGS[@]+"${SMOKE_FLAGS[@]}"}
BIN="$(swift build --package-path "$ROOT/macos" -c release ${SMOKE_FLAGS[@]+"${SMOKE_FLAGS[@]}"} --show-bin-path)"
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
<key>NSAppleEventsUsageDescription</key><string>Tiny asks Finder to move the items you confirm to the Trash, so you can put them back.</string>
</dict></plist>
PLIST
codesign --force --sign - "$APP"
codesign --verify --deep --strict "$APP"
# Ad-hoc signing without hardened runtime needs no apple-events entitlement.
/usr/libexec/PlistBuddy -c 'Print :NSAppleEventsUsageDescription' "$APP/Contents/Info.plist" >/dev/null
# Test hooks must never reach the app (T5d).
if grep -qi 'debugpanic' "$ROOT/macos/Sources/TinyEngine/tiny_ffi.swift" || LC_ALL=C grep -aqi 'debug_panic' "$APP/Contents/MacOS/Tiny"; then
  printf 'Debug test hooks found in the bridge or binary: %s\n' "$APP" >&2
  exit 1
fi
# The flag name is a short inline Swift string, so check a long hook-only message instead.
if [ ${#SMOKE_FLAGS[@]} -eq 0 ] && LC_ALL=C grep -aq 'only an instance launched here may be quit' "$APP/Contents/MacOS/Tiny"; then
  printf 'Default build must not contain smoke hooks: %s\n' "$APP" >&2
  exit 1
fi
printf 'Built and verified: %s\n' "$APP"
