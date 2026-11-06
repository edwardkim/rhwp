---
kind: reference
status: active
canonical: mydocs/tech/native_gl_recovery.md
last_verified: 2026-11-07
---

# Native GL context 회복과 품질 acceptance

GL owner는 frame 오류를 모두 device loss로 취급하지 않는다. [불변 입력/CPU lease](native_presentation.md),
[host FBO 게시](native_gl_presenter.md), [GPU tile admission](native_gl_tiles.md) 경계를
유지하면서 context lifecycle과 실제 CPU/GL 합성 품질을 별도로 확인한다.

## typed 실패와 host 정책

GL C 응답은 `code/error/failureKind/domain/deviceFailure/requiresRecreate/retryable`을
반환한다. Rust의 `NativeGlError/NativeGlFailureKind`도 같은 분류를 제공한다. source
borrow에는 typed invalid-handle/stale 경로를 추가하고 기존 P89 API 오류 코드는 보존한다.

| 상황 | code와 의미 |
| --- | --- |
| strict page/font/image/resource | `documentError`, domain `document` |
| byte/count admission | `limitExceeded`, domain `resource` |
| source/view가 달라짐 | `staleFrame`, `retryable: true` |
| matching current 부재 | `contextNotCurrent`, current 이벤트 후 명시적 재시도 가능 |
| 다른 identity/generation | `contextMismatch`, 입력/owner 선택을 고쳐야 함 |
| 이미 retired/abandoned | `contextUnavailable`, 새 context로 명시적 recreate 필요 |
| native factory/host FBO wrap의 미확정 실패 | `contextCreationFailed`/`targetUnavailable`, device/메모리 원인 미확정 |
| Skia가 보고한 실제 loss | `deviceLost`, `deviceFailure: true` |
| target allocation 실패 | `allocationFailed`, domain `resource`, loss 미확정 |
| flush/submit 실패·loss 미확정 | `submissionFailed`, `deviceFailure: false` |
| interface/backend 구성 불가 | `backendUnavailable`, 물리 device loss 인증은 아님 |
| 재사용한 identity/진행하지 않은 generation | `recoveryRejected` |
| 승인된 전환 뒤 새 owner 구성 실패 | `recoveryFailed`, handle에 실패 상태 보존 |
| live+orphan context quota | `recoveryBudgetExhausted`, domain `resource` |

`deviceFailure: true`는 실제 render/retire 중 Skia `is_device_lost`가 loss를 보고할 때만
낸다. 합성 오류나 단순 allocation/submit 실패를 그 값으로 바꾸지 않는다. late strict
오류는 visible FBO를 바꾸지 않는다. GPU failure가 게시 시작 뒤 발생한 경우에는 이미
제출한 pixels를 되돌릴 수 없다.

Native 계층은 자동 retry loop나 숨은 CPU fallback을 만들지 않는다. 호스트는 stale이면
최신 입력/configure를 기다리고, strict/resource 오류는 진단과 이전 화면 보존으로 다룬다.
단순 `domain == gpu`나 모든 `unsupported`를 device failure로 간주하지 않는다. 명시적으로
선택한 `auto` renderer 정책도 실제 backend/context 사용 가능 상태와 확인된 device
분류를 소비해야 하며 generic allocation/문서 오류를 조용히 CPU로 바꾸지 않는다.

## retire, recreate, 실패 상태

정상 종료의 `release`는 matching context가 current인 상태에서 자원을 정리하고 TLS
handle을 없앤다. 새 `retire`는 같은 검증/정리를 수행하되 handle을 `released` 상태로
유지한다. 이후 새 context에서 같은 handle을 recreate하거나 `abandon`으로 남은 handle을
없앨 수 있다. Python wrapper의 retired `release()`도 handle만 제거한다.

```json
{"op":"retire","contextId":1,"deviceGeneration":1}
```

GPU 정리는 먼저 actual current를 확인하고 Skia loss를 검사한다. 이미 loss가 보고되면
정상 정리가 증명되었다고 credit을 반환하지 않고 abandon으로 전환한다.

새 context를 current로 만든 뒤 callback을 JSON이 아닌 typed C 인자로 다시 제공한다.

```c
char *rhwp_gl_presenter_recreate(uint64_t presenter, uint64_t context_id,
    uint64_t device_generation, RhwpGlGetProcAddress loader,
    RhwpGlIsCurrent is_current, void *user_data,
    const uint8_t *config, size_t config_len);
```

context ID는 이전 것과 마지막 시도 것 모두와 달라야 하고 device generation은 마지막
시도보다 커야 한다. actual new current는 host callback이 정확히 확인한다. same-handle
callback/user_data는 성공 전까지 이전 것을 유지하고, 성공하면 새 것으로 교체한다.
빈 config는 이전 limits를 유지한다. source/view/device stamp의 별도 owner-thread
`presentationConfigure`도 새 세대로 진행해야 frame을 게시할 수 있다.

