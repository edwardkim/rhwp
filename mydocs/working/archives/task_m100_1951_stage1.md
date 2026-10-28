# Task M100 #1951 Stage 1 작업 기록

- 이슈: #1951
- 브랜치: `codex/task_m100_1951`
- 재현 원본: `samples/복학원서.hwp`
- 작성일: 2026-07-10

## 시작 기준

브라우저에서 첫 표의 `Name of College` 값 셀에 장문을 입력해 셀 하단을 넘어가는 캐럿을
재현했다. JavaScript 예외는 없었고, Canvas 텍스트 clip과 별도 DOM 편집 오버레이의 좌표 기준이
서로 다른 것이 확인됐다.

## Stage 1 결과

`tests/issue_1951_table_cell_cursor_clip.rs`에서 첫 표의 `Name of College` 값 셀에 `가` 160자를
삽입했다.

- 일반 즉시 삽입은 전체 페이지네이션으로 행 높이가 재계산되어 셀 bbox 안에 남았다.
- Studio가 사용하는 지연 삽입은 기존 셀 bbox `y=145.3, h=60.1`을 유지한 상태에서 raw caret이
  `y=348.1, h=14.7`까지 내려가 기존 행 밖으로 나갔다.
- 따라서 원인은 저장 표 높이나 Canvas clip 자체가 아니라, page-local 갱신 중의 stale table layout과
  그 좌표를 그대로 쓰는 편집 오버레이다.
- one-depth `cellPath` IME 삽입은 일반 셀 삽입과 달리 셀 폭 리플로우와 vpos 재계산을 거치지
  않는 별도 경로도 확인됐다.

Stage 2에서는 cursor JSON에 셀 bbox/overflow 상태를 제공하고, overflow 순간에만 지연
페이지네이션을 즉시 flush하는 방식으로 진행한다.
