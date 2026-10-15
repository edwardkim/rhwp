# RHWP Swift Binding

Swift wrapper for the shared native ABI in `bindings/Native`.

The package exposes:

- `Rhwp.readText(inputFile:page:)`
- `Rhwp.exportText(inputFile:outputDirectory:page:)`
- `Rhwp.exportMarkdown(inputFile:outputDirectory:page:)`
- `Rhwp.exportPDFDirect(inputFile:outputFile:page:)` (native-skia opt-in)
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
CARGO_TARGET_DIR="$PWD/target/pr-review" cargo build --locked -p rhwp-native-ffi
```

The Swift module links against `rhwp_native_ffi`, so the built dynamic library
must be discoverable by the app or test host at link/runtime.

For local SwiftPM tests on macOS:

```sh
cd bindings/swift
swift test -Xlinker -L../../target/pr-review/debug
```

For app integration, package the native library as an `XCFramework` from the
repository root:

```sh
./scripts/package-swift-xcframework.sh
```

The default archive is written under `dist/swift/default/` and contains
`RhwpNative.xcframework`, `LICENSE`, and this README.

By default, the iOS simulator slice includes Apple Silicon (`arm64`). Set
`INCLUDE_IOS_SIM_X86_64=1` when an Intel simulator slice is also required.

## Direct PDF 선택 빌드

```sh
RHWP_NATIVE_SKIA=1 CARGO_TARGET_DIR="$PWD/target/pr-review" ./scripts/package-swift-xcframework.sh
```

`RHWP_NATIVE_SKIA`는 `0`(기본) 또는 `1`만 허용한다. 켠 빌드는 각 Apple target에
`native-skia`를 전달하며 archive 이름은 `*-apple-xcframework-native-skia.zip`이다.
출력은 `dist/swift/native-skia/`에 저장되어 기본 variant의 archive를 지우지 않는다.
공유 target 경로는 `CARGO_TARGET_DIR`로 지정한다. 기본값은 Git common directory의
부모 저장소 아래 `target/pr-review`이므로 linked worktree도 같은 경로를 사용한다.
Git 정보가 없는 source archive에서는 현재 저장소의 `target/pr-review`를 사용한다.
macOS·Xcode와 대상 플랫폼의 Skia 빌드 지원이 필요하며 이 옵션 추가만으로 Apple artifact의
빌드·링크가 검증되었다는 뜻은 아니다. `--help`는 다른 OS에서도 사용할 수 있다.

`exportPDFDirect`는 print profile로 최대 128 MiB 입력·256개 선택 쪽을 처리한다.
모든 쪽을 성공적으로 생성한 뒤 새 파일에 기록하며 기존 파일은 덮어쓰지 않는다.
기본 빌드에서는 `RhwpError.exportFailed`로 feature-disabled 오류를 전달한다.
자세한 JSON·페이지 선택·실패 계약은 [상위 README](../README.md#direct-pdf-p50)를 따른다.
