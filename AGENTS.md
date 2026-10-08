> Global standard: ~/.claude v2026-10-04.

# tiny-cli

Rust workspace with a shared [core](crates/tiny-core/src/lib.rs), a
[CLI](crates/tiny/src/main.rs), and a native [SwiftUI app](macos/Sources/Tiny/TinyApp.swift).
The existing Tauri source remains a separate reference surface.

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
- The current [native interface](macos/Sources/Tiny/ProcessesView.swift) is read-only;
  process termination and cleanup integration are deferred. Whole-machine usage
  comes from the [system sampler](crates/tiny-core/src/processes/snapshot.rs), never
  summed app/process values.
- [App groups](macos/Sources/Tiny/AppGroups.swift) use bundle paths and conservative
  same-owner ancestry. [AppCatalog](macos/Sources/Tiny/AppCatalog.swift) owns local
  icons/metadata; nested helper metadata must not replace an outer app's identity.
- [Native checks](scripts/test-native.sh) use Foundation-only executable tests because
  Command Line Tools do not include XCTest/Swift Testing. Do not add dependencies
  merely to replace these checks.
- Docs and comments follow existing English usage; PR titles and bodies are Vietnamese.
