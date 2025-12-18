// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let tag = "v0.2.6-pubky"
let checksum = "8a32e3f07feea5faed3b6b32bc2858a2c807ff33aa7b91dd8104141a3b1dda66"
let url = "https://github.com/BitcoinErrorLog/bitkit-core/releases/download/\(tag)/BitkitCore.xcframework.zip"

let package = Package(
    name: "bitkitcore",
    platforms: [
        .iOS(.v15),
        .macOS(.v12),
    ],
    products: [
        .library(
            name: "BitkitCore",
            targets: ["BitkitCoreFFI", "BitkitCore"]),
    ],
    targets: [
        .target(
            name: "BitkitCore",
            dependencies: ["BitkitCoreFFI"],
            path: "./bindings/ios",
            sources: ["bitkitcore.swift"]
        ),
        .binaryTarget(
            name: "BitkitCoreFFI",
            url: url,
            checksum: checksum
        )
    ]
)
