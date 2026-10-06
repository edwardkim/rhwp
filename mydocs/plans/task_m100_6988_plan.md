# Issue #6988 수행 계획

- Issue: #6988 — Chrome 최초 다운로드 상태 저장과 완료 이벤트 경합
- 기준: upstream/devel `a3de5826c3b404bba8d7f3383d55c947f5a35aef`
- 브랜치: `codex/issue-6988-download-event-race`
- 담당자: postmelee (2026-09-20 이슈 assignee 지정)

## 원인과 구현 범위

Chrome adapter의 후보 처리 잠금은 최초 추적 상태 저장을 포함하지 않는다. 저장 전에 도착한
filename/complete 이벤트는 미추적 상태로 종료하고, 뒤늦은 onCreated는 불완전 metadata를 보류한다.
Chrome adapter에서 download ID별 이벤트를 도착 순서대로 처리하여 최초 조회·저장부터 후보 처리와
terminal 기록까지 직렬화한다. 다른 ID는 독립적으로 진행하고 실패한 이벤트가 큐를 막지 않게 한다.
생성 이벤트의 신선도는 대기 후 시각이 아닌 수신 시각으로 판정한다.

공통 분류 정책, Firefox/Safari, 파일명 결정 단계, 다운로드 취소 정책, 뷰어/WASM은 변경하지 않는다.
미추적·과거 다운로드, XLSX, 자체 Blob, autoOpen=false의 기존 제외 조건을 유지한다.

## 검증

1. 최초 storage write를 수동 barrier로 보류한 Node mock에서 변경 전 누락을 확인한다.
2. filename/complete 및 중복 created, 다른 ID 병행, storage 오류 후 후속 이벤트, 제외 조건을 검증한다.
3. 실제 확장 패키지와 격리 Chrome에서 실제 다운로드·이벤트·storage를 사용한다. 최초 저장만
   테스트에서 지연하며 제품에 테스트용 API를 넣지 않는다. 대조군과 지연군 모두 ID당 viewer 1개,
   원본 파일 바이트 보존과 중복 탭 없음이 기대값이다.
4. 기존 Chrome/shared adapter 단위 테스트와 실제 다운로드 E2E를 실행한다.
5. 실행 환경·명령·결과·미검증 범위를 최종 보고서에 남기고 로컬 커밋한다.

작업 지시로 구현과 검증을 진행한다. 원격 push·PR 생성·이슈 close는 별도 승인 단계다.
