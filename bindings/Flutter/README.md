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

## Texture 종료와 등록 해제

`NativeTextureController.close()`는 같은 Future를 반환한다. 종료를 시작하면 `closing` 상태로
전환하고 이전 문서 픽셀을 불투명 cover로 가린다. **`NativeTextureView`를 mount한 상태로
close를 기다린 뒤 제거한다.** cover 아래 Texture는 계속 paint된다. Linux의 자기 Texture
populate callback이 native GL 정리를 수행하고, 그 뒤 GTK에서 등록을 해제하기 때문이다.
`Offstage`, `Opacity(0)`, widget 제거를 먼저 적용하면 이 정상 정리 경로를 사용할 수 없다.

Linux는 `cleanupComplete:true`와 `registered:false`를 모두 확인한 뒤 `closed`가 된다.
macOS는 `ownerFinished`와 `textureUnregisteredNotification`, Windows는 `ownerFinished`와
`unregistered`를 확인한다. 명시적인 `registered:false`도 등록 해제 증거로 소비한다.
이 완료는 producer/등록 수명의 종료다. macOS/Windows의 immutable·quarantined GPU 프레임은
계속 보유될 수 있으며, consumer GPU 완료나 프레임 credit 환급을 뜻하지 않는다.

close/status 확인은 기존 controller별 3개·isolate 전체 8개 RPC 한도와 `operationTimeout`의
단일 deadline을 지킨다. 시간 초과나 먼저 unmount한 경우에는 `retirePending` 상태,
`retirementPending`, `retirementDiagnostic`으로 미확정 정리를 알린다. close Future는 bounded
대기를 마치지만 정리 완료를 보증하지 않는다. 자동 무한 status polling이나 수동 native retire는
수행하지 않으며 native의 bounded retire/quarantine 정책을 유지한다. 늦은 원래 RPC는 실제 응답까지
credit을 보유한다. 늦게 도착한 create ID는 기존 소유권 observer에서 bounded close를 진행할 수 있다.
등록 해제가 확인되지 않은 Texture는 여전히 cover 아래 paint할 수 있으며, 이미 등록이 해제된 것은
제거한다. snapshot 전달·미수령 reply의 별도 해제 observer도 계속 유효하다.

## Dart 저장·복구 연결 (P103)

`NativeFileStore(document)`는 기존 worker의 prepareSave → Native buffer 복사/해제 →
파일 게시 → acknowledgeSave/cancelSave를 사용한다. Rust serializer나 worker를 새로 만들지 않는다.
`openPath`, `openRecovery`, `save`, `writeRecovery`, `sourcePath`, `isBusy`, `whenIdle`를 제공한다.

```dart
final files = NativeFileStore(document);
await files.openPath('/absolute/input.hwpx');
final saved = await files.save(
  destination: '/absolute/output.hwpx',
  approveLoss: (report) async => await confirmLoss(report),
  approveOverwrite: (observedStamp) async => await confirmOverwrite(observedStamp),
);
// saved.cancelled면 게시하지 않았다. 이후 편집이 있으면 saved.info.isDirty는 true다.
final recovery = await files.writeRecovery('/absolute/dedicated-recovery');
// 복구 파일 선택은 명시적이며 원본 경로를 자동으로 덮어쓰지 않는다.
if (recovery != null) await files.openRecovery(recovery);
```

같은 prepare의 bytes/report/token을 승인 대기 동안 유지한다. loss 승인 거절은 token을 취소하며,
승인 도중 새 편집이 생겨도 준비했던 revision만 저장하고 이후 편집의 dirty 상태를 유지한다.
Native serialization은 실행 전에 최대 문서 크기를 예약하므로 owner worker는 capability의
`maxDocumentBytes`(최대 128 MiB)를 예약 한도로 사용한다. Dart/shim의 요청·복사·문서·저장 payload
한도는 계속 16 MiB다. 128 MiB 예약이 같은 크기의 선할당이나 전송 허용을 뜻하지 않는다.

