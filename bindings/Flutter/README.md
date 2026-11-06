# Flutter Native 참조 호스트

Flutter UI에서 기존 rhwp Native worker C ABI를 호출하는 작은 CPU 참조 패키지다.
브라우저·WASM·platform channel의 대형 픽셀 JSON을 사용하지 않는다. 문서 열기·정보·페이지 크기·
CPU RGBA 표시와 종료를 연결한다. 문서 편집·IME·접근성·GPU texture plugin·설치 프로그램은 이 단계의 범위가 아니다.

## 스레드와 메모리 계약

- Dart isolate는 OS thread를 고정하지 않는다. 기존 Native C ABI의 worker/buffer registry는 thread-local이므로
  `src/owner_dispatcher.c`의 전용 native thread가 `host_request`, capabilities, buffer copy/release와 string free를
  모두 소유한다. UI isolate는 dispatcher의 비차단 submit/poll만 호출한다. 실제 문서 파싱·조판·Skia는 기존 Rust
  `NativeRuntime` worker에서 실행한다. `Isolate.run`만으로 native 소유권이 해결된다고 가정하지 않는다.
- dispatcher ABI는 1이다. 프로세스마다 하나의 Native library 경로를 사용하며, 해당 library는 Rust TLS와 작업 코드의
  수명을 위해 프로세스 종료까지 pin한다. dispatcher owner와 worker·버퍼·reply는 명시적으로 종료·해제한다.
  `close()`는 즉시 종료 완료를 주장하지 않고, 접수한 RPC와 실행 중 Native 작업·worker release가 끝날 때 완료된다.
- dispatcher는 완료됐지만 읽지 않은 응답을 포함해 최대 64 RPC, 입력·소유 출력·출력 예약 합계 최대 64 MiB다.
  참조 문서와 단일 RGBA frame은 각각 16 MiB, Dart FIFO는 최대 64개·열기 입력 32 MiB다. Native worker는
  outstanding 4개·pending buffer reservation 32 MiB다. 이 수치는 Core 모델·글꼴·Skia 임시 메모리나 전체 RSS 상한이 아니다.
- 일반 Native 작업은 FIFO로 실행한다. 새 viewport는 실행 중 render에 취소를 요청하지만, 실제 completion 이전에
  다음 작업을 실행하지 않는다. 취소/skipped와 실패를 구분한다. 문서 epoch·ID·revision·font generation·view generation이
  맞지 않는 frame은 native buffer를 복사하지 않고 해제한다.
- Native RGBA는 owner thread에서 shim 소유 메모리로 복사한 뒤 native buffer를 해제한다. Dart는 독립적인 `Uint8List`로
  복사하고 shim reply를 해제한다. native/FFI 메모리를 가리키는 `asTypedList` view가 이 경계 밖으로 나가지 않는다.
  CPU 경로의 이 복사 비용은 존재한다. zero-copy나 Flutter/rhwp 간 Skia context 공유를 주장하지 않는다.
- `CpuFramePresenter`는 decode 중 한 frame과 최신 대기 frame 하나만 보유한다. Flutter의 비동기 `ImmutableBuffer`/
  `ImageDescriptor`/codec 작업 중 Dart 픽셀을 유지하고 codec·descriptor·buffer를 finally에서 해제한다.
  stale/dispose 뒤 완료된 image는 표시하지 않고 해제하며, 표시됐던 image는 해당 RawImage를 제거한 frame 뒤 해제한다.

## 실행

Flutter 3.47 이상·Dart 3.13 이상과 C11 compiler가 필요하다. Native shared library는 기존 Native 빌드에서 준비한다.
이 패키지는 rhwp 자체를 자동 빌드하거나 다운로드하지 않는다.

```bash
python3 tool/build_dispatcher.py --output /absolute/path/librhwp_flutter_owner.so
flutter pub get
dart run tool/native_ffi_smoke.dart /absolute/path/librhwp_flutter_owner.so /absolute/path/librhwp_native_ffi.so sample.hwpx
flutter analyze
flutter test --concurrency=1
```

Linux의 dispatcher 파일은 `.so`, macOS는 `.dylib`, Windows는 `.dll`이다. Windows 빌드는 C11을 지원하는
Visual Studio 개발 셸에서 실행한다. 실제 검증한 OS·SDK와 source SHA는 PR 검증 기록을 따른다.

`example/lib/main.dart`는 파일 경로로 HWP/HWPX를 열고 첫 페이지의 보이는 CPU 영역을 표시한다.
큰 문서·화면은 위 상한을 명시적으로 거절하며 전체 데스크톱 편집기를 제공하지 않는다.
표준 Flutter desktop runner가 없는 checkout에서는 example에 필요한 runner를 먼저 생성한다.

```bash
cd example
flutter create --platforms=linux .
flutter pub get
flutter run -d linux \
  --dart-define=RHWP_OWNER_LIBRARY=/absolute/path/librhwp_flutter_owner.so \
  --dart-define=RHWP_NATIVE_LIBRARY=/absolute/path/librhwp_native_ffi.so
```

## 검증과 후속 GPU 연결

`test/native_owner_contract.py`는 `test/support/mock_native.c`로 owner thread·실패 시 버퍼 해제·큐/바이트 상한·
close 지연 완료를 검사한다. `tool/native_ffi_smoke.dart`는 실제 Native library를 열어 문서를 다시 열고,
정보·CPU render·buffer release·close 이후 Dart 픽셀 소유권을 확인한다. Dart/Flutter 테스트는 FIFO·stale 결과·
dispose와 decode 수명을 검사한다. 실제 OS 배포·서명·IME·screen reader·GPU 성능 검증과 구분한다.

