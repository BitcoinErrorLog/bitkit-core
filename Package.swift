// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let tag = "v0.2.5-pubky"
let checksum = "e2d9706d8df051076bcc6d502c7369b264846320bdbf1027dad11bcd26552ad5"
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
