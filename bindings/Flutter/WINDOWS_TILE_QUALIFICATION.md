# Windows 두 타일 소비 경로 검증

P119는 기존 [Windows 불변 target 연속 표시 검증](WINDOWS_CONTINUOUS_QUALIFICATION.md)에
두 개의 실제 Flutter external texture, 공통 snapshot, 실제 창 크기와 renderer 교체 검증을
추가한다. Windows SDK compile/link나 portable 계약 검사만으로 실제 화면 검증을 완료 처리하지
않는다. 이 문서는 실행 경로이며 실제 Windows 실행 결과를 포함하지 않는다.

## 빌드와 실행

현재 커밋을 깨끗한 Windows checkout에서 빌드한다. 이전 단일 texture probe의 영수증은
거절한다. Native DLL은 기존 실제 Windows 빌드 영수증과 일치해야 한다.

```powershell
python bindings/Flutter/tool/build_desktop_probe.py `
  --flutter-sdk C:/SDK/flutter `
  --native C:/artifacts/rhwp_native_ffi.dll `
  --native-receipt C:/artifacts/windows-native-receipt.json `
  --output C:/Temp/rhwp-tile-build --tile-group

python bindings/Flutter/tool/qualify_windows_tile_group.py `
  --runner C:/Temp/rhwp-tile-build/runner/build/windows/x64/runner/Debug/rhwp_texture_qualification.exe `
  --native C:/artifacts/rhwp_native_ffi.dll `
  --dispatcher C:/Temp/rhwp-tile-build/rhwp_flutter_owner.dll `
  --fixture C:/Temp/rhwp-tile-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx `
  --build-receipt C:/Temp/rhwp-tile-build/build-receipt.json `
  --output C:/Temp/rhwp-tile-scale-100 --expected-device-scale 1 --timeout 280
```

실제 Windows의 대화형 desktop에서 창을 가리지 않은 상태로 실행한다. Linux와 Wine은 수집 전에
거절한다. 출력 디렉터리는 runtime bundle 밖의 새 경로여야 한다. 각 target의 producer handle을
읽거나 widget screenshot으로 대신하지 않고, 실행한 PID의 foreground 창을 Win32 desktop에서
직접 두 번 캡처한다. 창을 다른 monitor로 옮기거나 OS 배율을 바꾸면 실행을 다시 시작한다.

명시적인 `useTileGroups:true`는 폭 1,026 × 높이 480인 화면에도 두 native 등록을 만든다.
따라서 두 타일 소비 경로 검증에 4K 초과 화면이 필요하지 않다. 대형 viewport의 admission 한계는
기존 P112 계약과 Linux 대형 viewport 검증의 범위다. 다음 네 단계에서 같은 문서와 editor를 유지한다.

1. 최초 두 타일 게시: 두 멤버의 accepted/rendered/descriptor-open serial과 전체 stamp를 확인한다.
2. 실제 편집 revision 게시: 새 CPU 출력과 현재 native accepted snapshot이 일치해야 한다.
3. 실제 Win32 client를 가로·세로 각각 32 physical pixel 늘린 뒤 target을 1,025 × 496으로 바꾼다.
   새 창 geometry를 `FlutterView.physicalSize`와 대조한다.
4. 두 native 등록의 종료·unregister·retirement를 먼저 확인한 뒤 `NativeGpuSession.recover()`로
   같은 문서의 새 controller를 설치한다. Native device generation이 바뀌고 CPU 출력은 보존돼야 한다.
   Flutter texture 번호 자체는 재사용될 수 있으므로 generation과 함께 확인한다.

각 단계의 한 CPU strip이 두 타일 전체와 실제 경계선을 가로지른다. 같은 immutable snapshot을
별도로 1배와 2배 CPU 재생하여 비교하고, source가 바뀌면 실패한다. 기존 독립 CPU 색상·AA·정수
위치 판정을 그대로 사용한다. 전체 평균으로 경계의 한 pixel gap을 상쇄하지 않는다. CPU와 reference
읽기는 byte 수를 제한하고 한계 초과·누락을 거절한다.

