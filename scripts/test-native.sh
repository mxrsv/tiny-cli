#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# CLT-only installations do not ship XCTest/Swift Testing. Compile executable
# checks against the very same AppState source and built bridge, using stdlib.
"$ROOT/scripts/build-tiny-ffi.sh"
swift build --package-path "$ROOT/macos"
BIN="$(swift build --package-path "$ROOT/macos" --show-bin-path)"
swiftc -swift-version 6 -parse-as-library \
  -I "$BIN/Modules" -I "$ROOT/macos/Generated" \
  -L "$ROOT/macos/Generated" -ltiny_ffi \
  -framework CoreFoundation -framework IOKit -lobjc -liconv -lSystem -lc -lm \
  "$BIN"/TinyEngine.build/*.o \
  "$ROOT/macos/Sources/Tiny/AppState.swift" \
  "$ROOT/macos/Sources/Tiny/AppGroups.swift" \
  "$ROOT/macos/Sources/Tiny/AppCatalog.swift" \
  "$ROOT/macos/Sources/Tiny/ProcessActions.swift" \
  "$ROOT/macos/Sources/Tiny/AppQuit.swift" \
  "$ROOT/macos/Sources/Tiny/PortsModel.swift" \
  "$ROOT/macos/Tests/TinyEngineTests/EngineTests.swift" \
  "$ROOT/macos/Tests/TinyTests/ProcessStateTests.swift" \
  "$ROOT/macos/Tests/TinyTests/AppGroupTests.swift" \
  "$ROOT/macos/Tests/TinyTests/ActionTests.swift" \
  -o "$BIN/native-checks"
"$BIN/native-checks"
