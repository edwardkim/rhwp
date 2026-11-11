# P117 macOS 두 타일 — Draft 검증 경계

`rhwp_native/metal_view_macos`의 AppKit/CAMetalLayer 경로가 하나의 문서를 최대 두 타일로
표시합니다. 기존 CVPixelBuffer 채널은 그대로 단일 target이며, 네 개 불변 프레임·64 MiB
크레딧을 환급하지 않습니다. 이 변경은 #8907 통합에서 후속으로 분리한 Mac native 범위입니다.

참조앱의 `RHWP_GPU=true`, `RHWP_GPU_CONTINUOUS=true`, `RHWP_GPU_TILES=true` 조합은
기존 `macosMetalView` 선택과 공통 `NativeTileGroupChannel`을 사용합니다. `RHWP_GPU_TILES=false`는
기존 단일 AppKit 경로를 선택합니다. 플랫폼 채널로 픽셀을 전달하거나 Flutter의 Skia context를
공유하지 않습니다.

## 소유권과 게시

- 한 그룹이 기존 두 개의 process-wide producer/layer 슬롯과 두 view 등록을 모두 사용합니다.
  그룹을 추가한다고 상한을 늘리지 않습니다. 각 타일은 최대 4096 차원·4 Mi pixels·행 정렬을
  포함한 16 MiB이며, CAMetalLayer의 drawable 수는 기존처럼 최대 세 개입니다.
- `createGroup`은 두 슬롯을 한 번에 예약하고 같은 device generation의 서로 다른 두 handle을
  반환합니다. 도중 thread 시작이 실패하면 시작한 owner의 실제 종료·layer detach를 기다린 후
  외부에 노출하지 않은 등록만 회수합니다. 불명확한 GPU 완료는 기존 quarantine 정책을 유지합니다.
- `submitGroup`은 동일한 full stamp와 최대 네 개의 고유 snapshot ID를 받습니다. 두 owner를
  함께 잠그고 gate·lease·전체 ID receipt를 준비한 뒤 한 번에 소유권을 이전합니다. 한 페이지가
  경계를 지나면 두 job이 같은 lease를 보유하고, 최종 참조가 사라질 때 Native release를 한 번
  호출합니다. pending/in-flight/displayed job의 weak-map 장부는 owner당 최대 32개의 고유 ID로
  제한하며 반복적인 borrowed-ID 요청으로 장부를 늘리지 않습니다.
- 실제 두 AppKit view를 먼저 가린 뒤 두 producer에 job을 전달합니다. 한 타일만 준비됐거나
  full stamp·실제 drawable 크기가 다르면 두 타일 모두 가립니다. 두 readiness와 stamp가 일치할
  때만 같은 Core Animation transaction에서 가림을 해제합니다. producer readiness나 이
  transaction은 소비자 GPU fence 또는 같은 scanout 시점의 증명이 아닙니다.
- geometry 변경과 view 분리도 peer를 함께 가립니다. 오래된 visibility serial은 새 가림 상태를
  바꾸지 않습니다. 새 serial의 잘못된 uncover는 serial을 소비하고 두 타일을 계속 가립니다.
- 개별 member의 submit/status/cover/close/ack는 거절합니다. `closeGroup`은 두 gate를 함께
  닫고 두 layer를 분리합니다. `retireGroupAck`는 두 owner 종료가 모두 확인된 뒤 두 receipt를
  준비해 함께 확정합니다. 하나만 ack한 중간 상태는 외부에 노출하지 않습니다.

채널 method는 `createGroup`, `submitGroup`, `statusGroup`, `setCoveredGroup`, `closeGroup`,
`retireGroupAck`입니다. 생성 이후 요청은 `groupId`와 `deviceGeneration`을 함께 보내며,
상태 응답의 `members` 두 항목에는 각각 handle·full stamp·전체 accepted snapshot 목록이
들어 있습니다. terminal receipt는 Native owner를 보유하지 않으며 새 그룹 생성 시 이전 그룹
ID로 보낸 늦은 요청을 거절합니다.