## 실제 DPR 기록

매 단계 `FlutterView.devicePixelRatio`, 전체 stamp의 `deviceScale`, 실제 `GetDpiForWindow/96`
값이 같아야 한다. `--expected-device-scale`은 기대하는 OS 배율을 검사하는 옵션이며 배율을 주입하거나
시스템 설정을 바꾸지 않는다. 한 번의 실행은 그 배율만 검증하며 `actualDprComparison:false`로 기록한다.

서로 다른 배율까지 대조하려면 OS에서 다른 배율을 설정한 뒤 같은 빌드의 runner를 새 프로세스로
다시 실행한다. 위 명령의 출력 경로와 다음 옵션을 바꿔 실행한다.

```powershell
# 나머지 runner/native/dispatcher/fixture/build-receipt 인자는 첫 실행과 동일하다.
# OS 배율을 실제 125%로 설정한 환경에서만 실행한다.
--output C:/Temp/rhwp-tile-scale-125 --expected-device-scale 1.25 `
  --previous-dpr-receipt C:/Temp/rhwp-tile-scale-100/receipt.json
```

두 receipt의 네 단계·종료·snapshot 계수와 서로 다른 실제 배율을 검증한다. source/Native/Flutter/
engine SHA, runner·plugin·engine·dispatcher·Native·fixture hash와 전체 runtime manifest도 같아야
`actualDprComparison:true`가 된다. 다른 빌드나 같은 배율의 결과는 별도 DPR 증거로 인정하지 않는다.
한 창의 monitor 이동 중 연속 DPR 전환 검증과는 구분한다.

## 판정과 보존 범위

성공 결과는 `windows-two-tile-consumer-passed`다. `receipt.json`에 build와 collector identity,
PID·nonce, native 상태, 수집 화면/독립 source hash, 실제 창 resize, recovery retirement, 종료 코드를
남긴다. 실행 전후 전체 runtime byte identity를 검사하며 collector와 직접 사용하는 pixel/capture·ownership
검증 helper 2개의 hash도 빌드 입력과 대조한다. helper만 바뀐 상태나 이전 collector 빌드도 거절한다.
제한 시간은 대기·CPU 비교·두 화면 수집·ack·종료·최종 hash 확인 전체에 적용된다. 실패한 실제 캡처와
로그를 보존하고 소유 runner를 종료한다.

생산자 상한 4 frames/64 MiB, frame당 16 MiB, descriptor ticket 8개를 유지한다. 두 타일 모두 현재
serial의 descriptor opening을 확인하고, 첫 pair retirement와 최종 close에서 생산자·import·ticket
계수를 검사한다. 최종 문서 종료 후 Native 계수 9개는 모두 0이어야 한다.

`consumerGpuCompletionVerified:false`, `hardwareAccelerationVerified:false`, `targetReuse:false`,
`consumerAndDriverMemoryIncluded:false`를 유지한다. 실제 화면을 관측해도 consumer queue fence,
물리 GPU 성능, 강제 device loss, 장시간 IME/AT를 입증하는 것은 아니다.

```bash
python3 -B bindings/Flutter/test/windows_tile_group_qualification_contract.py
python3 -B bindings/Flutter/test/windows_continuous_qualification_contract.py
python3 -B bindings/Flutter/test/desktop_qualification_contract.py
python3 -B bindings/Flutter/test/native_gpu_tile_qualification_contract.py
cd bindings/Flutter
flutter test test/tile_probe_channel_test.dart
```

이 검사는 portable 거절·orchestration·shared probe 계약만 검증한다. 실제 Windows 실행은 위 build와
collector로 생성한 원본 `receipt.json`과 raw capture를 별도로 확인해야 한다.
