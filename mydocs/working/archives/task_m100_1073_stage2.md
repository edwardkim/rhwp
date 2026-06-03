# Stage 2 보고서 — Task #1073: 중첩 표 분할 설계 + 페이퍼 검증

- 브랜치: `local/task1073` (소스 무변경)

## 컷→렌더 데이터 흐름 (현행)
```
cell_units(cell) → Vec<CellUnit>            // 셀 콘텐츠 분할 단위
  ↓ (per-cell 유닛 인덱스)
advance_row_cut → start_cut/end_cut         // 페이지에 들어가는 유닛 컷
  ↓
row_cut_content_height(cut)  = Σ units[su..eu].height   // 행 높이
cell_line_ranges_from_cut(cut) = per-para (vis_start,vis_end)  // 렌더 가시 줄
  ↓
layout_partial_table(start_cut,end_cut) → line_ranges → 가시 줄 렌더
```
- **비분할 행**(start/end_cut 빈 Vec): line_ranges 미산출 → 셀 전체 렌더(영향 없음).
- 현 중첩 표: cell_units 가 **atom 1개**(vis 0..line_count) → is_row_splittable=false(아래) →
  분할 경로 미진입.

## 설계 — 중첩행을 컷 1급 단위로

### (1) CellUnit 확장
```rust
struct CellUnit {
    height: f64,
    hard_break_before: bool,
    para_idx: usize,
    vis_start: usize,
    vis_end: usize,
    nested_row: Option<usize>,   // [신규] 이 유닛이 표현하는 중첩 표 행 인덱스
}
```
- 텍스트 줄 유닛: `nested_row=None` (기존 의미 불변).
- 중첩 표 유닛: para 당 **per-중첩행 유닛 N개**, `nested_row=Some(ri)`, height=중첩행 높이
  (+cell_spacing), vis_start/vis_end 는 해당 para 전체 줄 범위(폴백).

### (2) cell_units (table_layout.rs:3519)
- 중첩 표 보유 para 에서 `resolve_row_heights(nested)` 로 per-행 유닛 push.
- **2단계 이상 중첩 / rowspan 걸친 중첩행은 범위 외** → 기존 atom 폴백(가드).
- 단일 셀 1개 nested table(paragraphs.len()==1, find_map 첫 nested) 우선.

### (3) is_row_splittable (height_measurer.rs:1490)
- 현행: `line_heights.len() > 1`. 중첩 표 셀은 1 → false.
- 변경: MeasuredCell 에 `nested_row_count: usize`(신규) 추가(측정 시 채움). splittable =
  `line_heights.len() > 1 || cells.any(nested_row_count > 1)`.

### (4) row_cut_content_height (table_layout.rs:3862)
- 이미 `Σ units[su..eu].height` → per-중첩행 유닛에 자동 정합.
- **정합 가드(Stage 3 핵심)**: 비분할 행 `Σ(중첩행 높이) == calc_nested_table_height` 여야
  기존 측정과 동일(드리프트 0). 불일치 시 보정항 추가.

### (5) 부분 렌더 (table_partial.rs:1067 Control::Table)
- 현행: `available_h` 휴리스틱으로 `calc_nested_split_rows` 호출.
- 변경: 분할 행(start_cut/end_cut 존재)일 때 **컷 유닛 인덱스 → 중첩행 범위**로
  `NestedTableSplit{start_row,end_row,offset_within_start,visible_height}` 구성. 신규 헬퍼
  `cell_nested_split_from_cut(cell, start_unit, end_unit)`. 비분할 행은 현행 유지.

## 페이퍼 검증 (비회귀)

| 케이스 | cell_units | 분할 경로 | 동작 |
|------|-----------|----------|------|
| 중첩 표 없는 셀 | 기존 줄 유닛 | 불변 | 불변 |
| 중첩 표가 페이지에 들어감(비분할) | per-행 유닛 N | is_whole_row → line_ranges 미산출 | 셀 전체 렌더(불변) |
| 중첩 표 > 페이지(분할) | per-행 유닛 N | splittable → advance_row_cut 중첩행 컷 | **신규: 중첩행 경계 분할** |
| 2단계+ 중첩 / rowspan 중첩 | atom 폴백 | 불변 | 불변(범위 외) |
| TAC 중첩 표 | per-행 유닛(treat_as_char 무관) | 분할 가능 | 검증 필요(Stage 4) |

- 비분할 행 무영향의 근거: line_ranges 는 `is_in_split_row` 에서만 산출(table_partial.rs:569).
- row_cut_content_height 의 whole-row 합이 atom 높이와 동일해야 함(정합 가드, Stage 3 측정).

## 한컴 정합 기준
- 중첩행 경계 분할(부분 행 미발생) — `pdf/kps-ai-2022.pdf` p62~63 정합.
- 부분 행(단일 중첩행 > 페이지)은 우선 행 경계까지만(부분 행 미분할), 발생 시 Stage 4 확장.

## 리스크 / Stage 3 선결
- **whole-row 높이 정합**: `Σ 중첩행 높이(+spacing) == calc_nested_table_height` 검증 필수
  (드리프트 0 가드). 불일치 시 전수 sweep 광범위 회귀.
- 공유 함수(cell_units/row_cut_content_height/cell_line_ranges_from_cut) 변경 → Stage 3 에서
  dump-pages(kps-ai 청크) + 전수 sweep 단계적 측정.

## 다음 (Stage 3)
측정·컷 레이어 구현 → kps-ai dump-pages 중첩행 청크 확인 + whole-row 높이 정합 + 전수 sweep 1차.
