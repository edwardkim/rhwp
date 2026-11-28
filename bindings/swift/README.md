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

## AppKit 문서 창

AppKit과 WinUI는 같은 Rust `DocumentCore`와
`native_host`를 C ABI로 호출할 수 있습니다. 문서 모델, 편집 명령, Skia 렌더링, revision과
저장 토큰은 공통이며, 파일 패널·메뉴·입력기·접근성 요소는 앱에서 연결합니다. Apple 빌드는
`rhwp-native-ffi`의 `native-skia` 기능으로 네이티브 라이브러리를 링크합니다.

### 첫 AppKit 구현 범위

1. `NSWindow`의 `NSScrollView`에 페이지 뷰를 넣고 `NSOpenPanel`로 HWP/HWPX를 엽니다.
   `Rhwp.sessionCapabilities()`에서 `abiVersion == 1`, `worker`,
   `renderTiledRegion`을 확인합니다. 기존 `Rhwp.hostRequest(request:data:)`로
   `workerCreate`·`workerOpen`·`workerSubmit`·`workerPoll`을 호출할 수 있습니다.
   Swift 호출은 같은 OS 스레드에 유지하고, AppKit 뷰 갱신은 메인 스레드로 전달합니다.
   GCD의 직렬 큐는 동일 OS 스레드를 보장하지 않습니다.
2. `pageSize`와 `renderTiledRegion`으로 첫 쪽과 보이는 영역을 그립니다. 배율과 창의
   `backingScaleFactor`를 반영하고, 결과의 문서 ID·revision·글꼴 세대·픽셀 원점 및
   행 크기를 검증합니다. `Rhwp.sessionBufferCopy(buffer:byteLength:)`로
   premultiplied RGBA/sRGB를 복사해 페이지에 표시한 뒤
   `Rhwp.sessionBufferRelease(buffer:)`로 해제합니다. 창을 닫을 때는 `workerClose`,
   `workerStatus.isFinished`, `workerRelease` 순서로 종료합니다.
3. 본문과 표 셀의 `hitTest`·`beginEdit`·`setSelection`·`replace`·`undo`·`redo`를
   `NSTextView` 입력과 연결합니다. 표시 영역의 직접 편집으로 확장할 때는
   `atomicImeEdits`를 확인하고 `imeEditContext`·`applyImeEdit`을 사용합니다.
   AppKit의 UTF-16 범위를 Rust 명령의 Unicode scalar 범위로 변환하고, 한글 조합 중인
   범위와 커밋 후 범위를 구분합니다.
4. `NSSavePanel`과 저장 경로의 외부 변경 확인을 연결합니다. `prepareSave` 응답의
   손실 보고와 바이트를 검사하고 안전한 파일 교체가 확인된 뒤 `acknowledgeSave`를
   보냅니다. 실패·취소에는 `cancelSave`를 보냅니다. 로컬 복구 사본의 저장·선택·폐기는
   앱에서 담당하며, 복구 열기는 `recoveryOpen`을 확인하고 `workerOpenRecovery`를 사용합니다.
5. `accessibilityPageSnapshots`와 `accessibilityCellLayout`을 확인하고
   `accessibility`의 쪽·중첩 표·셀·문단을 `NSAccessibility` 요소로 노출합니다.
   확대·스크롤·편집이 바뀌면 화면 좌표와 트리를 함께 갱신합니다.

Swift 바인딩의 `session*`과 `hostRequest`는 현재 JSON 및 바이트 전달을 지원합니다.
worker 수명·응답의 Swift 타입, 창·툴바, 입력기, AppKit 접근성은 구현 대상입니다.
텍스트 검색에는 `workerTextSearch`, 기본 글자·문단 서식에는 `basicTextFormatting`,
클립보드에는 `copySelection`·`cutSelection`·`pasteText`·`pasteHtml`을 사용할 수 있습니다.
공통 기능으로 제공되는 범위와 제약은 [Native 호스트 계약](../README.md#전용-문서-worker-p59)을
따릅니다. 새 편집 기능은 Rust 명령과 capability를 먼저 보강한 다음 두 앱에서 연결합니다.

### 검증 경로

같은 SHA로 빌드한 Skia 라이브러리에 [Native 계약 테스트](../Native/tests)를 실행하고,
`swift test`에서 Swift 바인딩의 링크·호출을 확인합니다. AppKit 창을 추가하면 공개 HWP 및 HWPX로
파일 열기, 쪽 이동, Retina 좌표, 한국어 조합과 실행 취소, 저장 후 재열기, 복구,
VoiceOver의 중첩 표 탐색을 macOS에서 확인합니다.

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