## 로컬에서 실행할 수 있는 검사

```bash
c++ -std=c++17 -O2 -pthread -Wall -Wextra -Werror \
  bindings/Flutter/macos/tests/metal_tile_group_test.cc -o /tmp/rhwp-metal-tile-test
/tmp/rhwp-metal-tile-test
python3 -B bindings/Flutter/macos/tests/tile_group_qualification_test.py
python3 -B bindings/Flutter/macos/tests/sdk_qualification_contract.py
```

첫 검사는 제품의 C++ admission·visibility·retirement 함수를 직접 실행합니다. allocator 실패
전후 소유권, 400회 같은 ID 차용, 한 member의 미준비·이전 stamp, 부분 ack 실패를 검사합니다.
Python 검사는 실제 실행 영수증의 두 handle·공유 snapshot·정수 경계·drawable 크기·같은
stamp·두 view의 가림·종료를 검증하는 판정 함수의 양성/음성 대조입니다. 둘 다 Apple SDK나
GPU 실행을 뜻하지 않습니다.

[Apple SDK 도구](SDK_QUALIFICATION.md)는 새 `metal_tile_group_sdk_test.mm`도 컴파일하고
실행합니다. 실제 AppKit view와 Flutter method bridge에서 개별 member 호출 거절, peer cover,
종료·ack를 검사하지만 의도적으로 없는 Native library를 사용하므로 GPU 출력 증거가 아닙니다.
기존 실제 Native Metal producer 검사는 별도로 계속 실행합니다.

## 실제 Flutter·Native·AppKit 두 타일 실행

macOS 14 이상, 실제 Apple SDK·Flutter SDK·`native-skia-metal` Native dylib와 성공한
SDK 영수증이 필요합니다. 아래 경로는 전체 제품 경로를 실행하며 코드 대역이나 위젯 screenshot을
실제 소비 증거로 받아들이지 않습니다. Screen Recording 권한과 보이는 foreground window는
기존 [실제 native view 검증](NATIVE_VIEW_PRESENTATION.md)의 전제입니다.

```bash
python3 bindings/Flutter/macos/tests/qualify_native_view.py build \
  --flutter-sdk /SDK/flutter \
  --native /probe/librhwp_native_ffi.dylib \
  --native-receipt /probe/macos-sdk-receipt.json \
  --output /private/tmp/rhwp-metal-tile-build
python3 bindings/Flutter/macos/tests/qualify_native_view.py capture \
  --build-receipt /private/tmp/rhwp-metal-tile-build/macos-view-build.json \
  --tile-group --frames 32 --timeout 600 \
  --output /private/tmp/rhwp-metal-tile-capture
```

`--tile-group`은 실제 두 AppKitView를 생성합니다. 같은 immutable snapshot의 CPU replay와
ScreenCaptureKit의 전체 viewport를 비교하므로 두 타일의 경계도 비교 영역에 포함됩니다.
홀수 물리 폭 321 → 353의 타일 분할, 편집, resize, 연속 32회 publication, 두 native view를
mount한 채 유지하는 가림, 두 owner의 완료 ack와 Native 자원 계수 아홉 개 0을 요구합니다.
각 member의 실제 drawable 크기·stamp·snapshot ID·가림 serial·frame 수를 보존합니다.
다른 DPR은 해당 Mac 화면의 실제 backing scale에서 별도로 실행하고 영수증을 구분합니다.

빌드·capture 도구와 SDK 경로는 준비되어 있지만, 이 Draft의 Linux 개발 환경에서는 실제
Apple SDK 컴파일·두 타일 Metal 출력·ScreenCaptureKit 수집을 실행하지 않았습니다. 이
명령을 실제 target에서 완료하기 전에는 `tileGroupSamplingVerified` 성공으로 표시하지 않습니다.
물리 GPU 성능·장치 상실·장시간 IME/AT·최종 한컴 코퍼스 검증도 별도 범위입니다.
