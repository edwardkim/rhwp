# RHWP Swift Binding

Swift wrapper for the shared native ABI in `bindings/Native`.

The package exposes:

- `Rhwp.readText(inputFile:page:)`
- `Rhwp.exportText(inputFile:outputDirectory:page:)`
- `Rhwp.exportMarkdown(inputFile:outputDirectory:page:)`
- `RhwpDocumentTextView(inputFile:page:)` for SwiftUI text display

The export methods return `RhwpExportResult`; direct reads return
`RhwpDocumentText`. All methods throw `RhwpError` when the native call fails.

## SwiftUI Display

```swift
import Rhwp
import SwiftUI

struct DocumentScreen: View {
    let fileURL: URL

    var body: some View {
        RhwpDocumentTextView(inputFile: fileURL)
    }
}
```

## Build the Native Library

From the repository root:

```sh
cargo build --manifest-path bindings/Native/Cargo.toml
```

The Swift module links against `rhwp_native_ffi`, so the built dynamic library
must be discoverable by the app or test host at link/runtime.

For local SwiftPM tests on macOS:

```sh
cd bindings/swift
swift test -Xlinker -L../../bindings/Native/target/debug
```

For app integration, package the native library as an `XCFramework` from the
repository root:

```sh
./scripts/package-swift-xcframework.sh
```

The archive is written under `dist/swift/` and contains
`RhwpNative.xcframework`, `LICENSE`, and this README.

By default, the iOS simulator slice includes Apple Silicon (`arm64`). Set
`INCLUDE_IOS_SIM_X86_64=1` when an Intel simulator slice is also required.