순서는 입력 검증→새 context credit 예약→old context abandon→새 native 구성이다.
입력/current/quota 거절은 기존 handle을 바꾸지 않는다. 승인된 구성 실패는 old handle을
`recoveryFailed`로 유지하며 `info.recovery`에 시도 수·마지막 context/generation과 원인
failure를 남긴다. 그 상태에서 옛 GL 호출을 재개하지 않는다. 다시 실제 새 context와
더 큰 generation을 제공하거나 handle을 abandon해야 한다. 성공한 새 세대는 GPU
pool/cache·device별 제출 계수를 초기화하고 recovery 이력을 이어 받는다.

## process live/orphan credit

한 native library instance의 process credit 총량은 **live+unproven orphan 4개**다.
config/identity/GL string/interface 검증처럼 GPU object를 만들지 않는 probe는 credit을
소모하지 않는다. native context factory를 시도하기 직전에 예약하며, factory가 GPU를
건드린 뒤 실패해 정상 current cleanup을 입증할 handle이 없으면 보수적으로 orphan이다.

정상 matching-current retire/release만 credit을 돌려준다. context가 없어진 뒤의
abandon/Drop, old context의 정리를 입증하지 않은 recreate는 credit을 permanent orphan으로
남긴다. old share-group가 살아 있으면 실제 driver GL 이름도 남을 수 있다. 새 세대의
logical target 수를 0으로 초기화했다고 그 메모리까지 회수되었다고 주장하지 않는다.
context destruction JSON boolean이나 단순 destructor 호출을 forgiveness로 쓰는 API는 없다.

새 context의 credit을 먼저 확보하므로 한도가 가득 차면 old handle을 버리기 전에
추가 native GPU 구성을 거절한다. `info.contextCredits`와 `gpuGl.contextCredits`는
`limit/inUse/orphaned`를 일관된 snapshot으로 보고한다. `recovery/retire/typedFailures`
capability도 명시한다. 정상 retire→실제 context 교체→recreate는 반복해도 orphan을 늘리지
않는다. 미입증 abandon이 누적되면 더 이상 GPU를 할당하지 않으며 자동 재시도하지 않는다.

이는 native가 소유한 context/target 수명의 admission이다. 호스트 소유 FBO, Flutter
texture pool, driver 지연 삭제·share-group의 모든 자원이나 전체 VRAM을 측정하는 상한이
아니다. UI/Dart/plugin은 같은 native shared library 인스턴스를 써야 registry/credit을
공유한다. 별도로 정적 링크한 사본과 다른 SO의 전역 상태는 공유되지 않는다.

특히 public Linux `FlTextureGL` 경로는 close 후 다음 populate가 오지 않을 수 있고
일반적인 destruction callback이 없다. 원 owner thread/current에서 정리가 증명되지
않은 slot은 plugin의 bounded orphan 보유나 owner TLS 종료까지 남겨야 한다. CPU lease
반환이나 UI close만으로 texture/fence 완료 credit을 돌려주지 않는다. plugin의 별도
orphan budget은 이 native credit과 독립이며 더 보수적인 제한으로 유지한다.

## 독립 CPU 합성과 품질 기준

`renderCpuBatch`는 같은 snapshot placements를 GL/Linux와 무관한 borrowed RGBA CPU
Canvas에 strict replay한다. native frame/lease 한도를 재사용하고 모든 source stamp를
확인한 뒤에만 producer를 게시한다. 같은 physical viewport·frozen font·scale/origin/
background를 GL FBO 결과와 비교할 수 있다. Core/조판 조회는 worker snapshot export에
남고 CPU/GL replay가 UI에서 layout을 실행하지 않는다.

quality probe는 GPU의 raw stored RGBA를 별도 호스트 `glReadPixels`로 읽고 bottom-left
행을 top-left로 바꾼다. Qt QImage의 alpha 변환을 CPU 참조로 사용하지 않는다. CPU lease는
sRGB, premultiplied RGBA8, top-left/stride를 명시하며 GL도 같은 계약을 보고한다.

acceptance는 uniform 3×3 내부의 각 encoded 8-bit 채널에 **1 LSB**를 허용한다. 서로 다른
CPU/GPU blending quantization을 구분하기 위한 제한이며 평균 오차로 큰 내부 색 오류를
숨기지 않는다. glyph/image AA boundary의 차이는 별도 최대값을 보고하고 CPU boundary의
**1 device pixel support** 밖으로 새지 않아야 한다. 여러 page 순서, 음수/fractional
origin·scale, transparent/partial-alpha viewport clear와 premul `RGB <= A`를 함께 검사한다.
전체 HWP 문서의 한컴 시각 일치율이나 glyph 경계의 완전한 pixel identity 인증을 대신하지 않는다.

