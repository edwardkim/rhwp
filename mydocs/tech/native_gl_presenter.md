---
kind: reference
status: active
canonical: mydocs/tech/native_gl_presenter.md
last_verified: 2026-11-07
---

# 호스트 GL FBO에 그리는 Native owner

`native-skia-gl`은 Linux/OpenGL Ganesh를 사용하는 명시적 opt-in feature다. 기본 빌드와
`native-skia`만 켠 빌드는 계속 CPU renderer이며, Native FFI의 같은 feature가 core로
전달된다. 호스트가 context 생성·current 설정·FBO·swap/present를 소유한다. Native는
해당 context의 owner thread에서 불변 page snapshot을 GPU target에 재생한다.

capability `gpuGl.available`과 `gpuGl.hostFboRendering`은 이 backend가 포함된 Linux
빌드에서만 `true`다. `gpuGl.externalTextureExport`는 `false`이고,
[P89 CPU presentation](native_presentation.md)의 `presentation.gpuFrames`도 계속
`false`다. 이 API는 host FBO 직접 draw이며, 다른 context/Flutter compositor에
GPU texture handle이나 GPU consumer lease를 넘기는 API가 아니다.

## owner와 callback 수명

Rust의 `NativeGlPresenter`는 `DirectContext`, scratch GPU surface pool, frozen-font
renderer cache를 함께 소유한다. `Rc` marker로 `!Send + !Sync`를 유지하고 thread ID도
확인한다. session·renderer·context에 `unsafe impl Send`를 추가하지 않는다. C presenter
handle도 생성 OS thread의 TLS map에만 존재하며 다른 thread에서는 거절한다.

호스트가 matching context를 current로 만든 뒤 아래 typed C callback을 직접 전달한다.
function pointer나 `SkCanvas*`, `Surface*`, `GrContext*`는 JSON에 넣지 않는다.

```c
typedef const void *(*RhwpGlGetProcAddress)(void *user_data, const char *name);
typedef int32_t (*RhwpGlIsCurrent)(void *user_data, uint64_t context_id);

char *rhwp_gl_presenter_create(uint64_t context_id, uint64_t device_generation,
    RhwpGlGetProcAddress loader, RhwpGlIsCurrent is_current, void *user_data,
    const uint8_t *config, size_t config_len);
char *rhwp_gl_presenter_request(uint64_t presenter,
    const uint8_t *request, size_t request_len);
```

loader는 생성 중에만 호출한다. 그 결과 GL function pointer는 owner의 전체 수명 동안
유효해야 하며 loader에는 `gl`로 시작하는 심볼만 전달한다. Skia의 optional EGL probe는
전달하지 않는다. GL resolver가 외부 API 이름에 반환하는 dispatch stub을 EGL 함수로
호출하는 일을 막는다. function pointer는
해당 context에 유효해야 한다. `is_current`는 생성·render·cache clear·정상 release 전에 호출하며
정확히 1일 때만 통과한다. callback과 `user_data`는 release/abandon 또는 owner thread
종료까지 살아 있어야 하고 callback은 ABI에 재진입하거나 unwind하면 안 된다.
`bindings/Native/examples/native_gl.py`는 Python callback과 그 closure/context, CDLL
참조를 명시적 release/abandon까지 보유한다.

context ID와 device generation은 0이 아니어야 한다. ID는 호스트가 실제 context
수명에 부여하는 identity이며 숫자로 변환한 native pointer를 요구하지 않는다. context를
교체하면 세대를 증가시키고 새 presenter를 만든다. 기존 owner를 다른 context/device에
다시 묶지 않는다. 잘못된 owner·identity·generation·actual current는 GL 재생 전에 거부한다.

정상 종료는 matching context를 current로 만든 상태에서 `release`에 `contextId`와
`deviceGeneration`을 전달한다. 이는 Skia 자원을 해제하고 handle을 제거한다. context
loss나 current를 확보할 수 없는 종료는 `abandon`을 쓴다. abandon과 Rust Drop/TLS 종료는
먼저 Skia context를 abandon하여 이후 native object destructor가 GL을 호출하지 않게 한다.
이 경로는 host callback도 호출하지 않는다. library는 모든 native worker/call/handle의
종료까지 loaded 상태로 유지한다.

## 페이지 입력과 target

worker에서 [P89 snapshot export/source 설정](native_presentation.md)을 끝내고 snapshot
ID를 받는다. snapshot은 document/revision/font 입력이며 view가 달라져도 재사용할 수 있다.
표시할 frame의 전체 source stamp는 별도로 현재성을 검사한다. `render` 예시는 다음과 같다.

```json
{"op":"render","stamp":{"documentId":2,"revision":0,"fontGeneration":0,"viewGeneration":1,"deviceGeneration":1,"deviceScale":1.5},"target":{"contextId":1,"deviceGeneration":1,"fbo":7,"width":1200,"height":900,"format":"rgba8","origin":"bottomLeft","sampleCount":0,"stencilBits":8},"placements":[{"snapshot":3,"scale":1.5,"originX":20,"originY":10,"backgroundColor":16777215}],"clearColor":4293848814}
```

