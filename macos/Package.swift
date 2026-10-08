// swift-tools-version: 6.0
import PackageDescription
import Foundation

let generated = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Generated").path

let package = Package(
    name: "TinyNative",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "Tiny", targets: ["Tiny"])],
    targets: [
        .systemLibrary(name: "tiny_ffiFFI", path: "Generated"),
        .target(name: "TinyEngine", dependencies: ["tiny_ffiFFI"], linkerSettings: [
            .unsafeFlags(["-L", generated, "-ltiny_ffi"]),
            .linkedFramework("CoreFoundation"), .linkedFramework("IOKit"),
            .linkedLibrary("objc"), .linkedLibrary("iconv"),
            .linkedLibrary("System"), .linkedLibrary("c"), .linkedLibrary("m")
        ]),
        .executableTarget(name: "Tiny", dependencies: ["TinyEngine"])
    ]
)
