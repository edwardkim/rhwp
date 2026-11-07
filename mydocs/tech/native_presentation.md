---
kind: reference
status: active
canonical: mydocs/tech/native_presentation.md
last_verified: 2026-11-07
---

# Native snapshot과 CPU presentation lease

이 계층은 worker가 소유한 문서에서 불변 render input을 export하고, 별도 render
thread에서 CPU pixels를 만든 뒤 소비자가 명시적으로 acquire/release하게 한다.
session·renderer·Skia context를 다른 스레드로 이동하지 않는다. GPU texture·context
공유나 Flutter 앱의 실제 표시를 구현한 API는 아니다. capability의 `gpuFrames`는
항상 `false`다.

## 소유 스레드와 ABI

기존 `rhwp_session_*`, worker, buffer handle은 여전히 생성한 스레드에서 사용한다.
다음 두 진입점과 **여기서 반환하는 snapshot/frame/lease handle만** 스레드 간에
공유할 수 있다. handle은 pointer가 아닌 프로세스 내 단조 증가 opaque `uint64_t`다.
해제한 ID를 재사용하지 않으며, 종류가 다른 ID·0·중복 반환은 오류다.

```c
char *rhwp_presentation_request(const uint8_t *request, size_t request_len);
const uint8_t *rhwp_presentation_lease_data(uint64_t lease, size_t *byte_length);
```

request는 호출 동안 유효한 UTF-8 JSON bytes이며 1 MiB 이하여야 한다. JSON 응답은
호출한 스레드와 관계없이 `rhwp_string_free`로 한 번 해제한다. 새 객체 ID는 기존
thread-local buffer ID와 호환되지 않는다. `rhwp_presentation_lease_data`의 반환값은
문자열이 아니므로 `rhwp_string_free`에 넘기지 않는다.

`rhwp_session_capabilities().presentation`은 `abiVersion: 1`,
`workerRenderSnapshots: true`, `crossThreadHandles: true`와 실제 빌드의
`cpuFrames`를 보고한다. `native-skia`가 없으면 snapshot 정보/수명 API는 유효하지만
`renderCpu`는 `featureDisabled`다. 기존 RGBA buffer API와 capability는 그대로다.

## worker에서 snapshot 받기

worker owner thread에서 새 문서나 편집 결과의 `documentId`, `revision`,
`fontGeneration`을 받는다. 표시할 view/device의 첫 generation은 1 이상이고
`deviceScale`은 유한한 `(0, 16]` 값이다. 먼저 기존 host request로 source를 설정한다.

```json
{"op":"presentationConfigure","worker":1,"stamp":{"documentId":2,"revision":0,"fontGeneration":0,"viewGeneration":1,"deviceGeneration":1,"deviceScale":1.0}}
```

source는 worker마다 하나이며 worker owner thread만 설정한다. 변경된 stamp를
설정하려면 view/device generation 중 적어도 하나를 증가시켜야 하고 어느 generation도
감소할 수 없다. 문서 교체·편집·글꼴 교체·DPR 변경 후에는 실제 현재 문서의 stamp로
다시 설정한다. 잘못된 설정은 이전 stamp를 바꾸지 않는다.

그다음 `workerSubmit`의 command로 export를 제출하고 기존 `workerPoll`로 결과를 받는다.

```json
{"op":"exportRenderSnapshot","target":{"documentId":2,"revision":0},"fontGeneration":0,"page":0,"profile":"screen"}
```

`profile`은 기존 `RenderProfile::parse`가 받는 profile이고 응답은 정규 이름을 쓴다.
선택적인 `limits`는 [Rust snapshot의 제한](native_render_snapshot.md)을 낮춘다.
export는 일반 render coalescing 대상이 아니다. worker는 payload ceiling을 자신의
`maxPendingBytes`에 맞추고, query 완료 후 실제 논리 입력 추정값을 미수신 completion
예산에 계상한다. 기본 128 MiB ceiling을 미리 예약해 작은 snapshot까지 거부하지 않는다.
이미 대기 중인 다른 출력 때문에 합계가 넘으면 `limitExceeded`로 끝난다. 입력 tree
생성 중의 transient memory를 이 completion 예산이 제한한다는 뜻은 아니다.

