# P100 macOS Metal 참조 브리지 — Draft/WIP

선택 피처 `native-skia-metal`의 Ganesh renderer와 공식 `FlutterTexture` 기반 macOS plugin이다.
문서의 immutable P89 snapshot을 GPU scratch에 strict replay한 뒤 새 BGRA `CVPixelBuffer`의
`CVMetalTexture`로 그린다. 프로듀서 GPU 완료를 기다리는 전용 native thread를 사용하며,
UI thread와 raster callback은 파싱·조판·GPU 대기를 수행하지 않는다. 픽셀을 CPU로 읽거나
platform channel JSON으로 보내는 경로는 없다. Flutter 내부 `SkCanvas*`, Skia 버전, Impeller
context를 공유하지 않는다.

**연속 GPU 표시는 ARCH-017로 차단돼 있다.** 공개 `FlutterTexture` 계약에는 소비자 GPU 완료를
관찰하는 수단이 없다. 최대 네 개 프레임 / 64 MiB 예약 바이트를 프로세스 종료까지 변경 없이
보관하고, 그 뒤에는 `backpressure`를 반환한다. 해제되지 않은 프레임을 임의로 재사용하거나
unregister를 fence로 처리하지 않는다. 이 단계는 지속적인 데스크톱 GPU 뷰 완성이 아니다.

## API와 스레드

`bindings/Native/src/gpu_metal.rs`는 항상 다음 C 함수를 export한다. macOS와 선택 피처가 모두
없으면 GPU 포인터나 요청 span을 읽지 않고 `featureDisabled`를 반환한다.

| 함수 | 호출 스레드와 소유권 |
| --- | --- |
| `rhwp_metal_capabilities()` | 임의 스레드의 작은 feature probe |
| `rhwp_metal_presenter_create(device, queue, deviceGeneration, config, len)` | 전용 producer OS thread. 실제 `id<MTLDevice>`와 같은 device의 `id<MTLCommandQueue>`를 전달한다. Ganesh가 둘을 retain한다. |
| `rhwp_metal_presenter_render(presenter, texture, request, len)` | 같은 producer thread. 실제 `id<MTLTexture>`를 call 동안 보유한다. 주소는 JSON에 넣지 않는다. |
| `rhwp_metal_presenter_request(presenter, request, len)` | 같은 producer thread의 `info` / `release` / `abandon` |
| `rhwp_metal_frame_check(request, len)` | 임의 스레드. `{stamp,snapshots:[ID...]}`의 source와 view/device 세대만 검증하며 Core/renderer를 호출하지 않는다. |

모든 응답 문자열은 `rhwp_string_free`로 해제한다. 요청/config는 최대 64 KiB, 프레임은
최대 4096 차원·4,194,304 pixels·16 MiB, 배치는 최대 8개, presenter는 thread마다 최대 2개다.
renderer는 기존 `NativeRenderSnapshot::create_renderer`와 `render_to_surface_strict`를 소비한다.
Core·편집기·폰트 모델을 다시 구현하지 않는다. renderer 한 개, scratch 한 개, resource/font/cache
예산을 별도로 유지하며 이 논리 예산을 총 RSS/driver VRAM의 상한이라고 주장하지 않는다.

`skia-safe` 0.156의 `mtl::BackendContext::new`와 `make_metal`은 각자 device/queue 참조를 보유한다.
target은 `BGRA8Unorm`, premultiplied alpha, sRGB, top-left, 2D, single sample,
render-target와 shader-read usage로 고정한다. `flush_surface`의 `Result`와
`submit(SyncCpu::Yes)`의 bool을 모두 검사한다. 이 wait는 producer queue 완료만 증명한다.
source가 GPU wait 도중 바뀌면 성공 프레임을 전달하지 않는다. stale 검사 위치가 target 사용 전 또는
성공한 GPU wait 뒤인 경우에만 `unpublishedTargetSafeToDiscard:true`를 반환해 미공개 target을
폐기하고 크레딧을 환급한다. GPU 완료가 불명확한 실패 target도
같은 bounded 보관 한도에 격리하고 CPU 경로로 돌아가야 한다.

