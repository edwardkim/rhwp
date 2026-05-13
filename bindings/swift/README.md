# RHWP Swift Binding

Swift wrapper for the shared native ABI in `bindings/Native`.

The package exposes:

- `Rhwp.readText(inputFile:page:)`
- `Rhwp.writeText(_:outputFile:)`
- `Rhwp.exportText(inputFile:outputDirectory:page:)`
- `Rhwp.exportMarkdown(inputFile:outputDirectory:page:)`
- `RhwpDocumentTextView(inputFile:page:)` for SwiftUI text display

The export methods return `RhwpExportResult`; direct reads return
`RhwpDocumentText`; writes return `RhwpWriteResult`. All methods throw
`RhwpError` when the native call fails.

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

## Examples

`Examples/read_text_ffi.swift` — FFI 직접 호출로 HWP 파일의 텍스트를 읽어 출력하는 예제.
`Examples/write_text_ffi.swift` — FFI 직접 호출로 단일 문단 HWP 파일을 생성하는 예제.

```sh
# 1. 네이티브 라이브러리 빌드
cargo build --manifest-path bindings/Native/Cargo.toml --release

# 2. 예제 실행 (기본: samples/KTX.hwp 전체 페이지)
swift bindings/swift/Examples/read_text_ffi.swift

# 3. 특정 파일 + 특정 페이지
swift bindings/swift/Examples/read_text_ffi.swift samples/aift.hwp 0

# 4. HWP 생성 예제 (기본 출력: output/ffi-created.hwp)
swift bindings/swift/Examples/write_text_ffi.swift

# 5. 출력 경로 + 본문 지정
swift bindings/swift/Examples/write_text_ffi.swift output/hello.hwp "한글 English 123"
```

## XCFramework

For app integration, package the native library as an `XCFramework` from the
repository root:

```sh
./scripts/package-swift-xcframework.sh
```

The archive is written under `dist/swift/` and contains
`RhwpNative.xcframework`, `LICENSE`, and this README.

By default, the iOS simulator slice includes Apple Silicon (`arm64`). Set
`INCLUDE_IOS_SIM_X86_64=1` when an Intel simulator slice is also required.