성공한 callback 결과는 `kind: "renderSnapshot"`, opaque `snapshot`, `documentId`,
`info`를 포함한다. source 설정 누락·이미 달라진 문서·registry quota는 handle을
전달하지 않고 오류 또는 stale skip으로 끝난다. 전송 후에는 수신자가 snapshot을
해제할 책임이 있다. Python `NativeWorker`는 호출자에게 전달하지 않은 deferred
completion의 snapshot만 close 때 대신 해제한다.

`snapshotInfo`는 document/revision/font/page/profile/fingerprint/usage와 `current`를
반환한다. 원래 문서가 닫힌 뒤에도 정보와 불변 입력은 snapshot handle을 해제할 때까지
살아 있다. C ABI의 **새 표시 프레임 게시**는 현재 source만 허용한다. 문서 종료 뒤의
독립 replay가 필요하면 [P88 Rust snapshot API](native_render_snapshot.md)를 사용한다.
명시적 font bytes는 snapshot에 고정되어 있고 원본 path를 다시 읽지 않는다. 시스템
fallback까지 고정한 portable font 공급이라는 의미는 아니다.

`presentation.fullStampChecks: true`에서는 `snapshotCheck`에 `snapshot`과 전체 `stamp`를
보내 `{ok:true,current:bool}`로 확인한다. 이는 doc/revision/font input identity에 더해
현재 source의 view/device generation과 정확한 DPR까지 비교하는 thread-safe metadata
query다. `snapshotInfo.current`의 input current와 구분하며 Core/replay/legacy TLS를
호출하지 않는다. 유효 handle의 stale/closed는 false, 잘못된 handle/stamp는 오류다.

## render, producer handle, consumer lease

다른 render thread에서 아래 JSON을 `rhwp_presentation_request`로 보낸다. 이 호출은
동기 CPU replay이고 renderer를 호출 스레드에서 생성·사용·파괴한다. UI나 raster
callback에서 매번 호출하기보다 호스트가 소유한 render 작업 스레드에서 수행한다.

```json
{"op":"renderCpu","snapshot":3,"stamp":{"documentId":2,"revision":0,"fontGeneration":0,"viewGeneration":1,"deviceGeneration":1,"deviceScale":1.0},"region":{"x":0,"y":0,"width":400,"height":300},"scale":2.0}
```

region은 논리 페이지 좌표다. `scale`은 논리 좌표를 출력 pixels로 바꾸는 실제 배율이고
`deviceScale`은 호스트의 DPR metadata다. 배율에 DPR을 자동으로 두 번 곱하지 않는다.
호스트가 zoom과 DPR을 반영한 `scale`을 계산한다. 기존 enclosing pixel-window 규칙으로
출력 origin/크기를 구하며 native region replay의 geometry/working-memory 제한을 따른다.

성공은 producer `frame`과 `info`를 반환한다. `info.stamp`는 전체 generation이고
`info.descriptor.backend`는 `"cpu"`다. descriptor에는 다음을 명시한다.

| 필드 | CPU 계약 |
| --- | --- |
| `width`, `height`, `stride`, `byteLength` | 실제 출력 pixels와 행 간격/바이트 수 |
| `pixelFormat`, `colorSpace`, `alpha` | `rgba8Premultiplied`, `srgb`, `premultiplied` |
| `origin`, `originX`, `originY` | `topLeft`, scaled page의 정수 출력 origin |
| `rasterScale`, `deviceScale` | 실제 raster 배율과 별도 DPR metadata |

