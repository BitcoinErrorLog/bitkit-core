// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let tag = "v0.2.8-pubky-fix-session"
let checksum = "25e9b61c29b7fca7d4a9e3d282fe155fe2478d9d6c918a5e1d59ddd497771bd3"
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