`scale`은 페이지 논리 좌표에서 target pixels로 가는 실제 배율이다. `originX/Y`도 target
pixels이며 DPR은 자동으로 곱하지 않는다. `stamp.deviceScale`은 별도 DPR metadata다.
`backgroundColor`는 기존 page replay의 COLORREF, `clearColor`는 Skia의 32-bit ARGB다.
모든 placement는 같은 presentation source와 document/revision/font stamp에 속해야 한다.
호스트가 visible page snapshot 여러 개를 한 target에 배치할 수 있다.

target은 현재 context의 살아 있는 FBO를 정확히 기술해야 한다. sampleCount는 0,
stencilBits는 0 또는 8이며, format은 `rgba8`(GL_RGBA8) 또는
`srgb8Alpha8`(GL_SRGB8_ALPHA8), origin은 `topLeft` 또는 `bottomLeft`다. CPU pointer나
외부 texture를 받지 않는다. caller가 FBO의 실제 크기·format·origin·수명을 보장한다.
Qt가 외부 GL state를 바꿀 수 있어 render 진입마다 Ganesh `reset(None)`을 호출한다.
render가 끝난 뒤 호스트가 사용하는 GL state는 호스트가 복구한다. Qt는
`beginNativePainting`/`endNativePainting` 경계 안에서 호출할 수 있다.

성공 결과의 `frame`은 backend=`openGlGanesh`, context identity, 전체 stamp, typed target,
pageCount, pixelFormat=`rgba8Premultiplied`, colorSpace=`srgb`, alpha=`premultiplied`,
stride=`null`, submission 순서와 wall-clock replay/flush-submit 시간을 보고한다.
`flushSubmitMicros`에는 scratch flush부터 GPU image/wrap/copy/target submit까지의
호스트 작업이 포함되며 순수 GPU 실행 시간을 뜻하지 않는다.
GPU target에는 이 ABI로 접근할 수 있는 CPU stride/view가 없다.

## scratch 게시와 동기화

1. target·placement·quota와 snapshot의 현재 source stamp를 확인한다. RAII borrow가
   producer snapshot handle이 반환된 뒤에도 입력과 P89 reservation을 pin한다.
2. 제한된 GPU scratch surface에 clear하고 모든 page를 strict replay하거나,
   같은 불변 입력에서 strict replay가 완료된 [GPU tile](native_gl_tiles.md)을 재사용한다. renderer는
   snapshot의 고정된 명시적 font bytes로 만들며 원래 font path를 다시 읽지 않는다.
3. scratch flush/submit 성공 뒤 GPU-backed image를 만들고 source를 다시 확인한다.
4. borrowed host FBO에 `Src` copy로 전체 scratch를 그린 뒤 flush/submit한다.

늦은 image/SVG/font resource 오류는 scratch만 바꾸므로 이전 host pixels를 보존한다.
encoded image는 lazy image 생성만으로 승인하지 않고 Skia codec의 완전한 decode 성공을
검사한다. 형식 지원은 실제 빌드에 포함된 Skia decoder를 따른다. source는 8192×8192의
각 변과 32M pixels, encoded bytes는 256 MiB 이내여야 하며 RGBA 검증 버퍼와 codec의
decode memory 옵션에는 각각 128 MiB 상한을 적용한다. 색조 처리 등으로 RGBA로 확장하기
전에도 source pixels를 검사한다. 이 과정은 개별 이미지 decode이며 target 전체를 CPU로
읽어오는 작업이 아니다. 일반 CPU replay의 best-effort 정책은 별도로 유지한다.
source가 달라지면 C ABI는 `staleFrame`, `retryable: true`를 반환한다. Qt는 최신
worker/configure 결과를 기다릴 수 있다. strict replay 오류는 `documentError`, target/
입력 한도 오류는 `invalidArgument`/`limitExceeded`다. GL/context/device 실패는
[typed failure와 회복 계약](native_gl_recovery.md)으로 구분한다. 확인되지 않은 allocation/
submit 실패를 device loss로 취급하지 않는다. GPU loss/flush 실패는 owner를 abandon한다. GPU/device 실패가 host copy
시작 뒤 발생한 경우 이미 제출한 pixels를 되돌릴 수 있다는 보장은 하지 않는다.

source 검사는 동시 변경의 관찰 시점에 판정한다. 확인 직후 발생하는 편집은 다음
호스트 표시 판단에서 stamp를 다시 비교한다. context 자체를 다른 thread에서 동시에
조작하는 것은 허용하지 않는다.

flush/submit은 같은 GL context/queue의 scratch 재사용과 host FBO draw 순서를 보장한다.
CPU wait, swap 완료, 화면 표시 완료, 다른 context의 외부 sampling 완료와는 다른 경계다.
외부 GPU texture export와 consumer fence는 구현하지 않았으며, CPU lease 반환이나
submit 반환만으로 외부 GPU sampling이 끝났다고 판단하지 않는다.

