// swift-tools-version: 6.0
import PackageDescription
import Foundation

let generated = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Generated").path

let package = Package(
    name: "TinyNative",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "Tiny", targets: ["Tiny"])],
    dependencies: [
        .package(url: "https://github.com/airbnb/lottie-spm.git", from: "4.6.1")
    ],
    targets: [
        .systemLibrary(name: "tiny_ffiFFI", path: "Generated"),
        .target(name: "TinyEngine", dependencies: ["tiny_ffiFFI"], linkerSettings: [
            .unsafeFlags(["-L", generated, "-ltiny_ffi"]),
            .linkedFramework("CoreFoundation"), .linkedFramework("IOKit"),
            .linkedLibrary("objc"), .linkedLibrary("iconv"),
            .linkedLibrary("System"), .linkedLibrary("c"), .linkedLibrary("m")
        ]),
        .executableTarget(name: "Tiny", dependencies: [
            "TinyEngine",
            .product(name: "Lottie", package: "lottie-spm")
        ], resources: [.copy("Resources/moved-to-trash.json")])
    ]
)
