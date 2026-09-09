# Task #6964 — 구현과 회귀 검증

- 계획: [task_m100_6964.md](../plans/task_m100_6964.md)
- 이슈: https://github.com/edwardkim/rhwp/issues/6964

## 구현 판단

기존 Firefox handler는 session.get/set과 settings 조회를 await하는 동안 같은 ID의 다음
handler가 실행된다. 두 handler가 모두 handledAt이 없는 상태를 읽을 수 있고, terminal
handler가 이전 상태를 다시 기록할 수도 있다. openViewer 직전 플래그만 추가하면 상태 쓰기
경합은 남으므로 Firefox adapter 전체 이벤트 처리를 ID별 Promise 큐로 직렬화했다.
수신 시각을 별도 보존하고, 완료 후 큐 항목을 제거하며, 실패는 로깅하고 다음 이벤트를 진행한다.

저장 재오픈은 download filename만으로 분류하면서 자신의 Blob도 새 입력으로 인식한 것이다.
Chrome/Firefox 모두 runtime.getURL('')의 정확한 extension origin에 속한 Blob만 제외한다.
HTTP, 외부 웹 Blob, 다른 확장 Blob, 비슷한 extension ID는 제외하지 않는다.
취소/다운로드 내역 제거 API는 호출하지 않는다.

공통 evaluateDownloadCreated/evaluateDownloadChanged, 5초 freshness, TTL, terminal 정리,
미추적 onChanged 제외 정책은 변경하지 않았다. Chrome의 기존 동시 처리 잠금도 유지했다.

## 결정적 테스트

추가 테스트를 기존 코드에서 실행: 49개 중 6개 실패, 43개 통과.
실패는 session/fallback 중첩 이벤트, 생성 저장보다 먼저 온 변경 유실,
storage 오류 이후 처리, 두 브라우저 자체 Blob 재오픈이었다.
수정 후 수신 시각 보존 테스트까지 포함한 shared/Chrome/Firefox 전체는 147/147 통과했다.
서로 다른 ID 진행, late terminal, 과거 다운로드와 재시작, 비-HWP, 설정 보호도 포함한다.

Firefox E2E는 별도 임시 프로필, loopback 실제 HWP, production DOM 편집과 메뉴,
실제 downloads/tabs API 및 저장 파일 WASM 재파싱으로 결과를 확인한다.
Chrome E2E는 기존 4종 다운로드 뒤 실제 extension Blob 저장의 원본 바이트와 추가 탭 0개를 확인한다.
테스트용 OS 저장 대화상자는 자동 다운로드 디렉터리로 대체한다.