fractional page clip의 alpha를 frame 전체에서 CPU와 동일하다고 가정하지 않는다.
locked `skia-safe/skia-bindings 0.156.0`의 Skia tag는 `m156.5`다. CPU의
[SkRasterClip.cpp:141–158](https://github.com/rust-skia/skia/blob/m156.5/src/core/SkRasterClip.cpp#L141-L158)은
non-AA clip을 device rectangle의 `round()`로 region에 넣는다. Ganesh의
[GrClip.h:94–110, 158–178](https://github.com/rust-skia/skia/blob/m156.5/src/gpu/ganesh/GrClip.h#L94-L178)은
GPU raster rounding의 불확실성을 위해 half-pixel tolerance `0.05`, bounds tolerance
`0.001`과 서로 다른 내부/외부 pixel bounds를 명시한다. host surface의 non-AA clip
요청을 서로 다른 backend의 모든 경계 byte가 같다는 계약으로 확대하지 않는다.

실제 Mesa 22.3.6 llvmpipe 검사에서 `(32.5, 24.25)`의 작은 page는 `x=32,
y=24..164`의 141개 pixel에서 CPU alpha 0, GL alpha 255였다. `(32.75, 24.75)`의
내용 없는 page와 `clearColor=0x40112233`에서는 premul clear `(4,9,13,64)`와 white
paper `(255,255,255,255)` 사이의 GL 경계 값 `(67,71,74,112)`도 관찰됐다. 차이는
분수 좌표가 지나는 pixel cell 안에만 있었으며 내부·외부의 정확한 색은 바뀌지 않았다.
이는 software GL에서 확인한 backend 경계 동작이며 hardware device loss나 성능 증거가 아니다.

probe는 replay 전에 worker에서 얻은 page size와 실제 placements의 f32 physical geometry로
각 pixel cell을 분류한다. CPU/GPU 출력으로 허용 mask를 만들지 않는다. page와 양의 면적으로
일부만 겹치는 cell에서만 clear/white-paper endpoint 사이 **같은 혼합 비율의 premul RGBA**를
허용하고, encoded 채널의 양자화 차이는 최대 1로 제한한다. 완전히 들어간 불투명 paper의
alpha와 완전히 벗어난 clear는 정확해야 한다. 내용 없는 대조군의 내부·외부는 RGBA 전체가
정확한 endpoint와 같아야 한다. 정수 clip은 경계의 모호한 cell도 없으므로 전체 결과를
정확히 비교한다. nested page에서 뒤 page가 완전히 덮은 cell은 앞 경계의 모호함을 남기지 않는다.

독립 blank 대조군 15개는 정수·1/4·1/2·3/4 좌표, half-pixel 직전·직후, 음수 origin의
오른쪽·아래 clip, transparent/opaque/partial-alpha viewport clear, host page preclear 생략,
nested page 합성을 검사한다. 1 pixel 이동, 내부 alpha 변조, page 밖 pixel 변조를
거절하는 별도 음성 대조군 3개도 실행한다. 원래 picture/table/서로 다른 text page의 여섯
입력은 유지한다. `clipBoundaryMaxDifference/nonClipMaxDifference`는 page 경계 혼합과
나머지 paint 차이를 구분하며 `ambiguousBackgroundCells/alphaDifferentPixels`도 보고한다.

이 corpus의 Screen `PageLayerTree`에는 기본 흰 종이 `PageBackground`가 있다
([layout::build_page_background](../../src/renderer/layout.rs)). `backgroundColor: null`은
host의 추가 page clear를 생략하며, authored 흰 종이 paint를 제거하지 않는다. 따라서
probe의 해당 경우는 transparent paper 구현을 인증하지 않고 transparent/partial-alpha
viewport clear 뒤에 흰 paper가 합성되는 경로를 검사한다.

`timingScope: hostWallClock`이며 replay 시간에는 tile miss의 canonical/copy submit도
포함된다. publish의 flush-submit 시간은 scratch/image/wrap/host copy 작업이다. 정상
`wholeFrameReadbacks/wholeFrameCpuUploads`는 0이고 resource별 CPU decode/upload는 별도다.
probe의 `hostVerificationReadbacks`와 CPU reference frame은 검증 작업으로 따로 계상한다.

## 검사 경로

공개 Rust 계약은 `tests/cases/issue_536_native_gl_recovery.rs`에서 feature-off CPU batch,
공유 viewport 한도, typed device/resource/stale 분리, premul CPU pixels/lease 수명을 검사한다.
실제 C ABI·GL은 별도 process에서 다음 명령으로 실행한다.

```sh
QT_QPA_PLATFORM=xcb xvfb-run -a python3 bindings/Native/tests/native_gl_recovery_contract.py /path/to/library --require-gl
```

기존 authored fixture를 편집해 서로 다른 두 page snapshot을 만들고 CPU/GL 품질을
비교한다. 실제 context를 반복 생성·destroy하며 정상 retire/recreate, 거절의 FBO 불변성,
failed 상태와 마지막 시도 세대, 4 orphan ceiling과 CPU 독립 사용도 검사한다. host context
destruction은 합성 lifecycle 전환이며 **실제 hardware device loss를 주입/관측한 것으로
보고하지 않는다**. `hardwareDeviceLossObserved`는 이 probe에서 `false`다.