## Flutter channel

Plugin class는 `RhwpMetalPlugin`, channel은 `rhwp_native/texture_macos`다. macOS podspec과 source만
제공하며 기존 P97 Dart/pubspec 등록은 변경하지 않는다. 통합 시 macOS plugin 등록을 추가하고,
별도 runner의 generated registrant 또는 `registerWithRegistrar:`로 등록한다. 이 디렉터리를
`flutter create .`로 덮어쓰지 않는다. macOS deployment target은 12.0이다.

이 ObjC++ CocoaPods target은 `FlutterMacOS`를 명시적으로 framework link한다.
Flutter 3.47의 SwiftPM host와 CocoaPods plugin을 함께 쓰는 경우에도 필요하다.
`s.dependency 'FlutterMacOS'`는 Flutter의 placeholder pod와 framework 검색 경로를
설정하지만, 순수 ObjC++ pod의 dynamic link를 Swift import의 autolink로 대신할 수
없다. engine의 실제 framework 배치는 Flutter tooling에 맡기고, unresolved symbol을
허용하거나 자체 Flutter runtime을 중복해서 배포하지 않는다.

| Method | 인자 / 결과 |
| --- | --- |
| `capabilities` | 인자 없음. 프레임 크레딧과 `consumerRetirement:false`, `continuousPresentationSupported:false`, `cpuFallbackRequiredWhenSaturated:true` 반환 |
| `create` | `{libraryPath:absolute,closeAcknowledgementRequired?:bool}` → `{textureId,deviceGeneration,initializing:true,closeAcknowledgementRequired,limits}`. dylib load·Metal device/context 생성은 producer thread에서 실행한다. `status.ready` 전 submit은 거절한다. |
| `submit` | `{textureId,stamp,width,height,placements,clearColor}` → `{accepted,acceptedStamp,reason,limits}`. stamp는 P89의 document/revision/font/view/device/deviceScale 전부다. deviceGeneration은 create 결과를 사용한다. placements는 `{snapshot,scale,originX,originY,backgroundColor?}` 최대 8개다. |
| `status` | `{textureId}` → ready/closed/ownerFinished/pending/inFlight/hasFrame/staleFramesDropped/error/deviceGeneration/acceptedStamp/acceptedSnapshots/limits. 결과는 단일 bounded 상태다. |
| `close` | `{textureId}` → 즉시 `{closed:true,ownerCompletionPending,limits}`. platform thread에서 unregister하고, producer thread는 실행 중 GPU 작업과 Native presenter 정리를 마친 후 종료한다. 최근 closed ID 하나는 status/idempotent close를 위한 bounded receipt로 남긴다. |

새 클라이언트는 create에 `closeAcknowledgementRequired: true`를 요청하고 응답의
top level과 `limits`에 같은 true가 있을 때만 종료 확인 계약을 활성화한다.
협상한 모든 `submit`·`status`·`close`·`retireAck` 요청에는 `textureId`와
`deviceGeneration`을 함께 보낸다. Flutter texture ID는 객체 주소여서 재사용될 수
있으므로 이전 세대의 늦은 요청이 새 texture를 닫거나 그 상태로 응답하지 않는다.
누락되거나 형식이 잘못된 세대는 `invalidArgument`, 서로 다른 ID·세대는
`contextChanged`로 enqueue 전에 거절한다. 공통 Dart 클라이언트는 이 명확한
사전 거절에서 아직 전달하지 않은 snapshot을 한 번 해제할 수 있다.

