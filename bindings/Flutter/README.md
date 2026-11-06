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
