# GPU 참조 호스트의 지연 기록

`NativeTextureController(trace: NativeGpuTrace())`는 opt-in 진단이다. 기본값은
null이며 recorder·clock·event queue를 만들지 않는다. 참조 example에서
`--dart-define=RHWP_GPU_TRACE=true`를 주면 각 controller에 새 recorder를 만들고
명시적 복구 또는 호스트 종료 경계에서 bounded JSON을 debug 출력한다.
복구 뒤 새 controller에 이전 generation의 recorder를 재사용하지 않는다.

기본 ring은 event128개와 완료 sample128개이며 각각 최대512개로 제한된다.
`trace.toJson()`은 수명 전체 건수와 버린 event/sample 수, 보관된 완료 sample의
nearest-rank p50/p95/p99를 구분한다. 문서 경로·본문·raw 오류나 snapshot ID는
기록하지 않는다. plugin이 제공한 알려진 numeric 자원 counter만 포함하고
없으면 생략한다. 전체 driver memory를 추정하지 않는다.
`wholeFrameReadbacks`/`pluginWholeFrameReadbacks`도 Native plugin의 계수다.
Flutter engine의 X11 compositor가 수행하는 전체 프레임 CPU 복사·재업로드는 포함하지 않으므로
0을 engine 전체의 zero-copy 또는 GPU-only 표시 근거로 사용하지 않는다.

요청 경계는 controller invalidate이며 OS 입력 수신 시각이 아니다.
`captureStarted`/`captureCompleted`, `submitStarted`/`accepted`, `producerReady`는
같은 controller의 단조 증가 clock과 generation을 쓴다. producerReady는 현재
batch 전체 receipt·full stamp·필요한 uncover 확인 뒤이며 Flutter compositor의
화면 표시, 물리 GPU 가속 또는 consumer GPU 완료 fence를 뜻하지 않는다.

`captureStarted`부터 `captureCompleted`까지는 문서 FIFO 대기, presentation configure,
worker snapshot export, owner/worker 응답의 Dart timer polling과 full-stamp 검사까지 포함한다.
따라서 이 구간의 timeout만으로 Native snapshot 생성 자체가 느리다고 판정하지 않는다.
Linux 참조앱 builder는 공개 runner API로 Dart UI를 GTK platform thread에서 분리한다.
또한 현재 Native GL bridge와의 호환성을 위해 Flutter compositor의 Skia 경로를 선택한다.
GTK draw/resize 대기와 Dart polling의 스레드 경계 및 compositor 선택을 명시하는 설정이며,
renderer 성능 개선이나 GPU consumer 완료의 측정 결과를 대신하지 않는다.

현재 batch 수락이 끝나 `_working`이 false가 된 뒤 한 번만 status를 확인해
불필요한 다음100ms poll 대기를 줄인다. 아직 준비되지 않았으면 producer-ready 이벤트와
기존 periodic cadence로 확인한다. capture·submission 직렬화, pause/closing/stale, RPC3/8,
snapshot cap4, deadline·ambiguous ownership과 native budget은 그대로다.

대역과 가상 시간의 controlled before/after는 request→producer-ready
100000→5000µs, accepted→ready95000→0µs이며 status호출은 두 경우 모두2회였다.
이는 물리 화면의 지연·FPS 측정이 아니다. early poll을 제거하거나 ring eviction을
제거한 음성 대조는 각각 의도한 assertion에서 실패했다. Native/실물 GPU의
성능 수치는 별도 실행 근거로만 보고한다.

P121 plugin은 create 응답의 `producerReadyEvents:true`로 공개 EventChannel
`<platform method channel>/events`를 알린다. Linux GL, Windows D3D, macOS CV texture와
native Metal view가 현재 native publication을 게시한 뒤 `producerReady`를 보낸다.
각 이벤트는 texture ID, device generation, native serial과 전체 source/view/device stamp만
포함한다. 이미지·문서 경로·본문을 전송하지 않는다.

Native mailbox는 두 등록과 한 번의 platform-thread drain으로 제한한다. Linux는 GTK main
context, Windows는 plugin 등록 스레드의 message-only HWND, macOS는 main queue에서
Flutter EventSink를 호출한다. 중복·역순 알림은 합치거나 버리며 stream 취소와 plugin 종료는
대기 알림을 폐기한다. 생산자 완료는 Flutter compositor 표시나 consumer GPU fence가 아니다.

Dart는 messenger/backend별 한 구독을 여러 controller가 공유한다. 이벤트의 owner와 전체
stamp가 현재 요청과 일치할 때만 기존 status/snapshot 검사를 앞당긴다. 이벤트 자체로 cover를
해제하지 않으며, 두 타일은 두 멤버의 publication과 accepted snapshot을 모두 확인한다.
진행 중인 status 조회 사이에 도착한 새 알림은 후속 조회 한 번으로 합친다. 닫힌·정지된 owner의
알림은 무시하고, 마지막 controller가 종료되면 구독을 취소한다.