협상 경로는 활성 texture와 아직 확인하지 않은 close record를 합쳐 최대 두 개
보존한다. 두 close가 겹쳐도 handle·세대별 미완료 상태를 덮어쓰지 않는다.
`closed: true`, `ownerFinished: true`, `registered: false`를 확인한 다음
`retireAck {textureId,deviceGeneration}`를 보내야 record를 해제한다. 응답은 동일
ID·세대와 `acknowledged: true`, `closed: true`, `ownerFinished: true`,
`registered: false`다. 미완료 ack는 `retirePending`이며 상태와 한도를 보존한다.
그 뒤에는 Native owner 참조가 없는 완료 metadata를 최근 ack 순서로 두 개만
남겨 반복 status·close·ack에 응답한다. 협상하지 않은 기존 호출 형식은 유지한다.
완료된 owner 정보는 고정하지만, cached status·close 응답의 `limits`는 조회 시점의
전역 credit 값으로 갱신한다. 다른 texture의 새 예약을 이전 0 값으로 숨기지 않는다.

이 확인은 plugin registration 종료와 producer thread 종료만 다룬다. 공개 macOS
registrar의 unregister는 consumer GPU fence나 `onTextureUnregistered` 호출을
보장하지 않으므로 `textureUnregisteredNotification`을 완료의 필수 조건으로 삼지
않는다. 기존의 네 immutable CV frame·64 MiB credit은 ack 뒤에도 환급하지 않으며
`consumerGpuCompletionVerified`와 `continuousPresentationSupported`는 false다.
`tests/texture_close_routing_test.cc`는 실제 production identity router와 retirement
registry를 사용해 동시 종료, ID 재사용, 늦은 이전 세대 요청, 기존 클라이언트,
400회 ack와 immutable credit의 미환급을 portable 환경에서 검사한다.

배치의 scale에는 deviceScale을 포함하고, origin은 target pixel 좌표다. 96-DPI 문서 좌표를
사용하는 UI는 `pixelScale = zoom * deviceScale`, `pixelOrigin = logicalOrigin * deviceScale`로
변환하며 두 번 곱하지 않는다. 요청을 최신 pending 하나로 합치고 실행 중 하나만 허용한다.
오래된 document/revision/font/view/device 세대의 완료는 표시하지 않는다. 프로세스 내 active
producer는 최대 2개이며 register/unregister는 공식 계약대로 platform thread에서 수행한다.

`accepted:true`부터 신규 snapshot ID의 소유권은 plugin으로 이전한다. 거절하면 호출자에게
남는다. 이미 plugin이 보유한 ID는 view 변경에서 재사용할 수 있다. pending 교체·오류·close도
이전받은 참조를 해제한다. 채널 요청 하나를 차례로 기다리고, 응답 유실 시 `status`의
`acceptedStamp`와 `acceptedSnapshots`로 판정한다. Dart가 dispose됐다는 이유만으로 늦게 도착한
accepted ID를 다시 해제하면 안 된다. Native library는 P97 owner dispatcher와 **같은 canonical
dylib 파일**이어야 한다. legacy worker/source configure/capture는 계속 P97 owner RPC를 사용한다.

`copyPixelBuffer`는 준비된 immutable pixel buffer를 +1 retain해 반환하고, bounded source/stamp
조회만 수행한다. 공개 `snapshotInfo.current`는 view/device까지 검증하지 않으므로 별도
`rhwp_metal_frame_check`가 기존 registry의 `borrow_snapshot` 검증을 소비한다. stale이면 nil을
반환한다. **nil은 Flutter가 이미 캐시한 이전 texture를 지운다는 뜻이 아니다.** Dart 통합은
문서·뷰 교체, 오류, closed, backpressure에서 Texture 위젯을 숨기고 P97 CPU 참조 경로를
표시해야 한다. 계속 GPU 프레임을 재요청하는 retry loop를 만들지 않는다.

## 소비자 수명 근거와 ARCH-017

확인한 엔진은 Flutter 3.47.9 revision `c197ca5199e9ee5661e865596c9c07294dfab51c`다.