정상 경로의 `wholeFrameReadbacks`는 0이다. 개별 image decode·SVG CPU fallback·font
구성과 GPU 업로드까지 모두 GPU 연산이라는 뜻은 아니다. 이 수치는 host가 검증을 위해
따로 수행한 `QOpenGLFramebufferObject.toImage()`도 포함하지 않는다.

## 예산과 진단

| 항목 | 기본값 / 공개 상한 |
| --- | --- |
| viewport dimension·pixels·RGBA target 추정량 | 4,096 / 16,777,216 / 64 MiB |
| 한 frame의 page placement | 32개 |
| scratch target pool | 2개 / 128 MiB |
| page GPU tile cache | 512px tile / 64개 / 64 MiB |
| scratch·canonical 작업·tile의 공통 target admission | 67개 / 192 MiB |
| frozen-font renderer cache | 기본 2개 / 최대 8개 |
| renderer별 picture/glyph/image cache 합계 추정량 | 8 MiB 이하, 세 종류에 균등 분배 |
| cached renderer의 명시적 per-face font-copy 추정 합계 | 64 MiB |
| Skia GPU resource cache의 요청 상한 | 64 MiB |
| C ABI TLS presenter | thread별 최대 4개 |

config는 이 상한 안에서 낮출 수 있다. `rendererCacheBytes`와 `resourceCacheBytes`의
0은 해당 cache retention을 끈다. target 크기
계산은 checked multiplication을 사용하고 placement는 finite/f32 배율과 좌표 범위를
검사한다. pool/renderer는 LRU로 재사용하며 새 renderer font를 구성하기 전에 이전
cache를 필요한 만큼 비운다. renderer당 cache 한도를 3개 cache 각각에 중복 적용하지 않는다.

이 값은 **native가 관리하는 logical target/cache/font payload 추정량**이다. context
driver의 실제 texture 배치·Skia 중간 작업·deferred GPU references·metadata·allocator·
OS font manager·개별 resource decode의 전체 VRAM/RSS를 측정하거나 제한하지 않는다.
명시적 font 프로그램은 고정되지만 시스템 fallback 이식성은 인증하지 않는다.

`clearCaches`는 matching current context/generation에서 idle tile·scratch·renderer와
purgeable Skia 자원을 해제한다. source/snapshot은 유지한다. tile key·phase·LRU pin과
canonical replay의 fallback, 실제 cache/target 계상은 [GPU tile 계약](native_gl_tiles.md)을 따른다.

`info`는 device vendor/renderer/version, 알려진 software renderer 문자열의 탐지 결과,
현재 pool/cache 계상과 제출 수를 보고한다. software 이름 탐지는 문자열에 근거한
진단이며 `hardwareAccelerationVerified`는 항상 `false`다. wall-clock 시간은 GPU timestamp나
지속 FPS 측정이 아니다. `info`는 GL/current callback 없이 owner의 Skia bookkeeping을 읽는다.

정상 matching-current `retire`/`release`와 미입증 `abandon`의 자원 반환은 다르다.
현재 native context와 orphan을 합한 process credit은 4개다. 새 context로의 명시적
recreate와 failed/released 상태·credit, 독립 CPU frame 비교는
[회복·품질 계약](native_gl_recovery.md)을 따른다.

## 실행 검증

`tests/cases/issue_536_native_gl.rs`는 기본 feature에서 RAII snapshot borrow의 한도·수명과
view 변경 후 입력 재사용을 검사한다. GL feature에서는 context 없이 검사할 수 있는
target/placement 한도를 추가로 검사한다. 실제 FBO 검사는 다음 경로다.

```sh
QT_QPA_PLATFORM=xcb xvfb-run -a python3 bindings/Native/tests/native_gl_contract.py /path/to/library --require-gl
```

Qt의 offscreen **platform plugin**은 이 GL probe 대상이 아니다. Xvfb/xcb에서
`QOpenGLContext`와 `QOffscreenSurface`, `QOpenGLFramebufferObject`를 실제 생성한다.
기존 작은 authored PNG fixture를 사용하고 메모리 안에서 payload를 깨뜨린 HWPX도
열어 strict 오류 뒤 host pixels 보존을 검사한다. wrong owner/current/generation,
view 재설정, source close, abandon callback 부재와 metadata/계상을 함께 확인한다.
probe의 host readback은 별도로 계상한다. 기본/CPU-only library에서는 Qt를 import하지
않고 새 C entry가 `featureDisabled`인지 검사한다. 이 합성 검사는 실제 hardware 가속,
한컴 시각 일치율·실물 좌표·Qt 앱 표시·Flutter 외부 texture 연결의 인증이 아니다.

참조: [Skia 0.156 GL interface](https://github.com/rust-skia/rust-skia/blob/0.156.0/skia-safe/src/gpu/ganesh/gl/interface.rs),
[Skia context의 abandon/submit 계약](https://github.com/rust-skia/rust-skia/blob/0.156.0/skia-safe/src/gpu/ganesh/direct_context.rs),
[Qt current context와 getProcAddress](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QOpenGLContext.html).
