// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "Rhwp",
    platforms: [
        .macOS(.v12),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "Rhwp",
            targets: ["Rhwp"]
        ),
    ],
    targets: [
        .systemLibrary(
            name: "CRhwpNative",
            path: "Sources/CRhwpNative"
        ),
        .target(
            name: "Rhwp",
            dependencies: ["CRhwpNative"]
        ),
        .testTarget(
            name: "RhwpTests",
            dependencies: ["Rhwp"]
        ),
    ]
)
