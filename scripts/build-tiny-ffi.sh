#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
GENERATED="$ROOT/macos/Generated"
NATIVE_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
mkdir -p "$GENERATED"
cargo build -p tiny-ffi --release --locked --target-dir "$ROOT/target" --target "$NATIVE_TARGET"
# Keep the bindgen feature out of the static archive linked into the app.
cp "$ROOT/target/$NATIVE_TARGET/release/libtiny_ffi.a" "$GENERATED/libtiny_ffi.a"
cargo run -p tiny-ffi --features bindgen --bin uniffi-bindgen --locked --target-dir "$ROOT/target" --target "$NATIVE_TARGET" --   generate --library "$GENERATED/libtiny_ffi.a" --language swift --out-dir "$GENERATED"
cp "$GENERATED/tiny_ffi.swift" macos/Sources/TinyEngine/tiny_ffi.swift
cp "$GENERATED/tiny_ffiFFI.modulemap" "$GENERATED/module.modulemap"
printf 'Generated native bridge: %s\n' "$GENERATED"