저장 파일은 같은 파일시스템의 임시 파일에 쓰고 flush한 뒤 게시한다. 새 파일은 hard link의
exclusive 이름 생성으로 완성된 bytes만 공개하며, 기존 파일은 읽을 때 함께 얻은 SHA-256·크기·
수정/변경 시각·mode stamp를 staging 전후에 검사하고 rename한다. POSIX는 별도 isolate에서
디렉터리 fsync까지 수행한 뒤 ack하며 Windows는 기존 Native 참조 호스트처럼 디렉터리 fsync를
제공하지 않는다. hard link를 지원하지 않는 파일시스템에서는 새 파일 게시를 거절한다.
기존 파일 교체의 두 stamp 검사는 임의의 외부 writer에 대한 파일시스템 compare-and-swap은 아니다.

파일 게시 뒤 Native ack·최종 검증이 실패하면 `NativePublicationError.path`의 완성된 파일을 보존한다.
재시도 전에 경로를 확인한다. cleanup은 일반 입력 FIFO가 가득 차도 한 개의 예약 slot을 쓰며,
동시 cancel 호출은 같은 Future를 공유한다. cleanup 실패나 출처를 확인할 수 없는 prepared reply는
같은 문서의 새 prepare를 차단한다. worker 종료/명시적인 문서 교체 후에만 그 상태가 해제된다.
파일 작업 중 문서를 교체하거나 닫는 호스트는 먼저 `files.whenIdle`을 기다린다.

`newDocument`, `openBytes`, `openRecoveryBytes`, `files.openPath/openRecovery`의 선택적인
`replacing`에는 사용자 확인 직후의 `NativeDocumentInfo`를 전달할 수 있다. 문서 ID·revision·font
세대가 enqueue 또는 실제 실행 전에 달라지면 교체를 거절한다. 파일 읽기 때문에 늦어진 open도
읽기 전 snapshot을 유지한다.

복구는 Python/Qt 참조 호스트와 같은 `RHWP-R1\n` + little-endian header 길이 + JSON + HWPX
envelope이며, header 64 KiB·payload 16 MiB·전용 디렉터리 32개/256 MiB와 256-entry 조회 한도를
적용한다. source path/stamp는 provenance이고 원본 저장 목적지로 채택하지 않는다. 같은 dirty
revision의 중복 artifact는 생략하며 타이머·목록 선택·정리 정책은 호출 호스트가 소유한다.

저장·복구 검사와 실제 Native opt-in을 실행한다. Python envelope 상호 운용 검사는 `python3`와
`../Native/examples/native_file_store.py`를 사용하며 별도 package 사본에서는
`--dart-define=RHWP_NATIVE_EXAMPLES=/absolute/native/examples`로 지정한다.

```bash
flutter test --no-pub test/native_file_store_test.dart test/native_file_store_integration_test.dart \
  --dart-define=RHWP_OWNER_LIBRARY=/absolute/owner.so \
  --dart-define=RHWP_NATIVE_LIBRARY=/absolute/native.so
```

## 참조 문서 수명 (P104)

`NativeDocumentLifecycle`는 기존 `NativeFileStore` 위에서 편집 입력·파일 작업이 끝난 뒤
저장/폐기/취소와 문서 교체·복구 열기·종료를 조정한다. 경로/승인 대기 중 문서 교체를 거절하고,
GPU 종료 Future가 반환되어도 `retirePending`이면 view와 owner를 유지한다.
예제의 작은 경로·결정 UI와 실제 Native 저장/복구/종료 검사는
[문서 수명 계약](DOCUMENT_LIFECYCLE.md)을 따른다. OS 창 닫기 hook과 제품 앱은 범위에 포함하지 않는다.

## 다중 페이지 참조 viewport (P105·P111)

`NativePagedDocumentView(document: document, controller: controller)`는 문서당 하나의
view generation으로 보이는 페이지 영역만 순서대로 요청한다. `NativePagedViewportController`의
`scrollTo`, `jumpToPage`, `setZoom`과 휠/수평 휠, Ctrl/Cmd+휠을 지원한다. `editable:false`는
읽기 전용이며 기본값은 기존 단일 `NativeEditorController`를 통한 입력이다. `onViewportReady`는
현재 viewport의 디코딩 완료를 알리며 Flutter/GPU의 소비 완료를 뜻하지 않는다.

