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

이벤트 미지원·누락·오류에서는 기존 100ms polling이 계속 동작한다. 가상 시계 회귀는 accepted
직후 조회에서 아직 준비되지 않은 producer가 이벤트로 다음 tick 전에 표시 가능한 상태가 되는지,
잘못된 stamp·늦은 알림이 cover/ownership 검증을 우회하지 않는지 확인한다. 이는 실제 화면 지연
측정과 구분한다. 실행 경로는 `flutter test test/native_producer_ready_test.dart`와 기존
`native_texture_performance_test.dart`이며, native queue 계약은 Linux plugin 검사에 포함된다.