- [공식 FlutterTexture.h](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/darwin/common/framework/Headers/FlutterTexture.h)는 BGRA CVPixelBuffer와 raster callback, platform 등록/해제를 정의하지만 consumer command-buffer 완료를 노출하지 않는다.
- [macOS FlutterExternalTexture.mm](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/darwin/macos/framework/Source/FlutterExternalTexture.mm)의 `populateTexture`는 반환 buffer를 지역 CFRef로 보유하고, Metal handle을 export한 뒤 CVMetalTexture를 release한다. 이 사실만으로 앱이 버퍼를 다시 쓰거나 소비자 GPU 완료를 판정할 수 없다.
- [공식 Darwin Metal wrapper](https://github.com/flutter/flutter/blob/c197ca5199e9ee5661e865596c9c07294dfab51c/engine/src/flutter/shell/platform/darwin/graphics/FlutterDarwinExternalTextureMetal.mm)는 Skia/Impeller 경로가 다름을 보여 준다. plugin은 특정 내부 renderer를 전제로 하지 않는다.
- [Apple CVMetalTextureCacheCreateTextureFromImage](https://developer.apple.com/documentation/corevideo/cvmetaltexturecachecreatetexturefromimage(_:_:_:_:_:_:_:_:_:))는 GPU가 texture에 접근하는 동안 CVMetalTexture를 강하게 보유할 책임을 호출자에게 둔다. [Apple WWDC20의 설명](https://developer.apple.com/videos/play/wwdc2020/10090/?time=1302)도 command buffer 완료 뒤 반환하는 수명을 설명한다.
- [CVPixelBufferPool allocation threshold](https://developer.apple.com/documentation/corevideo/kcvpixelbufferpoolallocationthresholdkey)는 할당 수를 제한하지만 이 consumer fence를 제공하지 않는다. 이 구현은 pool을 쓰지 않으며 프레임을 제자리에서 다시 쓰지 않는다.

따라서 현재 프레임은 `CVPixelBuffer`·`CVMetalTexture`·`MTLTexture`를 강하게 보유한 채 프로세스
수명 보관소에 둔다. copy callback 반환, producer 완료, unregister, plugin dealloc, document close,
device 교체 어느 것도 크레딧을 환급하지 않는다. 공개 manual-retire API도 없다. 후속에서 검증된
consumer 완료 계약을 확보한 뒤에만 이 정책과 continuous capability를 변경할 수 있다.

## 검증

Linux에서 실행 가능한 실제 공통 수명 코드 검사:

```bash
c++ -std=c++17 -Wall -Wextra -Werror -pthread tests/frame_lifetime_test.cc -o /tmp/rhwp-metal-lifetime
/tmp/rhwp-metal-lifetime
python3 tests/native_metal_disabled.py --library /absolute/path/non-metal/librhwp_native_ffi.so
```

첫 검사는 plugin이 실제 사용하는 `MetalFrameLifetime.hpp`의 byte/count 상한, unpublished 환급,
공개/격리 프레임의 미환급, 동시 admission, source/device 세대와 close를 검사한다. 두 번째는
새로 빌드한 비활성 Native ABI에서 feature probe, 잘못된 typed pointer/span의 비역참조,
cross-thread probe/string 해제를 검사한다. 기존 P83 library에는 이 신규 symbol이 없으므로
그 library로 비활성 ABI 검사를 통과했다고 기록하면 안 된다.

macOS SDK/Metal hardware가 없는 환경에서 실행하지 않은 항목:

1. `native-skia-metal` arm64/x86_64 빌드·Clippy와 실제 Apple SDK Objective-C++/pod 빌드.
2. 실제 snapshot으로 Metal target에 그린 결과의 CPU/기준 이미지 대조, BGRA·alpha·sRGB·DPR·origin.
3. Flutter desktop runner에서 공식 texture registration/populate/close·device 교체·late completion.
4. 실제 device-loss/submit failure, close 중 GPU 작업, 네 프레임 뒤 CPU fallback와 지속 메모리 상한.
5. ARCH-017을 해결할 consumer 완료 계약. 이를 입증하기 전 연속 GPU 표시 qualification은 차단된다.

위 OS 검사는 작업지시자가 나중에 수행하는 별도 qualification이다. portable 검사나 비활성 ABI
통과를 Metal backend 컴파일·실제 GPU 표시 성공으로 대체하지 않는다.