페이지 크기는 Native가 반환한 값을 사용하고 조판하지 않는다. 페이지 metadata는 최대 256개,
보이는 영역은 최대 4쪽이며, 보이는 raster 계획과 게시할 decoded batch는 각각 16 MiB로 제한한다.
디코딩 중 입력 RGBA와 codec의 임시 복사본도 존재하며 이 수치는 process RSS나 전체 GPU 메모리 한도가 아니다.
현재 batch를 제거한 paint frame에서 이미지를 해제한 뒤 다음 decode를 시작하므로 스크롤로
미해제 image batch가 쌓이지 않는다. 렌더링은 한 번에 하나이며 새 탐색은 진행 중인 tiled
렌더를 기존 Native 취소 경로로 취소하고 가장 최근 viewport만 남긴다. 제한을 넘는 요청은
본문 일부를 조용히 생략하지 않고 진단으로 거부한다.

페이지 배치·포인터 hit-test·선택/caret overlay·IME candidate는 같은 `NativePageViewport`
변환을 사용한다. DPR은 raster 해상도에만 적용한다. Native caret가 화면 밖으로 이동하면
같은 배치 결과로 스크롤하고, 새 문서 ID는 이전 문서의 탐색 위치를 초기화한다.
`gpuController`가 없는 기본값은 기존 CPU 참조 경로다. 선택적인
`gpuController: textureController`를 주면 같은 `NativeDocumentViewport`의 보이는 최대 4쪽을
동일한 document/revision/font/view/device stamp로 capture하고 한 GPU target의 placements로
게시한다. 단일 페이지 `NativeTextureView` API도 유지한다. 여러 페이지의 GPU consumer 완료나
target 재사용을 추가한 계약은 아니다. [GPU 다중 페이지 계약](GPU_MULTIPAGE_VIEWPORT.md)을 따른다.

```bash
flutter test --no-pub test/native_multipage_view_test.dart
flutter test --no-pub test/native_multipage_integration_test.dart \
  --dart-define=RHWP_OWNER_LIBRARY=/absolute/path/librhwp_flutter_owner.so \
  --dart-define=RHWP_NATIVE_LIBRARY=/absolute/path/librhwp_native_ffi.so \
  --dart-define=RHWP_MULTIPAGE_DOCUMENT=/absolute/path/radial-corner-print-contract.hwpx
```

실제 integration 검사는 기존 공개 6쪽 입력의 두 번째 페이지를 Flutter에서 표시한 pixels와
동일한 Native CPU 영역 출력으로 대조한다. 합성 geometry/worker 검사와 구분하며, 한컴 독립
전쪽 비교·실물 OS IME/AT 및 GPU qualification은 별도 범위다.

기본 example은 CPU 다중 페이지 뷰와 최소 이전/다음·확대/축소 조작을 사용한다.
`RHWP_GPU=true`로 GPU controller가 연결되면 같은 다중 페이지 뷰·탐색·입력 owner를 유지하며
GPU 출력을 선택한다. 정상 GPU 경로에서는 CPU raster/decode를 실행하지 않고, 확정된 GPU
fallback 뒤에만 기존 CPU batch를 만든다. 문서/resource 오류는 실패 진단으로 남는다.
GPU example은 지원 capability가 있는 Linux·Windows에서 기존 두 producer slot을 사용하는
P112 tile group을 선택한다. `RHWP_GPU_TILES=false`로 기존 단일 target을 선택할 수 있다.
macOS의 CV Metal·AppKit Metal은 기존 단일 target 계약을 유지한다. AppKit Metal 선택은 아래
continuous opt-in이 필요하다.
`RHWP_GPU_CONTINUOUS=true`는 기존 Windows immutable ownership transfer 또는 macOS AppKit
Metal opt-in 선택을 유지한다. `onControllerChanged`로 같은 view-owned editor를 전달하므로
P104 lifecycle은 문서 교체 전에 입력 큐를 기다릴 수 있다.