Linux에서 SDK 없이 dispatcher 계약을 검사할 수 있다.

```bash
cc -std=c11 -O2 -fPIC -Wall -Wextra -Werror -shared test/support/mock_native.c -o /tmp/librhwp_mock_native.so
python3 test/native_owner_contract.py --dispatcher /absolute/path/librhwp_flutter_owner.so --mock /tmp/librhwp_mock_native.so
```

P89 presentation/snapshot/frame ABI는 별도의 capability다. 그 ABI가 cross-thread frame을 제공해도 legacy
workerCreate/submit/open/poll/string/buffer 소유권은 바뀌지 않는다. 이 dispatcher는 별도의 cross-thread actor ABI가
명시적으로 공개되기 전까지 필요하다. 향후 GPU plugin은 capability를 확인한 뒤 presentation snapshot/frame만
raster thread로 전달해야 하며, Flutter 내부 `SkCanvas*`나 렌더러 구현·버전·GPU context를 공유한다고 가정하지 않는다.

관련 API: [Flutter native FFI](https://docs.flutter.dev/interop/bind-native-code),
[Flutter raw image decode](https://api.flutter.dev/flutter/dart-ui/decodeImageFromPixels.html).

## 입력·선택·IME 참조 연결 (P102)

`NativeEditorView`는 `NativeDocumentView` 또는 외부 GPU viewport 위에 Native caret/selection
geometry를 표시한다. `NativeEditorController`가 편집을 직렬화하고, NativeDocument의 각 편집은
최신 info를 받기까지 FIFO slot을 유지한다. 이후 입력은 실제 완료 revision을 사용하며,
이미 받은 입력이 다른 문서나 새 editor owner로 옮겨 가지 않는다.

Flutter DeltaTextInputClient의 UTF-16 문맥은 Native `imeEditContext`/`applyImeEdit`와 연결한다.
전체 bounded context와 revision/font/selection/composition을 검증하며 surrogate 중간,
16 KiB·4096 scalar 초과, 오래된 oldText, 근거 없는 조합 범위 이동은 거절하고 Native 상태로
다시 동기화한다. 한글 preview 갱신은 같은 composition을 유지하고, 확정+새 조합은 취소 baseline을
분리한다. focus loss/Escape는 현재 조합을 취소한다. 조합 중 Undo는 조합만 취소한다.

키 이동·grapheme 삭제·문단 분리·undo/redo·plain-text clipboard는 Native 명령을 사용한다.
AltGr를 Ctrl 단축키로 취급하지 않는다. 드래그 hit-test는 하나만 진행하고 다음 위치를 합친다.
IME candidate rect와 선택 overlay는 Native geometry에 동일한 page/viewport transform을 적용한다.
DPR은 raster/texture 해상도에만 적용하고 pointer/candidate rect의 logical 단위를 두 번 확대하지 않는다.

접근성 조회는 완료한 Native body/table snapshot(64 nodes·32 KiB·depth4)을 읽는다. 현재 bounded
문단에는 Flutter text-field selection/action을 연결한다. 생략한 control·header·note·shape와
불완전 scope는 Native limitations/complete 상태를 유지한다. 여러 문단/객체를 걸친 선택은 Native에
유지되며 clipboard/명령으로 다룰 수 있지만 이 IME 문맥으로 편집하지 않는다.

이 뷰는 한 페이지 상단을 표시하는 참조 연결이다. 제품용 다중 페이지 navigation/scroll,
서식 도구·파일 전환 UI 전체와 OS별 실물 IME/AT qualification을 완료했다고 표시하지 않는다.

## 스냅샷 전송과 종료 경계

owner dispatcher ABI2는 immutable snapshot을 포함한 reply에 별도 소유권을 둔다. Dart에 정상
전달할 때만 `rhwp_flutter_reply_claim_snapshot`을 호출하고, 디코딩 실패·종료·미수령 reply는
`reply_free`가 thread-safe presentation API로 해제한다. 기존 ABI1 shim은 CPU 경로로 사용하며
snapshot capture는 `ownerPresentationCleanup` capability가 있어야 활성화한다.

`capturePresentation`은 owner thread에서 source stamp를 설정한 뒤 worker snapshot을 내보낸다.
반환 handle은 호출자가 해제하거나 platform adapter에 한 번만 넘긴다. dispose 이후 이미 시작한
poll이 결과를 받는 경우에도 release callback을 실행하므로 이 callback은 source/widget 종료 후에도
유효해야 한다. `NativeOwnerTransport.nativeLibraryPath`와 metadata adapter의 정규화한 module
경로가 일치해야 같은 registry를 사용한다.

`rasterInvalidations`는 문서 열기/교체와 텍스트·history·composition 작업의 FIFO 수락 후 즉시
발생한다. GPU 뷰는 이전 Texture를 숨기고 그 편집 뒤에 새 capture를 넣는다. CPU fallback의
`viewGeneration`과 GPU의 `presentationGeneration`을 분리하며, 다른 viewport가 presentation을
획득하면 이전 뷰는 재획득 경쟁 없이 중단한다. snapshotInfo.current만으로 view/device 유효성을
추정하지 않고 전체 stamp를 확인해야 한다.
