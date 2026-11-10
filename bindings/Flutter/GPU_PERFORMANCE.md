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

요청 경계는 controller invalidate이며 OS 입력 수신 시각이 아니다.
`captureStarted`/`captureCompleted`, `submitStarted`/`accepted`, `producerReady`는
같은 controller의 단조 증가 clock과 generation을 쓴다. producerReady는 현재
batch 전체 receipt·full stamp·필요한 uncover 확인 뒤이며 Flutter compositor의
화면 표시, 물리 GPU 가속 또는 consumer GPU 완료 fence를 뜻하지 않는다.

현재 batch 수락이 끝나 `_working`이 false가 된 뒤 한 번만 status를 확인해
불필요한 다음100ms poll 대기를 줄인다. 아직 준비되지 않았으면 기존 periodic
cadence로 돌아간다. capture·submission 직렬화, pause/closing/stale, RPC3/8,
snapshot cap4, deadline·ambiguous ownership과 native budget은 그대로다.

대역과 가상 시간의 controlled before/after는 request→producer-ready
100000→5000µs, accepted→ready95000→0µs이며 status호출은 두 경우 모두2회였다.
이는 물리 화면의 지연·FPS 측정이 아니다. early poll을 제거하거나 ring eviction을
제거한 음성 대조는 각각 의도한 assertion에서 실패했다. Native/실물 GPU의
성능 수치는 별도 실행 근거로만 보고한다.
