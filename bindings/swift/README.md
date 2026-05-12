# RHWP Swift Binding

Swift wrapper for the shared native ABI in `bindings/Native`.

The package exposes:

- `Rhwp.exportText(inputFile:outputDirectory:page:)`
- `Rhwp.exportMarkdown(inputFile:outputDirectory:page:)`

Both methods return `RhwpExportResult` and throw `RhwpError` when the native
call fails.

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

For app integration, package the native library as an `XCFramework` or place it
in the app bundle and configure the appropriate library search path.
