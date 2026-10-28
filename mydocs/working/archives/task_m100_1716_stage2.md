# Task #1716 Stage 2 완료보고서 — 페이지네이터 적용

## 작업
`src/renderer/typeset.rs` `header_overhead` 계산 교체(반복 제목행 overhead):
- 종전: `table.cells.filter(is_header && row < cursor_row)` 의 행 높이 전부 합산.
- 변경: `table.leading_header_rows()` ∩ `{r < cursor_row}` 만 합산.
- 빈 경우 overhead=0 (렌더러가 아무것도 반복 안 하는 경우와 정합).
- 미사용이 된 `header_row_height` 지역변수 제거.

## 근거
흩어진 하위 `is_header` 행까지 합산하면 cursor 전진 시 overhead 가 누적 → `avail_for_rows=0`
→ 페이지당 1행 폭주. 상단 연속 제목행 블록만 반복하면 overhead 는 작은 상수.

## 검증 (대표 파일, Stage 3 빌드와 함께)
`RHWP_TABLE_DRIFT` 기준 header_oh 누적 제거 확인. 총 페이지수는 Stage 3 반영 후 측정
(페이지네이터·렌더러 동시 정합 필요).

## 상태
완료. 렌더러(Stage 3)와 동일 헬퍼 사용.