`frameAcquire`에 `frame`과 **같은 전체 `stamp`**를 보내면 별도 opaque `lease`와
frame info를 받는다. acquire에는 render나 readback이 숨어 있지 않다. 소비자는 lease의
metadata를 확인하고 `rhwp_presentation_lease_data(lease, &len)`으로 불변 CPU view를
받는다. 잘못되거나 이미 해제한 lease는 NULL과 길이 0을 반환한다. `byte_length`가
NULL이면 NULL을 반환한다.

producer가 `frameRelease`해도 살아 있는 모든 lease가 같은 boxed CPU bytes를 pin한다.
source close·새 revision·새 view/device generation은 새 render 게시와 새 acquire를
거부하지만 이미 받은 lease의 bytes를 회수하지 않는다. `frameInfo`/`leaseInfo`의
`current: false`로 표시 가능 상태를 구분한다. 마지막 lease와 producer handle이 모두
해제되어야 실제 frame bytes와 해당 quota credit이 반환된다.

소비가 끝나면 `leaseRelease`, producer가 더 이상 acquire를 제공하지 않으면
`frameRelease`, render 입력이 필요 없으면 `snapshotRelease`를 보낸다. 각각의 해제
대상은 `lease`, `frame`, `snapshot` 필드다. 정보 조회는 `leaseInfo`, `frameInfo`,
`snapshotInfo`다. 모든 요청은 `op` tag를 포함하고 알 수 없는 필드는 거부한다.

raw CPU pointer를 읽는 동안 **같은 lease를 다른 스레드에서 해제하면 안 된다**.
별도 소비자는 별도 lease를 acquire해야 한다. 반환 bytes를 수정하거나 lease 해제 후
접근하면 안 된다. 이 규칙은 C pointer 사용자의 책임이며 JSON request 직렬화만으로
pointer 사용/해제 경쟁을 막을 수 없다. library는 모든 call·worker·handle·lease가
끝날 때까지 loaded 상태로 유지한다. worker는 close 후 `isFinished`를 확인하고
release하는 기존 종료 절차를 따른다.

## 여러 page의 CPU viewport

`native-skia`에서는 cross-thread `renderCpuBatch`로 같은 source의 snapshot들을 하나의
물리 viewport에 합성할 수 있다. GL/Linux feature에 의존하지 않는다. caller의 render
owner에서만 renderer를 구성·사용하며 UI/paint callback에서 Core/조판을 조회하지 않는다.

```json
{"op":"renderCpuBatch","stamp":{"documentId":2,"revision":0,"fontGeneration":0,"viewGeneration":1,"deviceGeneration":1,"deviceScale":1.5},"viewport":{"width":1200,"height":900},"placements":[{"snapshot":3,"scale":1.5,"originX":20,"originY":10,"backgroundColor":16777215}],"clearColor":2148606515}
```

공용 `NativePagePlacement`는 GL의 기존 `NativeGlPlacement`와 같은 JSON/의미이며
scale/origin은 physical viewport pixels다. page background는 opaque COLORREF 또는
`null`, clear는 ARGB다. 최대 32개 page, dimension 4,096, 16,777,216 pixels를 checked
계산한 뒤 기존 frame/replay quota를 먼저 확보한다. snapshot/font bytes는 고정된 입력이다.
borrowed RGBA8 CPU Surface가 곧 output buffer에 그려 추가 전체-frame copy를 만들지 않는다.
동시에 한 renderer만 보유하고 그 세 cache 합계 추정량도 8 MiB 이하로 나눈다.

모든 strict replay와 현재 source stamp 재검사가 끝나야 producer frame을 등록한다.
오류는 producer handle 없이 종료하고 임시 bytes/credit을 반환한다. 다른 기존 lease는
영향받지 않는다. 성공 frame은 `pageCount`, `coordinateSpace: "viewportPixels"`,
top-left origin 0/0과 physical target의 stride를 명시한다. 이 descriptor의 `rasterScale`
1은 viewport pixel 단위이며 각 page의 입력 scale을 대체하지 않는다. 기존 single-page
`renderCpu`는 `coordinateSpace: "pageRaster"`와 실제 page scale/origin을 유지한다.
반환 producer/acquire/consumer lease와 stale/close 이후 메모리 계약은 같다.

