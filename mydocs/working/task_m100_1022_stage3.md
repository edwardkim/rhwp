# Stage 3 완료보고서 — #1022: cell_units / row_cut_content_height 정합 구현

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 3 — Stage 2 설계 (C → A) 구현

## 1. 베이스라인 측정 정정 (중요)

이전 분석에서 베이스라인 8be5e0c2 의 `LAYOUT_OVERFLOW` 0건으로 보고했으나
`grep` 정규식 오류였다 (`^LAYOUT_OVERFLOW ` 가 `LAYOUT_OVERFLOW: ` 와
매칭되지 않음). **재집계 결과 베이스라인 = 42건, task993 머지 후 = 42건,
페이지 22 18.3px 초과는 베이스라인부터 존재**.

따라서 본 타스크 #1022 의 원래 전제("task993 회귀 50건")는 정정 필요:
- 본 타스크는 task993 회귀 해소가 아니라 측정 시스템 정합으로
  사전 존재 LAYOUT_OVERFLOW 의 감소를 노린다.
- 페이지 22 18.3px 초과는 task993 도입 사안이 아니라 더 깊은 측정 미스매치.

## 2. 구현 — Stage 2 의 (a) C → A

### 2-1. `cell_units` (`table_layout.rs:3405~`)

- 줄별 trailing line_spacing 규칙 → A 와 정합:
  `include_trailing_ls = !is_cell_last_line || para_count > 1`
  (셀 단일 문단·단일 줄 셀에서만 ls 제외)
- `cell.height` 필러 제거 (행 단계 정합으로 이동).
- 비인라인 Picture/Shape(wrap=TopAndBottom) 높이를 셀 끝에 별도 필러로 가산
  (HeightMeasurer::measure_non_inline_controls_height 와 정합).

### 2-2. `row_cut_content_height` (`table_layout.rs:3691~`)

- 반환값을 **행 총 높이(패딩 포함)** 로 변경. 셀별 식:
  - 분할 아닌 행: `max(cell.height, content + pad_cell)`.
  - 분할 행: `content + pad_cell` (cell.height 강제 없음).
- 행 max 산출 — HeightMeasurer 의 단계 1·2 정합.

### 2-3. 페이지네이터 walk (`typeset.rs typeset_block_table`)

- `cut_row_h[r] = row_cut_content_height(r, &[], &[])` 단일 호출.
- 일반 행 배치: `row_total = cut_row_h[r]`, fits 검사·`consumed += cs + row_total`.
- 분할 행: `advance_row_cut` 으로 컷 결정 후, `row_cut_content_height(r, &start, &end)` 으로 행 기여를 `consumed` 에 가산.
- `partial_height = consumed + header_overhead` — 분할 행 별도 가산 제거(walk 가 일원화).

### 2-4. 렌더러 2b (`table_partial.rs layout_partial_table`)

- 분할 행만 컷 측정 오버라이드 (HEAD 의 "all rows" 확장 환원).
- `row_heights[r] = row_cut_content_height(r, su, eu)` — 함수가 행 총 높이 반환하므로 별도 padding 가산 제거.

## 3. 검증

- `cargo build --release` 무경고, `cargo test --release` **1302 passed**, svg_snapshot 8 passed.
- `form-002` 골든 부동소수 말단 차이 갱신.
- 비공개 184페이지 샘플 `LAYOUT_OVERFLOW`:
  - 베이스라인 42 → **38 (4건 감소)**.
  - 페이지 22 18.3px 잔존 (베이스라인부터 존재, 본 정합으로는 해소 안 됨).

## 4. 잔여 — 페이지 22 18.3px

페이지 22 항목 6개의 렌더러 누적 y_offset = 959.4 vs 본문 941.1 → 18.3 초과.
페이지네이터 `used=902.2` 와 렌더러 959.4 사이 57.2 drift.

페이지네이터 vs 렌더러 항목별 측정 차이:
- pi=222·224·225 (FullParagraph): 렌더러 31.2 vs 페이지네이터 17.3 — **+13.9 각**.
- pi=223 (Table inline): 렌더러 319.7 vs 페이지네이터 288.1 — **+31.6**.

`MeasuredParagraph.total_height` vs `layout_composed_paragraph` 렌더링 사이의
trailing line_spacing 포함 여부 차이로 보임 — `cell_units`/`HeightMeasurer`
정합과 **별개 부류** (paragraph 측정 vs 렌더 정합).

## 5. Stage 4 방향

본 task1022 의 명시 목표(`HeightMeasurer ↔ cell_units` 정합)는 달성.
페이지 22 본문 초과는 **paragraph 측정 정합**이라는 별도 부류 — 본 타스크
범위 밖이며 별도 후속 작업이 필요하다.

검증 단계에서:
1. 본 정합 변경이 다른 회귀를 만들지 않았는지 공개 골든 SVG 교차 점검.
2. 페이지 22 잔여를 별도 이슈로 분리할지, 본 타스크 범위 확장할지 판단.
