# Task #1716 구현계획서 — 상단 연속 제목행 블록만 반복

- 이슈: edwardkim/rhwp#1716 / 브랜치: `local/task1716`
- 수행계획서: `task_m100_1716.md`

## 정의 (검증 완료)

**반복 제목행 = 표 상단(행 0)부터 연속인 제목행 블록 `rows 0..H`.**
- 행 r 이 "제목행" ⟺ header 셀(`is_header`)이 rowspan 포함으로 r 을 덮음:
  `∃ cell. cell.is_header ∧ cell.row ≤ r < cell.row + cell.row_span`.
- `H` = 행 0부터 제목행이 연속되는 최대 구간(첫 비-제목행에서 종료). 흩어진 하위 header 행 제외.

검증 사례:
- 건설공사 품질시험기준 pi=12: header 행 169개(0,15,16,…,182)지만 행1이 비-header → **H=1**.
- 다중 머리행(pi=111, rs≥2 상단 header): 행0·1 연속 header → **H=2** (#1022 보존).

## 단계

### Stage 1 — 공유 헬퍼 (`model/table.rs`)
`impl Table { pub fn leading_header_rows(&self, row_count: usize) -> Vec<usize> }`:
- 각 행 제목여부 배열 계산(rowspan 덮개 반영) → 행0부터 연속 구간 `0..H` 반환.
- `repeat_header`/`has_header_cells` 게이트는 호출부 유지(헬퍼는 순수 계산).
- 단위 테스트: (a) 상단 1행 + 흩어진 하위 header → [0], (b) 상단 rs=2 다중 → [0,1],
  (c) header 없음 → [], (d) 전 행 header → 전체(정상 소형 표 회귀 방지).

### Stage 2 — 페이지네이터 적용 (`renderer/typeset.rs` ~11320)
`header_overhead` 의 `filter(is_header && row<cursor_row)` 를 헬퍼 결과 `leading_header_rows`
∩ `{r < cursor_row}` 로 교체. 높이 합/`cs` 계산 로직은 유지. 동작:
- H=1 이고 cursor>0 → overhead = row0 높이 + cs (기존 정상 경로와 동일).
- 대표 파일 `dump-pages` 총쪽수 폭주 해소 확인(중간 점검).

### Stage 3 — 렌더러 적용 (`renderer/layout/table_partial.rs` ~166, ~308)
동일하게 `header_rows` 수집을 `leading_header_rows ∩ {r < start_row}` 로 교체(2곳).
페이지네이터와 **동일 헬퍼** 사용 → desync(오버플로) 차단.
- 대표 파일 페이지수/시각 정합: pi=12 페이지당 다행 배치 회복, ≈52쪽 수렴.

### Stage 4 — 회귀·통합 검증 + 샘플/보고
- `cargo test` 전체 그린.
- #1022 다중 머리행 표 페이지수 불변(회귀 테스트/픽스처).
- `verify_pi_page_vs_hangul` 로 대표 파일 + 인접 PAGE_DELTA 대형 아웃라이어 재검증.
- `samples/` 회귀 게이트(hwpx roundtrip baseline, clipping gate) 무회귀.
- 재현 샘플(대표 hwpx) `samples/task1716/` 추가.
- 최종 보고서 `task_m100_1716_report.md`.

## 커밋 규약
각 단계 소스 + `task_m100_1716_stage{N}.md` 를 `Task #1716: …` 로 함께 커밋.

## 롤백 안전
헬퍼 도입 + 호출 2곳 교체만. 실패 시 호출부를 기존 filter 로 즉시 원복 가능.
