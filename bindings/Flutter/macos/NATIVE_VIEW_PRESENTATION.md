# AppKitView·CAMetalLayer 참조 backend

`NativeTexturePlatform.macosMetalView`는 stock Flutter의 `AppKitView`로 실제
`NSView`를 배치하는 명시적인 실험 경로입니다. 기존 `texture_macos`의 불변
CVPixelBuffer backend와 Metal feature default는 유지합니다.

macOS platform view는 [공식 Flutter hybrid composition 경로](https://docs.flutter.dev/platform-integration/macos/platform-views)입니다.
현재 Flutter의 macOS gesture 지원에는 제한이 있으므로 이 view는 `hitTest:`에서
nil을 반환하고 first responder를 얻지 않습니다. 문서 입력·선택·IME의 소유자는
기존 Flutter 입력 view입니다. Flutter cover와 native view의 실제 가림·좌표·DPR은
각 target SDK에서 실제 compositor capture로 확인해야 합니다.

## 생성과 소유권

channel은 `rhwp_native/metal_view_macos`, factory type은
`rhwp_native/metal_view`입니다. channel의 `textureId`는 이 경로에서는 Flutter
texture registry ID가 아니라 native view state의 양수 handle입니다.

1. `create {libraryPath}`가 `textureId`, `deviceGeneration`,
   `presentationKind: nativeView`, `viewType`을 반환합니다.
2. Dart는 `AppKitView`를 만들고 `creationParams: {handle: textureId}`를 넘깁니다.
   하나의 handle은 NSView 하나에만 bind됩니다. duplicate/expired binding은
   다른 정상 view를 닫거나 다시 사용하지 않습니다.
3. 기존의 `submit {textureId, stamp, width, height, clearColor, placements}`와
   `status {textureId}`, `close {textureId}`를 사용합니다. view가 attach되기 전
   수락한 최신 요청은 pending으로 보존하며 실제 layer/geometry가 준비될 때까지
   UI thread를 막지 않습니다.
4. 필수 가림 계약인 `setCovered {textureId, covered: bool, visibilitySerial, stamp?}`은 native NSView도 가립니다.
   `covered: false`는 현재 전체 Native stamp가 유효한 게시 프레임이 있을 때만
   허용합니다. visibilitySerial은 증가하는 양수이고 오래된 요청은 새 가림을 풀 수
   없습니다. 응답은 `textureId`, `covered`, `visibilitySerial`, `accepted`,
   `renderedStamp`입니다. 가림은 GPU 완료나 frame credit 환급 신호가 아닙니다.
5. close 후 `ownerFinished: true`, `registered: false`,
   `nativeViewLayerDetached: true`를 확인하면 `retireAck {textureId}`를 보냅니다.
   성공 응답은 동일 `textureId`와 `acknowledged: true`, `closed: true`,
   `ownerFinished: true`, `registered: false`, `nativeViewLayerDetached: true`입니다.
   미완료·quarantine은 `retirePending`이며 해당 상태와 한도를 보존합니다.

기존 Native loader, JSON/geometry parser, snapshot lease와 generation gate를
같은 translation unit에서 재사용합니다. 새 경로는 CV 불변 frame credit을 예약하지
않습니다. 별도의 Core Animation drawable 소유권을 사용하기 때문입니다.

## 연속 표시와 한도

[`CAMetalLayer`](https://developer.apple.com/documentation/quartzcore/cametallayer)는
자체 drawable pool에서 화면에 표시 중이지 않고 strong reference도 남아 있지 않은
대상만 다시 제공합니다. backend는 이 공식 소유권을 사용합니다.

- `maximumDrawableCount = 3`, `allowsNextDrawableTimeout = YES`로 설정합니다.
  자체 drawable pool을 만들거나 임의 consumer-retire API를 노출하지 않습니다.
- worker 하나가 drawable 하나만 빌립니다. `nextDrawable`은 worker에서만 호출하며,
  nil/resize mismatch이면 최신 요청만 제한적으로 재시도합니다.
- Native presenter가 사용하는 device와 queue를 고정합니다. Native의 전체 stamp·
  target·색/alpha/origin 및 producer 완료를 확인한 뒤, 같은 queue의 command buffer로
  drawable을 present합니다. 이 command buffer 완료를 확인한 후 로컬 drawable
  참조를 autorelease pool에서 신속하게 놓습니다. 화면 표시와 후속 재사용은
  Core Animation이 소유합니다.
- target은 BGRA8Unorm·premultiplied·sRGB이며 `framebufferOnly = NO`로 설정해
  기존 Native의 render-target/shader-read 계약을 충족합니다. layer는 opaque SDR로
  고정하고 extended dynamic range를 끕니다. `clearColor`는 opaque ARGB만 받습니다.
- 소유한 Metal layer는 `actionForKey:`의 전체 조회를 재정의하고 최종 no-action인
  `nil`을 반환합니다. delegate의 `nil` fallback이나 `NSNull` sentinel 반환과 구분합니다.
  이 native view의 생성·hidden 변경·resize/geometry·detach는 duration 0인
  `NSAnimationContext`와 `CATransaction.setDisableActions:YES` 범위 안에서 수행합니다.
  platform-channel 호출은 Flutter compositor의 별도 transaction 안에서 실행된다고
  가정하지 않습니다. 주변 animation context는 복원하며 ancestor나 다른 view는 바꾸지
  않습니다. 준비된 프레임이 가림 해제 중 배경색과 fade되는 것을 허용하지 않습니다.
- viewport는 각 변 4,096 pixel 이하, 합계 4,194,304 pixel 이하입니다.
  backing 크기는 공통 Dart controller와 동일하게 ceil합니다. 다른 크기의 이전
  drawable을 새 viewport로 clamp하거나 억지로 표시하지 않습니다.
- process-wide Native producer/layer slot은 기존 CV 경로와 합쳐 최대 두 개입니다.
  layer도 slot을 보유하므로 channel close만으로 아직 살아 있는 layer의 한도를
  초기화하지 않습니다. view당 drawable pixel 저장의 계획 상한은 48 MiB입니다.
  Core Animation·driver·Flutter 전체 process 메모리를 이 값으로 약속하지 않습니다.
- Native job은 표시 중인 job·worker job·최신 pending job을 유지합니다. 교체 경계에서
  old/new pending 또는 old/new displayed가 잠시 공존할 수 있으므로 최대 네 개의
  job ownership이 존재합니다. job당 placement/snapshot ID는 최대 여덟 개이며,
  Native와 Dart의 별도 snapshot quota는 더 일찍 요청을 거절할 수 있습니다.

따라서 이 backend의 `continuousPresentationSupported`는 true이지만
`consumerGpuCompletionVerified`는 false입니다. Core Animation의 안전한 drawable
반환을 이용하는 구현이며 임의의 FlutterTexture consumer fence를 얻은 것은 아닙니다.
`wholeFrameReadbacks`는 정상 경로에서 0이고, qualifier의 화면/CPU 비교는 별도입니다.

## 교체·종료·실패

새 요청은 기존 generation gate를 갱신하고 pending을 최신 요청 하나로 합칩니다.
worker가 오래된 요청을 완료해도 전체 source와 gate를 다시 확인해 표시를 거절합니다.
metadata `hasFrame`도 전체 Native stamp와 현재 gate를 검사합니다.
수락한 새 요청과 오래된 프레임의 status 판정은 native view를 즉시 가립니다.
이 내부 무효화는 visibility serial을 소모하지 않으며 새 full-stamp uncover가
수락될 때까지 가림을 유지합니다.

close는 먼저 새 요청을 막고 pending과 displayed lease를 해제합니다. native view를
가리고 Metal layer를 분리하며, 진행 중인 Native GPU 작업과 같은 queue의
presentation command가 끝난 뒤 Native presenter를 그 생성 thread에서 해제합니다.
`ownerFinished`와 `registered: false`는 이 Native bridge의 종료를 뜻합니다.
Flutter platform view 객체의 늦은 dispose나 화면 consumer GPU의 완료와 구분합니다.

활성 view와 아직 `retireAck`하지 않은 close record를 합쳐 최대 두 개 보존합니다.
두 view를 함께 닫아도 handle별 상태를 덮어쓰지 않습니다. worker가 완료됐어도
caller가 이를 확인하고 ack하기 전에는 record를 버리거나 새 create에 그 공간을
사용하지 않습니다. ack는 producer 완료·layer 분리·quarantine 없음이 모두
확인됐을 때만 수락합니다. 이후에는 Native owner/resource 참조가 없는 완료 status
metadata를 최근 ack 순서로 두 개만 보존해 반복 status·close·ack에 응답합니다.
더 오래된 ack의 status는 `closed` 오류로 만료를 명시합니다. 미완료 record를
내보내거나 완료 map을 무한히 누적하지 않습니다.

Native/command completion이 확인되지 않으면 drawable, queue, layer, job과 producer
thread/slot을 process lifetime으로 보존하고 명시적 오류를 보고합니다. close나
재생성으로 이 한도를 환급하지 않습니다. 불명확한 GPU 작업에 대해 대상 재사용이나
Native owner 종료를 추정하지 않습니다.

## 검증 상태

`tests/metal_view_lifecycle_test.cc`는 실제 backend가 사용하는 portable lifecycle과
geometry/generation gate를 검사합니다. 400번 handoff, 늦은 uncover 거절, 중복/옛 ticket 거절,
작업 중 close, 불명확한 완료의 quarantine, DPR/resize ceil과 최신 세대 보존을
확인합니다. 동시 close 두 개·완료 전 ack 거절·ack 전 create 제한·역순 완료·400회
닫기/재생성에서도 완료 metadata와 owner 상태가 정해진 한도를 지키는지 검사합니다.
이 검사는 Apple SDK compile 또는 실제 macOS 화면 수집의 대체가 아닙니다.

실제 Apple SDK compile/link, 실제 AppKitView 표시·32회 이상 갱신·revision·resize·
cover·close와 Native lease 정리는 별도의 target runner에서 수행해야 합니다.
그 결과가 없으면 실기기/SDK 검증 완료로 표시하지 않습니다.

`tests/metal_view_animation_sdk_test.mm`은 실제 AppKit/Core Animation에서 소유 layer의
action 차단과 view의 즉시 가림/geometry/detach, ambient animation context 복원을
검사합니다. 명시적인 CA fallback action과 animated transaction에서도 새 animation이
생기면 실패합니다. SDK 도구가 이 테스트를 compile/run하고
`metalViewAnimationPolicyVerified`를 기록합니다. 테스트의
`RHWP_METAL_ANIMATION_PLUGIN_SOURCE` include macro를 이전 source의 plugin 경로로
지정하면 같은 테스트로 수정 전 action lookup 실패를 대조할 수 있습니다. 이 검사는
GPU render나 실제 WindowServer 화면 검증을 대신하지 않습니다.

`tests/qualify_native_view.py`는 실제 macOS 14 이상에서만 실행되는 build/capture
도구입니다. 먼저 같은 Flutter framework/engine으로
`tool/check_macos_metal_sdk.py`를 실행해 실제 Native dylib와
`macos-sdk-receipt.json`을 준비합니다. 이후 다음처럼 실행합니다.

```sh
python3 bindings/Flutter/macos/tests/qualify_native_view.py build \
  --flutter-sdk /absolute/flutter \
  --native /absolute/librhwp_native_ffi.dylib \
  --native-receipt /absolute/macos-sdk-receipt.json \
  --output /absolute/private/macos-view-build
python3 bindings/Flutter/macos/tests/qualify_native_view.py capture \
  --build-receipt /absolute/private/macos-view-build/macos-view-build.json \
  --output /absolute/private/macos-view-capture --frames 32 --timeout 600
```

capture에는 Pillow와 Screen Recording 권한이 있는 실제 WindowServer 세션이
필요합니다. 프로세스가 foreground가 아니거나 권한·창·Metal이 준비되지 않으면
실패로 기록합니다. TCC 설정을 바꾸거나 다른 화면 수집 방식으로 성공을 대신하지
않습니다. Swift helper는 ScreenCaptureKit의 실제 창을 BGRA·sRGB SDR로 수집한 뒤
명시적 sRGB CGContext/PNG로 정규화합니다. 출력 색을 임의 보정하거나 공통 AA
허용 범위를 넓히지 않습니다.

build는 clean source의 추적 파일을 private copy로 고정해 disposable borderless
Flutter runner를 생성합니다. 외부 Native/fixture 파일을 읽기 위해 이 runner의
Debug sandbox만 해제합니다. 제품 example과 production entitlement는 수정하지
않습니다. 실제 실행되는 Dart kernel, 모든 framework/data 파일과 framework의
내부 상대 symlink를 포함한 전체 `.app` manifest를 기록하고 capture 전후에
대조합니다. 외부·절대·깨진 symlink와 바뀌거나 추가된 실행 입력은 거절합니다.

동일한 snapshot의 독립 CPU replay와 고해상도 CPU source reference를 공통 pixel
검사에 사용합니다. 32회 publication에는 실제 source revision과 viewport resize가
포함됩니다. 각 단계에서 연속된 두 실제 화면이 byte 단위로 같아야 합니다.
실제 실행의 31번째 단계에서는 같은 geometry의 두 화면에서 RGB 최대 1 차이가
관측됐으며, 정확한 발생 원인은 확정하지 않았습니다. 모든 수집 화면을 원래 CPU
snapshot과 독립 source reference로 먼저 검사한 뒤, metadata·alpha가 같고 전체
sample의 RGB 최소·최대 폭이 1 이내인 경우에만 최대 네 번까지 수집해 연속된 exact
pair를 확인합니다. CPU 불일치·폭 1 초과·metadata/alpha 변화는 즉시 실패하며,
네 번 안에 exact pair가 없어도 실패합니다. 기존 pixel 허용 범위는 바꾸지 않습니다.
선택 전 sample도 PNG·hash·CPU 판정과 `held-pair-N.json`에 보존합니다.
첫 publication 뒤에는 controller를 pause한 채 같은 AppKitView를 mount 상태로
유지하고, Native cover ack와 두 Flutter paint frame 뒤의 창을 두 번 수집합니다.
이때 viewport 전체가 지정한 opaque Flutter cover와 정확히 일치해야 합니다.
이 검사는 Native hide와 Flutter opaque cover를 함께 적용한 경로를 검증합니다.
아직 표시 중인 NSView를 Flutter overlay 하나만으로 가릴 수 있다는 z-order 보장은
하지 않으며, 결과에도 `flutterOverlayOccludesVisibleNativeViewVerified: false`를
남깁니다. controller의 Native cover ack를 생략하면 안 됩니다.
마지막에는 producer 종료, layer detach, registration 해제와 모든 Native lease
counter 0을 확인합니다. 이 화면 검증도 consumer GPU fence를 증명하지 않습니다.

`tests/native_view_qualification_test.py`의 portable 검사는 번들·프로세스·stamp·
한도·cover·close 영수증의 음성 대조군을 확인합니다. 실제 macOS build/capture를
실행하지 않으며 해당 검증의 성공 영수증을 만들지 않습니다.