P127은 이벤트 구독의 `listen` 성공 응답까지 확인한 뒤, 현재 publication을 status와 snapshot으로
검증하여 표시 중일 때 유휴 GPU status/snapshot RPC를 생략한다. 100ms local timer는 유지하며
문서 revision·font·raster·다른 viewport의 presentation 세대·DPR의 현재 여부를 검사한다.
문서나 geometry 변경은 기존 invalidation 경로로 capture와 전체 stamp 검증을 다시 시작한다.
캐시된 pixels에 대한 같은 stamp의 원격 재검사를 매 tick 반복하지 않는 변경이며, 이벤트 자체로
표시 권한이나 snapshot 소유권을 부여하지 않는다.

진행 중인 publication은 알림이 누락돼도 기존 100ms fallback polling을 유지하며 2초 publication
기한 안에 준비되거나 실패해야 한다. 이벤트 미지원·listen 실패·stream 종료·오류/잘못된 envelope는
유휴 상태에서도 기존 polling으로 돌아간다. native cover와 registration retirement는 별도
경로와 기한을 유지한다. 유휴 상태의 물리 device 오류는 다음 native 호출에서 판정하며, 이 변경은
주기적인 원격 장치 health check나 consumer GPU 완료 보장이 아니다.

가상 시계 회귀는 accepted 직후 조회와 ready 이벤트, 유휴 10초간 GPU RPC 없음, 실패 시 polling
복귀, 두 타일 publication·문서/DPR 변경·cover·종료 및 deadline을 검사한다. 이는 실제 GPU의
전력·CPU 사용량·화면 지연 측정과 구분한다. 실행 경로는
`flutter test test/native_producer_ready_test.dart test/native_texture_idle_polling_test.dart`와
기존 `native_texture_performance_test.dart`이며, native queue 계약은 Linux plugin 검사에 포함된다.

## P128 내부 GPU 캐시 축소

Native GL·Metal·D3D presenter는 additive JSON `trimCache` 요청을 받습니다. `targetBytes`는 기본 0이며
기존 presenter의 managed target 한도 이내여야 합니다. GL 요청은 기존 `contextId`·`deviceGeneration`도
필수이며 실제 owner thread에서 해당 context가 current일 때만 실행합니다. Metal/D3D도 생성 owner
thread에서만 실행합니다. 진행 중인 제출·snapshot pin·quarantine·완료 불명확 상태는 해제하지 않습니다.
GL 타일과 scratch, Metal/D3D 타일과 완료된 scratch 중 pin이 없는 것만 줄입니다. 설정한 예산은
바뀌지 않으며 Flutter에 공개한 immutable texture·drawable·두 front/back 출력은 이 대상이 아닙니다.

결과의 `before`·`after`·`trimmed`는 managed target의 논리 바이트입니다. `trimmed = before - after`이며,
목표에 도달하지 못했거나 완료가 불명확하면 `busy: true`입니다. `scope: managedGpuTargets`는 Skia의
내부 자원 캐시·driver 전체 VRAM·RSS·공개 출력의 메모리를 포함하지 않습니다. Windows private producer
target pool과 CVMetalTextureCache도 각 bridge의 별도 장부·정책이며 이 Native 계수 밖입니다. 이 호출은 전체 메모리
회수나 consumer GPU fence를 증명하지 않습니다.

플랫폼 채널은 UI에서 GPU 소유자를 직접 부르지 않습니다. `trimCache`에 양수 `requestId`,
현재 `deviceGeneration`, `textureId`, 선택적 `targetBytes`를 보내면 owner당 최대 한 요청을 보관합니다.
같은 ID/target 재요청은 같은 결과를 반환하며 오래된 ID·동시에 다른 ID·불일치 target은 거절합니다.
`trimCacheStatus`는 같은 ID·generation으로 한 개의 마지막 결과를 조회합니다. 큐 대기는
`pending: true, busy: true`, owner가 처리한 뒤에는 `pending: false`입니다. 마지막 `busy: true`는
pin·완료 불명확 또는 명시한 Native 오류로 목표 축소를 확인하지 못했다는 뜻이며 성공으로 표시하지 않습니다.

Linux는 current raster callback에서, Windows와 macOS CVTexture/AppKit native view는 producer loop에서
이 요청을 처리합니다. 화면에 표시되지 않아 Linux callback이 오지 않으면 요청은 pending으로 남습니다.
호출자는 기한을 두고 pending만 poll하며, 기한 초과·owner 교체·close의 늦은 결과를 무시해야 합니다.
close·context 종료는 남은 큐를 취소하며 미확정 GPU 자원을 해제하지 않습니다.

기본 두 타일 경로도 같은 메서드를 `groupId`로 라우팅합니다. 그룹 결과는 순서가 고정된 두 `members`와
바이트 합계, pending/busy의 OR를 제공합니다. `targetBytes`는 **각 presenter의 목표**입니다.
따라서 두 owner가 각각 목표에 도달했어도 합계 `after`는 `targetBytes`보다 클 수 있습니다.
각 member의 ID·generation·requestId를 검증하며, 그룹 trim은 공개 프레임 교체·새 snapshot 인수나
두 GPU owner의 동시 transaction을 의미하지 않습니다.

검증은 실제 공통 Rc pin 제거 정책, bounded 요청 큐, Linux owner-thread/공개 front 보존 및
두 타일 Dart 라우팅을 다룹니다. 실제 OS SDK·GPU cache on/off·장시간 메모리·한컴 최종 검증은 후속입니다.