## 한도와 수명 경계

기본 process registry 한도는 다음과 같다. Rust API에서는 공개 상한 안에서 낮출 수
있고, C ABI는 capability가 보고한 기본값을 사용한다.

| 항목 | 기본값 |
| --- | --- |
| active source | 16개 |
| retained snapshot / 논리 입력 추정값 | 32개 / 128 MiB |
| retained CPU frame / 실제 bytes | 32개 / 128 MiB |
| consumer lease | 64개 |
| 동시 CPU replay | 4개 |

snapshot/frame 한도는 map의 producer handle 개수만 세지 않는다. 진행 중 replay가
snapshot 입력을 pin하면 snapshot credit을 유지하며, render 전에 실제 출력 bytes와
frame credit을 예약한다. producer 반환 뒤에도 lease가 남으면 frame credit을 유지한다.
잘못된 입력·quota 거절·중복 release는 유효 객체나 예산을 바꾸지 않는다. `stats`는
handle 수와 retained 객체/bytes 및 진행 중 replay를 따로 보고한다.

snapshot bytes는 P88의 보수적인 **논리 payload 추정값**이며 RSS/VRAM이 아니다.
frame bytes는 보유한 boxed RGBA payload의 실제 길이다. allocator/Arc/map metadata,
Skia font/image decode·cache·working raster·driver 메모리와 renderer 생성 중의 임시
할당은 별도다. CPU region replay 자체의 working-raster 한도와 동시 replay 수 제한을
함께 적용하지만 이 두 제한도 프로세스 전체 RSS 상한을 보장하지 않는다.

source observer는 `Weak` runtime 공유 상태로 현재 document stamp만 확인한다. registry나
lease가 원래 runtime·queue·문서를 계속 보유하지 않는다. worker close/release, owner
thread의 TLS 종료, panic 정리는 active source를 닫는다. 기존 snapshot/lease의 owned
입력/bytes는 독립적으로 남는다. lock 순서는 registry에서 source/budget/shared 관찰로
한 방향이며 runtime worker는 registry lock을 잡지 않는다. render 중 registry lock이나
runtime stamp lock을 잡고 있지 않는다. 동시 source 변경은 관찰 시점에 판정하고 다음
acquire가 다시 전체 stamp를 확인한다.

후속 GPU backend는 typed descriptor에 device/context 세대, 외부 image 소유권,
backend fence와 explicit release 완료 조건을 추가해야 한다. CPU lease 반환을 GPU
사용 완료로 간주하거나 `SkCanvas*`, `Surface*`, `GrContext*`를 JSON에 싣지 않는다.
GPU frame pool 재사용은 실제 backend fence 계약을 구현한 뒤에만 허용한다.

## 검사 범위

`tests/cases/issue_536_native_presentation.rs`는 기본 feature에서 작은 worker quota,
실제 미수신 snapshot charge, count/byte limits, stale source와 오류 불변성을 검사한다.
`native-skia`에서는 다른 스레드의 CPU replay/lease read, producer 해제 뒤 lifetime,
마지막 소비자까지의 actual-byte charge, view/device/close 거절을 검사한다.
`bindings/Native/tests/native_presentation_contract.py <shared-library>`는 실제 C ABI를
통해 같은 경계와 legacy owner-thread 계약, deferred completion/TLS 정리를 검사한다.
합성 CPU pixels와 메모리 수명 계약이며 실물 페이지 좌표·한컴 일치율·실제 GPU 표시나
Flutter engine integration의 완료를 판정하지 않는다.
