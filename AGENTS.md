> Global standard: ~/.claude v2026-10-04.

# tiny-cli

Rust workspace with a shared [core](crates/tiny-core/src/lib.rs), a
[CLI](crates/tiny/src/main.rs), and a native [SwiftUI app](macos/Sources/Tiny/TinyApp.swift).

## Common commands

- Rust checks: `cargo test -p tiny-ffi -p tiny-core --all-targets --locked`
- Rust lint: `cargo clippy -p tiny-ffi -p tiny-core --all-targets --locked -- -D warnings`
- Native bridge: [`scripts/build-tiny-ffi.sh`](scripts/build-tiny-ffi.sh)
- Native checks: [`scripts/test-native.sh`](scripts/test-native.sh)
- Native bundle: [`scripts/build-native-app.sh`](scripts/build-native-app.sh)

## Repo-specific conventions

- Rust uses `snake_case`; Swift uses `PascalCase` filenames. SwiftPM configuration
  lives in [macos/Package.swift](macos/Package.swift).
- The [bridge builder](scripts/build-tiny-ffi.sh) generates ignored Swift/C bindings
  and a static archive. Never edit generated files or enable `test-hooks` in the app.
- The [Swift engine actor](macos/Sources/TinyEngine/Engine.swift) serializes process
  sampling off the main actor. Keep PID/start-time identity through list and detail.
- Whole-machine usage comes from the [system sampler](crates/tiny-core/src/processes/snapshot.rs),
  never summed app/process values.
- App-row Quit is the one Swift-owned action ([AppQuit](macos/Sources/Tiny/AppQuit.swift),
  `NSRunningApplication.terminate()`); it is gated by the Rust `refusal` on
  `FfiProcessInfo`. Per-PID Quit/Force Quit and Move to Trash go through Rust and its
  session gate (`Busy`).
- Every Swift-side mutation claims the single [ActionState](macos/Sources/Tiny/ProcessActions.swift)
  guard and writes a per-operation [in-flight marker](macos/Sources/Tiny/InFlightMarker.swift)
  before acting; nothing is replayed at launch.
- A new cleanup category implements `CleanProvider`, is added to `all_providers_with`
  and gets one row in [`CATEGORIES`](crates/tiny-core/src/clean/providers/mod.rs)
  (family, `comes_back`, desktop decision). Ids and trust sections derive from that
  row; a test fails when the row and the provider disagree.
- Harness checks, snapshots and smoke flags never call the real `cleanExecute` (it
  moves real files through Finder); use `FakeCleanEngine`. Real `cleanDiscover` and
  `cleanPreview` are read-only. PC-C3 (review items are never moved unless explicitly
  selected) is enforced in [the FFI preview](crates/tiny-ffi/src/clean.rs) and in Swift.
- [build-native-app.sh](scripts/build-native-app.sh) adds `NSAppleEventsUsageDescription`
  and fails if smoke or debug hooks reach the default bundle; `--smoke-hooks` builds
  the O1 test flag. Hardened runtime is off, so no apple-events entitlement exists.
- [App groups](macos/Sources/Tiny/AppGroups.swift) use bundle paths and conservative
  same-owner ancestry. [AppCatalog](macos/Sources/Tiny/AppCatalog.swift) owns local
  icons/metadata; nested helper metadata must not replace an outer app's identity.
- [Native checks](scripts/test-native.sh) use Foundation-only executable tests because
  Command Line Tools do not include XCTest/Swift Testing. Do not add dependencies
  merely to replace these checks.
- Docs and comments follow existing English usage; PR titles and bodies are Vietnamese.