다른 `NativeDocument` owner에 viewport/controller를 다시 연결하면 이전 문서의 탐색 위치와
대기 중 page jump를 초기화한다. 같은 문서가 재조판되어 쪽수가 줄면 대기 jump를 새 쪽수에
맞춰 처리하며, 이전 page index 때문에 viewport 오류가 고정되지 않게 한다.

## 최대 두 GPU 타일 (P112)

현재 native tile group 구현은 Linux·Windows에 제공한다. macOS native plugin은 검증된 기존
단일 target 소스를 유지한다. 새 Mac tile group은 Apple SDK compile/link·실행 증거 확보 후
[#8701](https://github.com/edwardkim/rhwp/issues/8701)의 후속으로 검토한다. 공통 Dart client는
capability가 없는 plugin에 그룹을 요청하지 않으며, example도 Mac의 타일 모드를 켜지 않는다.

`NativeTextureController(useTileGroups: true, ...)`는 문서·입력·presentation owner 하나로
viewport를 두 물리 직사각형으로 나눈다. 각 축 8192·합계 8 Mi pixels 이하에서 타일별 4096축·
4 Mi pixels·16 MiB 한도를 만족해야 한다. Windows는 256-byte row pitch와 power-of-two surface
두 장의 실제 charge도 검사한다. 기존 두 producer/close record, 네 frame·64 MiB 예산은 늘리지 않는다.
다른 GPU view나 불확정 종료가 slot을 보유하면 그룹을 거부하거나 bounded CPU로 전환한다.

`createGroup/submitGroup/statusGroup/closeGroup/retireGroupAck`는 하나의 device generation을
공유한다. Windows의 registrar import owner는 타일마다 고유하다. 최대 네 쪽의 snapshot은 각각
한 번 capture하며 두 타일의 page job은 같은 destructive lease를 공유한다. 그룹 전체를 검증·준비한
뒤에만 인수하고, 두 publication의 전체 stamp와 page receipt 확인 뒤 같은 opaque cover를 벗긴다.
늦은 응답·불확정 release는 기존 credit을 유지한다.

기본 `NativeDocumentViewport`의 CPU 계획은 여전히 16 MiB다. 큰 GPU 계획은 명시적인
`NativeViewportRasterPolicy.gpuTiled`를 사용한다. `NativePagedDocumentView`는 tiled controller에
이 정책을 적용하며 GPU 실패 시 모든 page placement를 유지한 채 DPR을 내려 전체 CPU batch를
16 MiB에 맞춘다. 화면에 낮은 해상도 표시를 알리고 이전 batch를 paint 후 해제한 다음 교체한다.
본문 일부만 누락하여 한도를 맞추지 않는다. GPU 한도를 넘으면 전체 viewport 진단을 표시한다.

실제 Linux 두 target 실행은 clean source에서 다음과 같이 준비한다. X11 화면은 4200×700 이상이어야 한다.

```sh
python3 tool/build_linux_probe.py --tile-group --flutter-sdk /absolute/flutter \
  --native /absolute/librhwp_native_ffi.so --native-source-head NATIVE_SHA \
  --output /absolute/tile-build
python3 tool/qualify_linux_tile_group.py \
  --runner /absolute/tile-build/runner/build/linux/x64/debug/bundle/rhwp_texture_qualification \
  --native /absolute/librhwp_native_ffi.so --dispatcher /absolute/tile-build/librhwp_flutter_owner.so \
  --fixture /absolute/tile-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx \
  --build-receipt /absolute/tile-build/build-receipt.json --output /absolute/tile-capture
```

4098px viewport의 초기·편집·4097px resize를 실제 compositor에서 capture하고, 같은 immutable
snapshot의 별도 CPU strip·2배 source replay로 양쪽과 seam을 검사한다. 각 CPU 요청은 기존 한도
안에 남는다. 실제 두 registration 종료와 Native 아홉 자원 counter의 0을 확인한다. Xvfb/Mesa 결과는
소프트웨어 실행 증거이며 Windows/macOS SDK·실물 GPU·한컴 독립 출력 qualification을 대신하지 않는다.
