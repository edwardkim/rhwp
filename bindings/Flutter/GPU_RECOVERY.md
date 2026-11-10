# Flutter 참조 호스트의 명시적 GPU 복구

`NativeGpuSession`은 기존 `NativeDocument`와 입력·파일 owner를 유지한 채 GPU
controller만 교체한다. 호스트는 session의 변경 알림을 듣고 같은 페이지 뷰에
`session.controller`를 전달한다. GPU fallback 뒤 `await session.recover()`를
명시적으로 호출한다. 문서 replay 오류의 `failed` 상태는 장치 복구로 재분류하지 않는다.

이전 뷰를 mount한 채 종료를 기다리며, 이전 controller가 `closed`이고
`retirementPending == false`일 때만 동기 factory를 호출한다. 중복 복구 요청은
하나의 Future를 공유한다. factory 실패 뒤에는 확정 종료된 이전 controller를
유지하고 진단을 알린다. 다른 문서 또는 이미 사용한 controller는 거절한다.
복구 중 문서 종료가 시작되면 새 controller를 만들지 않는다.

새 controller는 정상 create/device generation·snapshot·publication 검증을 다시
수행한다. `recover()`의 true는 교체 완료이며 GPU 표시 성공을 뜻하지 않는다.
Native process의 immutable frame·quarantine 예산은 초기화하지 않는다. 이전
close 확인과 consumer GPU 완료 fence도 구분한다.

문서를 닫기 전에는 `await session.close()`의 true를 확인한다. false이면 이전
GPU retirement가 아직 미확정이므로 문서 owner를 닫지 않는다. session과 document의
dispose는 호출 호스트가 각각 소유한다.

검증: Flutter 3.47.9의 controller/transport 대역과 실제 widget 수명에서 명시적 교체,
미확정 retirement 거절, 문서 오류 분리, factory 실패/다른 문서 거절, 동시 close의
8개 검사가 non-skip으로 통과했고 package/example analyze 0을 확인했다.
다중 페이지·문서 수명 suite를 함께 실행한 43개 검사도 통과했다. 실제 OS 장치 상실·
물리 GPU 복구를 실행한 결과는 아니다. mount 중 controller 알림을 받는 session은
ancestor rebuild 알림만 frame 뒤로 합치며 실제 texture의 즉시 cover는 유지한다.

첫 bounded close의 false는 종료의 영구 결론으로 캐시하지 않는다. 원래 늦은 응답이
retirement를 확인하면 다음 close가 현재 상태를 다시 판정해 true를 반환한다. 이때
Native close를 재전송하거나 consumer 완료를 추정하지 않는다. 이 반례는 수정 전
실패·수정 후 통과했으며 Native close 호출이 한 번인 것도 검사한다.

참조 example의 `GPU 다시 연결` 버튼은 같은 document에 session factory로 새
controller를 연결한다. 복구 중 입력과 문서 동작을 잠시 막고, 문서 교체 뒤
확정 종료된 viewport는 CPU 경로를 사용하며 명시적 복구를 기다린다. 여러 페이지의
새 capture가 서로 다른 snapshot ID와 새 view generation을 사용하는지 검사한다.
