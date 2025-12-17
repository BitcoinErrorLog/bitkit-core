// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let tag = "v0.2.0-pubky"
let checksum = "baef215e7e98af0628ddd194b3b6b8c3eec74679a20e0a50e2a5bdf6f195255a"
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
