// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let tag = "v0.2.8-pubky-fix-session"
let checksum = "a48868672b209b161ede8ecf93f4a8f20e7aa3a37b166af0d39475de91a682c1"
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
