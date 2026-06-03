# Stage 4 보고서 — Task #1073: 부분 렌더 매핑 (컷 → NestedTableSplit)

- 브랜치: `local/task1073`
- 수정: `src/renderer/layout/table_partial.rs`

## 구현
`layout_partial_table` 의 중첩 표 렌더 분기를 페이지네이션 컷 구동으로 전환:
- 셀 컷 유닛 `(start_unit, end_unit)` 을 셀 스코프로 hoist(`cut_units`).
- 셀이 per-중첩행 분해 대상(단일 문단 + 가시 텍스트 없음 + 단일 중첩 표)이면
  `nested_cut_range = Some((su, eu))` — cut 유닛 인덱스 = 중첩행 범위.
- 중첩 표 렌더: `nested_cut_range` 가 있으면 `NestedTableSplit{start_row=su, end_row=eu,
  offset_within_start=0, visible_height=Σrow_h[su..eu]}` 직접 구성(연속 페이지 start_row 반영).
  없으면 기존 `available_h` 휴리스틱 폴백(비분할/비대상 불변).

## 검증 (kps-ai)
- **연속 페이지 정정**: page67(연속)이 더 이상 중첩행 0(제목)부터 재렌더하지 않음.
  - page66: 제목 ~ 3.민간소프트웨어 섹션(있음/없음/※없음) — PDF p62 정합.
  - page67: 시장침해 가능성 ~ 4.사업의 필요성 ~ 5.종합의견 — PDF p63 정합.
  - 전체 중복 렌더 제거, 본문 누락 없음, overflow 없음.
- **잔여(rowspan 라벨 누수)**: 중첩 표 내부 rowspan 섹션 라벨("3.민간소프트웨어")이 분할 경계를
  걸칠 때 연속 페이지(page67) 상단에도 그려짐. PDF p63 은 해당 라벨 공란. 외부 표의
  `advance_row_block_cut`(rs>1 라벨 연속 조각 공란) 처리가 **중첩 표 분할엔 미적용**.
  → 본문/구조 정합은 달성, **rowspan 라벨 블랭킹만 잔여**(별도 정밀화).

## 회귀 (Stage 4)
- 전수 sweep 3055 lines / 382054px (Stage 3 동일 — 렌더 변경은 overflow 미영향).
- 골든 SVG **8/8**, clippy/fmt clean.

## Stage 4b — rowspan 라벨 블랭킹 (승인: 가)
`layout_table_cells`: 중첩 표 분할 연속 페이지(row_filter `sr>0`)에서 분할 시작 행보다 먼저
시작한 rowspan 셀(`r < sr`)의 `composed_paras` 를 clear → 라벨 텍스트 미렌더(영역/배경만).
row_filter 는 중첩 표 분할 전용(외부 표는 layout_partial_table 경로)이라 외부 표 무영향.

검증:
- page67: "3.민간소프트웨어"/"시장침해 가능성" rowspan 라벨 공란화 → "주요기능…" 부터 시작.
- 누락 검사: "시장침해" p66(라벨 1)·p67(섹션5 내용 2), footer(기관명…직인) p67 완결 → **콘텐츠
  완전, 누락/중복 없음**.
- 골든 8/8, lib 1324, 전수 sweep 3055/382054(무회귀), clippy/fmt clean.

## 잔여 (minor) — break 위치 정밀도
rhwp 컷이 한컴(PDF p62/p63) 대비 ~2 중첩행 늦음(시장침해 블록을 p66 에서 시작) → 블록이 페이지
경계를 걸쳐 라벨은 p66, 잔여는 p67(공란). 콘텐츠/구조 정합·overflow 0 달성, **break row 정확
일치만 잔여**(available-height/overhead 측정 정밀도). known limitation 로 기록, 별도 정밀화 대상.
