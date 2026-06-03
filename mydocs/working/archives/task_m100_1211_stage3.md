# Task M100-1211 Stage 3 완료 보고 — 입력 라우팅 연결

## 변경 내용

- `InputHandler.executeOperation()`의 command 경로에서 실행 전/후 `DocumentPosition`을 비교한다.
- `afterEdit()`는 full refresh 기본 경로로 유지했다.
- `afterPageLocalEdit()`를 추가해 dirty 상태 갱신용 `document-mutated`는 유지하면서, 렌더 갱신은 `document-page-invalidated`로 보낸다.
- page-local 판정은 `input-edit-invalidation.ts`의 순수 helper로 분리하고 단위 테스트를 추가했다.

## narrow invalidation 대상

- `insertText`
- `deleteText`
- 실행 전/후가 같은 section, parent paragraph, control, cell, cell paragraph, cellPath에 남아 있는 표 셀 내부 편집

## full refresh 유지 대상

- 본문 문단 텍스트 입력
- header/footer, footnote
- 붙여넣기, snapshot 기반 작업
- 문단 분할/병합, 선택 삭제
- 표/객체/페이지/서식 대화상자 등 구조 변경 가능 작업

## PR 명시 필요 사항

PR 본문에는 이번 선택이 #865 대체가 아니라 #865 이후에도 남는 full `document-changed` 비용을 줄이는 작업임을 적는다. 또한 A+C 선택 이유와 후보 B/D를 이번 PR에서 제외한 이유를 명시한다.
